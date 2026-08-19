local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local BeamAimHelper = require("Shared.Helpers.BeamAimHelper")
local ProceduralCatalog = require("Shared.Content.ProceduralCatalog")

---@class WeaponTrackingSystem
---@overload fun(): WeaponTrackingSystem
local WeaponTrackingSystem = Class("WeaponTrackingSystem", function() end)

local ROOT_EPSILON = 0.000001
local DEFAULT_SAMPLE_RATE = 12.0
local DEFAULT_VELOCITY_RESPONSE = 0.75
local DEFAULT_ACCELERATION_RESPONSE = 0.35
local DEFAULT_PREDICTION_HORIZON = 8.0
local INTERCEPT_SAMPLES = 48
local INTERCEPT_ITERATIONS = 32

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
end

local function copyVector(value)
    return Vec3f(value.x, value.y, value.z)
end

local function add(a, b)
    return Vec3f(a.x + b.x, a.y + b.y, a.z + b.z)
end

local function subtract(a, b)
    return Vec3f(a.x - b.x, a.y - b.y, a.z - b.z)
end

local function scale(value, amount)
    return Vec3f(value.x * amount, value.y * amount, value.z * amount)
end

local function dot(a, b)
    return a.x * b.x + a.y * b.y + a.z * b.z
end

local function cross(a, b)
    return Vec3f(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x)
end

local function length(value)
    return math.sqrt(dot(value, value))
end

local function normalize(value)
    local magnitude = length(value)
    if magnitude <= ROOT_EPSILON then
        return nil
    end
    return scale(value, 1 / magnitude)
end

local function lerp(a, b, amount)
    return add(a, scale(subtract(b, a), amount))
end

local function getBodyVelocity(body)
    if body and body.getVelocity then
        return copyVector(body:getVelocity())
    end
    return Vec3f(0, 0, 0)
end

local function getBodyAngularVelocity(body)
    if body and body.getVelocityA then
        return copyVector(body:getVelocityA())
    end
    return Vec3f(0, 0, 0)
end

local function getTrackingConfig(weapon, turret, component)
    local config = {}
    for key, value in pairs(weapon.tracking or {}) do
        config[key] = value
    end
    local moduleRecord = turret.trackingModuleRef
        and ProceduralCatalog:resolve(turret.trackingModuleRef)
    for key, value in pairs(moduleRecord and moduleRecord.stats or {}) do
        config[key] = value
    end
    for key, value in pairs(turret.trackingModule or {}) do
        config[key] = value
    end
    for key, value in pairs(turret.trackingModuleStats or {}) do
        config[key] = value
    end
    for key, value in pairs(component.moduleStats or {}) do
        config[key] = value
    end

    config.sampleRate = config.sampleRate or DEFAULT_SAMPLE_RATE
    config.velocityResponse = config.velocityResponse or DEFAULT_VELOCITY_RESPONSE
    config.accelerationResponse = config.accelerationResponse or DEFAULT_ACCELERATION_RESPONSE
    config.predictionHorizon = config.predictionHorizon or DEFAULT_PREDICTION_HORIZON
    config.aimTolerance = config.aimTolerance or math.rad(0.5)
    config.traverseRate = config.traverseRate or math.rad(90)
    config.confidenceFloor = config.confidenceFloor or 0.05
    config.turnRateFloor = config.turnRateFloor or 0.0005
    config.turnRateLimit = config.turnRateLimit or 8.0
    config.predictionDamping = config.predictionDamping or 0.35
    return config
end

local function resolveWeapon(turret)
    if turret.weaponRef and turret.weaponRef.kind == "procedural" then
        local generated = ProceduralCatalog:resolve(turret.weaponRef)
        assert(generated,
            "unregistered procedural weapon: " .. tostring(turret.weaponRef.canonicalKey))
        return generated
    end
    local weapon = WeaponRegistry:get(turret.weaponId)
    assert(weapon, "unregistered weapon: " .. tostring(turret.weaponId))
    return weapon
end

local function getTargetKey(state, targetBody)
    if state.targetGeneration ~= nil then
        return state.targetGeneration
    end
    if state.weaponTargetEntity and state.weaponTargetEntity.getId then
        return state.weaponTargetEntity:getId()
    end
    if state.targetEntity and state.targetEntity.getId then
        return state.targetEntity:getId()
    end
    return targetBody
end

local function clearComponent(component, targetKey)
    if component.reset then
        component:reset(targetKey)
        return
    end
    component.track = nil
    component.targetKey = targetKey
    component.sampleTime = 0
    component.confidence = 0
    component.mountSolutions = {}
    component.targetPointByMount = {}
    component.lastTargetPoint = nil
end

function WeaponTrackingSystem:updateMotionTrack(component, state, targetBody, targetKey, dt, config)
    local currentPosition = targetBody:getPos()
    local currentRotation = targetBody.getRot and targetBody:getRot() or Quat.Identity()
    local currentAngularVelocity = getBodyAngularVelocity(targetBody)
    local track = component.track

    if not track or component.targetKey ~= targetKey then
        component.targetKey = targetKey
        component.sampleTime = 0
        component.confidence = 0.2
        component.track = {
            position = copyVector(currentPosition),
            velocity = getBodyVelocity(targetBody),
            acceleration = Vec3f(0, 0, 0),
            rotation = currentRotation,
            angularVelocity = currentAngularVelocity,
            age = 0,
            samples = 1,
            confidence = component.confidence,
        }
        return component.track
    end

    local previousPosition = track.position
    local measuredVelocity = getBodyVelocity(targetBody)
    local finiteVelocity = measuredVelocity
    if dt > ROOT_EPSILON then
        finiteVelocity = scale(subtract(currentPosition, previousPosition), 1 / dt)
    end

    local velocityResponse = clamp(
        config.velocityResponse * dt * config.sampleRate,
        0,
        1)
    local accelerationResponse = clamp(
        config.accelerationResponse * dt * config.sampleRate,
        0,
        1)
    local rawAcceleration = Vec3f(0, 0, 0)
    if dt > ROOT_EPSILON then
        rawAcceleration = scale(subtract(finiteVelocity, track.velocity), 1 / dt)
    end

    local predictedPosition = self:predictPosition(track, dt, config)
    local residual = length(subtract(currentPosition, predictedPosition))
    local expectedStep = math.max(0.05, length(finiteVelocity) * math.max(dt, ROOT_EPSILON) * 2)
    local confidenceDelta = residual <= expectedStep and 0.045 or -0.08

    track.position = copyVector(currentPosition)
    track.velocity = lerp(track.velocity, finiteVelocity, velocityResponse)
    track.acceleration = lerp(track.acceleration, rawAcceleration, accelerationResponse)
    track.rotation = currentRotation
    track.angularVelocity = currentAngularVelocity
    track.age = 0
    track.samples = (track.samples or 1) + 1
    component.confidence = clamp((component.confidence or 0) + confidenceDelta, 0, 1)
    track.confidence = component.confidence
    return track
end

---@param track table
---@param time number
---@param config table
---@return Vec3f
function WeaponTrackingSystem:predictPosition(track, time, config)
    assert(track and time >= 0 and config)
    local position = track.position
    local velocity = track.velocity
    local acceleration = track.acceleration
    local speed = length(velocity)
    local accelerationAlongVelocity = 0
    local perpendicularAcceleration = acceleration
    if speed > ROOT_EPSILON then
        local direction = scale(velocity, 1 / speed)
        accelerationAlongVelocity = dot(acceleration, direction)
        perpendicularAcceleration = subtract(
            acceleration,
            scale(direction, accelerationAlongVelocity))
    end

    local perpendicularMagnitude = length(perpendicularAcceleration)
    local turnRate = speed > ROOT_EPSILON and perpendicularMagnitude / speed or 0
    if turnRate >= config.turnRateFloor and turnRate <= config.turnRateLimit then
        local axis = normalize(cross(velocity, acceleration))
        if axis then
            local parallelVelocity = speed > ROOT_EPSILON
                and scale(velocity, accelerationAlongVelocity / math.max(speed, ROOT_EPSILON))
                or Vec3f(0, 0, 0)
            local perpendicularVelocity = subtract(velocity, parallelVelocity)
            local rotatedVelocity = cross(axis, perpendicularVelocity)
            local angle = turnRate * time
            local sine = math.sin(angle)
            local cosine = math.cos(angle)
            local integratedPerpendicular = add(
                scale(perpendicularVelocity, sine / turnRate),
                scale(rotatedVelocity, (1 - cosine) / turnRate))
            local displacement = add(
                scale(parallelVelocity, time),
                integratedPerpendicular)
            if math.abs(accelerationAlongVelocity) > ROOT_EPSILON then
                displacement = add(
                    displacement,
                    scale(normalize(velocity) or Vec3f(0, 0, 0),
                        0.5 * accelerationAlongVelocity * time * time))
            end
            return add(position, displacement)
        end
    end

    local damping = config.predictionDamping
    local dampedAcceleration = time > ROOT_EPSILON
        and scale(acceleration, (1 - math.exp(-damping * time)) / damping)
        or Vec3f(0, 0, 0)
    return add(
        add(position, scale(velocity, time)),
        scale(dampedAcceleration, 0.5 * time))
end

function WeaponTrackingSystem:predictTargetPoint(track, targetPointLocal, targetScale, time, config)
    local predictedCenter = self:predictPosition(track, time, config or track.config)
    local localOffset = scale(targetPointLocal, targetScale)
    local currentOffset = track.rotation and track.rotation:mulV(localOffset) or localOffset
    local angularVelocity = track.angularVelocity
    local angularSpeed = length(angularVelocity)
    local predictedOffset = currentOffset
    if angularSpeed > ROOT_EPSILON then
        local axis = scale(angularVelocity, 1 / angularSpeed)
        local deltaRotation = Quat.FromAxisAngle(axis, angularSpeed * time)
        predictedOffset = deltaRotation:mulV(currentOffset)
    end
    return add(predictedCenter, predictedOffset)
end

function WeaponTrackingSystem:solveIntercept(
    origin,
    originVelocity,
    track,
    targetPointLocal,
    targetScale,
    projectileSpeed,
    config,
    range)
    assert(origin and originVelocity and track and targetPointLocal and config)
    if projectileSpeed == nil or projectileSpeed <= ROOT_EPSILON then
        return nil
    end

    local currentTarget = self:predictTargetPoint(track, targetPointLocal, targetScale, 0, config)
    local initialDistance = length(subtract(currentTarget, origin))
    local physicalTime = initialDistance / projectileSpeed
    local horizon = math.max(
        config.predictionHorizon,
        physicalTime * 1.5,
        range and range / projectileSpeed or 0)
    local function residual(time)
        local predicted = self:predictTargetPoint(track, targetPointLocal, targetScale, time, config)
        local source = add(origin, scale(originVelocity, time))
        return length(subtract(predicted, source)) - projectileSpeed * time
    end

    local previousTime = 0
    local previousResidual = residual(0)
    local bestTime = 0
    local bestError = math.abs(previousResidual)
    local lowerTime
    local upperTime
    for sampleIndex = 1, INTERCEPT_SAMPLES do
        local sampleTime = horizon * sampleIndex / INTERCEPT_SAMPLES
        local sampleResidual = residual(sampleTime)
        local sampleError = math.abs(sampleResidual)
        if sampleError < bestError then
            bestTime = sampleTime
            bestError = sampleError
        end
        if previousResidual >= 0 and sampleResidual <= 0 then
            lowerTime = previousTime
            upperTime = sampleTime
            break
        end
        previousTime = sampleTime
        previousResidual = sampleResidual
    end

    local time = bestTime
    if lowerTime then
        for _ = 1, INTERCEPT_ITERATIONS do
            local middleTime = (lowerTime + upperTime) * 0.5
            if residual(middleTime) > 0 then
                lowerTime = middleTime
            else
                upperTime = middleTime
            end
        end
        time = (lowerTime + upperTime) * 0.5
    end

    local predicted = self:predictTargetPoint(track, targetPointLocal, targetScale, time, config)
    return {
        time = time,
        position = predicted,
        confidence = track.confidence,
        model = lowerTime and "intercept" or "closest-approach",
    }
end

function WeaponTrackingSystem:getTargetPoint(state, sourcePosition, targetPosition, targetBody, seed, time)
    if not state.targetSurface then
        return targetPosition, nil, nil
    end
    local toSource = subtract(sourcePosition, targetPosition)
    local toSourceLength = length(toSource)
    local pointOptions = state.targetPointOptions or {}
    local sampledOptions = {
        motionAmplitude = pointOptions.motionAmplitude,
        motionFrequency = pointOptions.motionFrequency,
        minFacingDot = pointOptions.minFacingDot or 0.15,
    }
    if toSourceLength > ROOT_EPSILON then
        sampledOptions.viewDirection = scale(toSource, 1 / toSourceLength)
    end
    local sample = WeaponSystem:sampleTargetPoint(
        state.targetSurface,
        seed,
        time,
        sampledOptions)
    local worldPoint = WeaponSystem:targetPointToWorld(sample, targetBody)
    return worldPoint, Vec3f(sample.position.x, sample.position.y, sample.position.z), sample
end

function WeaponTrackingSystem:update(state, dt)
    local control = state.control
    local targetBody = state.weaponTargetBody or state.targetBody
    local component = state.weaponTrackingComponent
    if not control or not component then
        return
    end

    local targetKey = targetBody and getTargetKey(state, targetBody) or nil
    if not targetBody then
        clearComponent(component, targetKey)
        state.trackingByMount = {}
        state.targetPointByMount = {}
        state.lastTargetPoint = nil
        return
    end

    local firstWeapon = state.turrets and state.turrets[1]
        and resolveWeapon(state.turrets[1].component)
    local trackConfig = getTrackingConfig(
        firstWeapon or {},
        state.turrets and state.turrets[1] and state.turrets[1].component or {},
        component)
    component.sampleTime = (component.sampleTime or 0) + dt
    local track = self:updateMotionTrack(component, state, targetBody, targetKey, dt, trackConfig)
    track.config = trackConfig
    state.targetTrack = track
    state.targetTrackConfidence = component.confidence
    state.targetPointTime = (state.targetPointTime or 0) + dt
    state.aimStep = (state.aimStep or 0) + 1
    state.beamSwayBasisByMount = state.beamSwayBasisByMount or {}

    local trackingByMount = {}
    local targetPointByMount = {}
    local targetPointRng = RNG.Create(state.targetPointSeed or 0)
    local beamSwayRng = RNG.Create(state.beamSwaySeed or 0)
    assert(targetPointRng and beamSwayRng,
        "weapon tracking could not create deterministic mount RNG streams")
    for mountIndex, mount in ipairs(state.turrets or {}) do
        local turret = mount.component
        local weapon = resolveWeapon(turret)
        local effect = weapon.effect
        assert(effect, "ship weapon has no effect definition: " .. tostring(turret.weaponId))
        local sourcePosition = mount.body:getPos()
        local sourceVelocity = getBodyVelocity(mount.body)
        local targetPosition = targetBody:getPos()
        local targetPoint, targetPointLocal, targetPointSample = self:getTargetPoint(
            state,
            sourcePosition,
            targetPosition,
            targetBody,
            targetPointRng:get64(),
            state.targetPointTime)
        if targetPointSample then
            targetPointByMount[mount.mountId] = {
                position = targetPoint,
                triangleIndex = targetPointSample.triangleIndex,
                normal = targetPointSample.normal,
            }
        end

        local trackingConfig = getTrackingConfig(weapon, turret, component)
        local targetScale = targetBody.getScale and targetBody:getScale() or 1
        local solution
        local beamSwayPhase = 0
        local beamSwayBasis
        if effect.kind == Enums.Weapon.Effect.Beam then
            beamSwayPhase = beamSwayRng:getUniform() * math.pi * 2
            beamSwayBasis = BeamAimHelper.getBasis(
                sourcePosition,
                targetPoint,
                state.beamSwayBasisByMount[mount.mountId])
            state.beamSwayBasisByMount[mount.mountId] = beamSwayBasis
            solution = {
                time = 0,
                position = targetPoint,
            }
        else
            solution = self:solveIntercept(
                sourcePosition,
                sourceVelocity,
                track,
                targetPointLocal or Vec3f(0, 0, 0),
                targetScale,
                effect.speed,
                trackingConfig,
                weapon.range)
        end

        local aimPosition = solution and solution.position or targetPosition
        local aim = nil
        if solution then
            local accuracySeed = effect.kind == Enums.Weapon.Effect.Beam
                and mountIndex
                or state.aimStep * 17 + mountIndex
            local accurateAim = WeaponSystem:applyAccuracy(
                sourcePosition,
                solution.position,
                weapon,
                accuracySeed)
            if effect.kind == Enums.Weapon.Effect.Beam then
                accurateAim.position = BeamAimHelper.computeEndpoint(
                    sourcePosition,
                    targetPoint,
                    effect,
                    state.targetPointTime,
                    beamSwayPhase,
                    accurateAim.angles,
                    beamSwayBasis)
            end
            solution = {
                time = solution.time,
                position = accurateAim.position,
                aimOffset = accurateAim.offset,
                aimAngles = accurateAim.angles,
                targetPoint = targetPoint,
                targetPointLocal = targetPointLocal,
                swayPhase = beamSwayPhase,
                swayTime = state.targetPointTime,
                swayBasis = beamSwayBasis,
                confidence = solution.confidence,
                model = solution.model,
            }
            aimPosition = solution.position
            local parentRotation = mount.parentBody and mount.parentBody:getRot() or Quat.Identity()
            local mountRotation = parentRotation * turret:getLocalRotation()
            aim = WeaponSystem:aimTurret(
                turret,
                sourcePosition,
                aimPosition,
                dt,
                weapon,
                mountRotation,
                trackingConfig)
            local turretRotation = Quat.FromEuler(aim.yaw, aim.pitch, 0)
            mount.body:setRot(mountRotation * turretRotation)
        else
            turret.ready = false
            turret.aimError = math.huge
        end

        trackingByMount[mount.mountId] = {
            weapon = weapon,
            solution = solution,
            aimPosition = aimPosition,
            aim = aim,
            sourcePosition = sourcePosition,
            targetPoint = targetPoint,
            targetPointLocal = targetPointLocal,
            targetPointSample = targetPointSample,
        }
    end

    component.mountSolutions = trackingByMount
    component.targetPointByMount = targetPointByMount
    component.lastTargetPoint = state.mountIds
        and targetPointByMount[state.mountIds[1]]
        or nil
    state.trackingByMount = trackingByMount
    state.targetPointByMount = targetPointByMount
    state.lastTargetPoint = component.lastTargetPoint
end

return WeaponTrackingSystem()
