---@class WeaponSystem
---@overload fun(): WeaponSystem
local WeaponSystem = Class("WeaponSystem", function() end)

local ffi = require("ffi")
local ShipWeaponRegistry = require("Shared.Registries.ShipWeaponRegistry")
local NULL_RIGID_BODY = ffi.cast("RigidBody*", nil)
local ROOT_EPSILON = 0.000001

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
end

local function wrapAngle(angle)
    while angle > math.pi do
        angle = angle - 2 * math.pi
    end
    while angle < -math.pi do
        angle = angle + 2 * math.pi
    end
    return angle
end

local function stepAngle(current, target, maximumDelta)
    local delta = wrapAngle(target - current)
    if math.abs(delta) <= maximumDelta then
        return target
    end
    return current + (delta > 0 and maximumDelta or -maximumDelta)
end

---@param world Physics
---@param sourceBody RigidBody
---@param targetBody RigidBody
---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@param obstacles table[]|nil
---@return boolean, string, Vec3f|nil
function WeaponSystem:hasLineOfSight(world, sourceBody, targetBody, sourcePosition, targetPosition, obstacles)
    assert(world and sourceBody and targetBody and sourcePosition and targetPosition)

    local dx = targetPosition.x - sourcePosition.x
    local dy = targetPosition.y - sourcePosition.y
    local dz = targetPosition.z - sourcePosition.z
    local distanceSquared = dx * dx + dy * dy + dz * dz
    if distanceSquared < ROOT_EPSILON then
        return true
    end

    local distance = math.sqrt(distanceSquared)
    local invDistance = 1 / distance
    local nx = dx * invDistance
    local ny = dy * invDistance
    local nz = dz * invDistance
    local epsilon = math.min(0.001, distance * 0.01)

    local ray = Ray()
    ray.tMin = 0
    ray.tMax = 1
    ray.px = sourcePosition.x + nx * epsilon
    ray.py = sourcePosition.y + ny * epsilon
    ray.pz = sourcePosition.z + nz * epsilon
    ray.dirx = targetPosition.x - ray.px
    ray.diry = targetPosition.y - ray.py
    ray.dirz = targetPosition.z - ray.pz

    local result = world:rayCast(ray)
    local hitBody = result and result.body
    if hitBody == nil or hitBody == NULL_RIGID_BODY then
        return false, "none"
    end

    if hitBody == sourceBody then
        return false, "source", nil
    end

    if hitBody == targetBody then
        local hitPosition = hitBody.getPos and hitBody:getPos() or nil
        return true, "target", hitPosition
    end

    for _, obstacle in ipairs(obstacles or {}) do
        local obstacleBody = obstacle.body or obstacle
        if hitBody == obstacleBody then
            local hitPosition = hitBody.getPos and hitBody:getPos() or nil
            if obstacle.kind == "hull" then
                local center = obstacle.position
                    or (obstacleBody.getPos and obstacleBody:getPos())
                local radius = obstacle.radius
                    or (obstacleBody.getBoundingRadius and obstacleBody:getBoundingRadius())
                if center and radius then
                    local sourceOffsetX = sourcePosition.x - center.x
                    local sourceOffsetY = sourcePosition.y - center.y
                    local sourceOffsetZ = sourcePosition.z - center.z
                    local sourceRadiusSquared = sourceOffsetX * sourceOffsetX
                        + sourceOffsetY * sourceOffsetY
                        + sourceOffsetZ * sourceOffsetZ
                    local outwardDot = sourceOffsetX * nx
                        + sourceOffsetY * ny
                        + sourceOffsetZ * nz
                    local targetOffsetX = targetPosition.x - center.x
                    local targetOffsetY = targetPosition.y - center.y
                    local targetOffsetZ = targetPosition.z - center.z
                    local targetRadiusSquared = targetOffsetX * targetOffsetX
                        + targetOffsetY * targetOffsetY
                        + targetOffsetZ * targetOffsetZ
                    local radiusSquared = radius * radius

                    if sourceRadiusSquared < radiusSquared
                        and targetRadiusSquared > radiusSquared
                        and outwardDot > ROOT_EPSILON then
                        local discriminant = outwardDot * outwardDot
                            + radiusSquared - sourceRadiusSquared
                        local exitDistance = -outwardDot + math.sqrt(math.max(0, discriminant))
                        if exitDistance < distance - epsilon then
                            local exitPosition = Vec3f(
                                sourcePosition.x + nx * exitDistance,
                                sourcePosition.y + ny * exitDistance,
                                sourcePosition.z + nz * exitDistance)
                            return true, "target", exitPosition
                        end
                    end
                end
                return false, "hull", hitPosition
            end
            return false, obstacle.reason or "other", hitPosition
        end
    end

    local hitPosition = hitBody.getPos and hitBody:getPos() or nil
    return false, "other", hitPosition
end

---@param origin Vec3f
---@param originVelocity Vec3f
---@param targetPosition Vec3f
---@param targetVelocity Vec3f
---@param projectileSpeed number
---@return table|nil
function WeaponSystem:solveIntercept(origin, originVelocity, targetPosition, targetVelocity, projectileSpeed)
    assert(origin and originVelocity and targetPosition and targetVelocity)
    assert(projectileSpeed and projectileSpeed >= 0)

    local rx = targetPosition.x - origin.x
    local ry = targetPosition.y - origin.y
    local rz = targetPosition.z - origin.z

    local vx = targetVelocity.x - originVelocity.x
    local vy = targetVelocity.y - originVelocity.y
    local vz = targetVelocity.z - originVelocity.z

    local a = vx * vx + vy * vy + vz * vz - projectileSpeed * projectileSpeed
    local b = 2 * (rx * vx + ry * vy + rz * vz)
    local c = rx * rx + ry * ry + rz * rz

    local time

    if math.abs(a) < ROOT_EPSILON then
        if math.abs(b) < ROOT_EPSILON then
            if c < ROOT_EPSILON then
                time = 0
            else
                return nil
            end
        else
            local candidate = -c / b
            if candidate >= 0 then
                time = candidate
            else
                return nil
            end
        end
    else
        local discriminant = b * b - 4 * a * c
        if discriminant < 0 then
            return nil
        end

        local root = math.sqrt(math.max(0, discriminant))
        local first = (-b - root) / (2 * a)
        local second = (-b + root) / (2 * a)

        if first >= 0 then
            time = first
        end
        if second >= 0 and (not time or second < time) then
            time = second
        end
        if not time then
            return nil
        end
    end

    return {
        time = time,
        position = Vec3f(
            targetPosition.x + targetVelocity.x * time,
            targetPosition.y + targetVelocity.y * time,
            targetPosition.z + targetVelocity.z * time
        )
    }
end

---@param mode string
---@param mountIds string[]
---@param readyByMount table<string, boolean>
---@param sequenceIndex integer|nil
---@return table
function WeaponSystem:planFire(mode, mountIds, readyByMount, sequenceIndex)
    assert(mode == "volley" or mode == "sequence", "unsupported weapon fire mode")
    assert(mountIds and readyByMount)

    local plan = {
        shots = {},
        nextIndex = sequenceIndex or 1
    }

    if #mountIds == 0 then
        return plan
    end

    if mode == "volley" then
        for _, mountId in ipairs(mountIds) do
            if readyByMount[mountId] then
                table.insert(plan.shots, mountId)
            end
        end
        return plan
    end

    local index = sequenceIndex or 1
    index = ((index - 1) % #mountIds) + 1
    for offset = 0, #mountIds - 1 do
        local candidateIndex = ((index + offset - 1) % #mountIds) + 1
        local mountId = mountIds[candidateIndex]
        if readyByMount[mountId] then
            table.insert(plan.shots, mountId)
            plan.nextIndex = (candidateIndex % #mountIds) + 1
            break
        end
    end

    return plan
end

---@param sizeClass string|nil
---@param aiConfig table|nil
---@return string
function WeaponSystem:selectFireMode(sizeClass, aiConfig)
    aiConfig = aiConfig or {}
    local modeBySizeClass = aiConfig.modeBySizeClass or {}
    local mode = modeBySizeClass[sizeClass or "small"] or aiConfig.defaultMode or "sequence"
    assert(mode == "volley" or mode == "sequence", "AI weapon mode must be volley or sequence")
    return mode
end

---@param origin Vec3f
---@param candidates table[]
---@param range number
---@return table|nil
function WeaponSystem:selectNearestTarget(origin, candidates, range)
    assert(origin and candidates and range >= 0)

    local rangeSquared = range * range
    local best = nil
    local bestDistanceSquared = math.huge

    for _, candidate in ipairs(candidates) do
        if candidate.enabled ~= false and candidate.position then
            local dx = candidate.position.x - origin.x
            local dy = candidate.position.y - origin.y
            local dz = candidate.position.z - origin.z
            local distanceSquared = dx * dx + dy * dy + dz * dz

            local closer = distanceSquared < bestDistanceSquared
            local tied = math.abs(distanceSquared - bestDistanceSquared) < ROOT_EPSILON
            local lowerId = best and candidate.id < best.id

            if distanceSquared <= rangeSquared and (not best or closer or (tied and lowerId)) then
                best = candidate
                bestDistanceSquared = distanceSquared
            end
        end
    end

    return best
end

---@param turret TurretComponent
---@param origin Vec3f
---@param target Vec3f
---@param dt number
---@param weapon table
---@return table
function WeaponSystem:aimTurret(turret, origin, target, dt, weapon)
    assert(turret and origin and target and dt >= 0 and weapon)

    local direction = Vec3f(
        target.x - origin.x,
        target.y - origin.y,
        target.z - origin.z
    )
    local desiredYaw, desiredPitch = self:directionToAngles(direction)
    desiredYaw = clamp(desiredYaw, turret.yawMin, turret.yawMax)
    desiredPitch = clamp(desiredPitch, turret.pitchMin, turret.pitchMax)

    turret.desiredYaw = desiredYaw
    turret.desiredPitch = desiredPitch
    turret.yaw = stepAngle(turret.yaw, desiredYaw, (weapon.traverseRate or turret.traverseRate) * dt)
    turret.pitch = stepAngle(turret.pitch, desiredPitch, (weapon.traverseRate or turret.traverseRate) * dt)

    local yawError = math.abs(wrapAngle(desiredYaw - turret.yaw))
    local pitchError = math.abs(desiredPitch - turret.pitch)
    local aimError = math.max(yawError, pitchError)
    local ready = aimError <= (weapon.aimTolerance or 0.01)

    turret.ready = ready
    turret.aimError = aimError

    return {
        yaw = turret.yaw,
        pitch = turret.pitch,
        desiredYaw = desiredYaw,
        desiredPitch = desiredPitch,
        aimError = aimError,
        ready = ready,
    }
end

---@param state table
---@param dt number
function WeaponSystem:update(state, dt)
    local control = state.control
    local targetBody = state.weaponTargetBody or state.targetBody
    if not control then
        return
    end

    control.interShotGap = math.max(0, (control.interShotGap or 0) - dt)

    if not targetBody then
        state.readyByMount = {}
        state.sightCount = 0
        state.sightReasons = { none = 0, source = 0, target = 0, other = 0, hull = 0 }
        state.readyCount = 0
        state.sightHitPosition = nil
        for _, mount in ipairs(state.turrets or {}) do
            mount.component.fireSolution = nil
            mount.component.aimPosition = nil
            mount.component.inRange = false
            mount.component.hasLineOfSight = false
            mount.component.ready = false
        end
        return
    end

    local targetPosition = targetBody:getPos()
    local targetVelocity = targetBody:getVelocity()
    local readyByMount = {}
    local weaponByMount = {}
    local sightCount = 0
    local sightReasons = { none = 0, source = 0, target = 0, other = 0, hull = 0 }
    local readyCount = 0
    state.sightHitPosition = nil

    for _, mount in ipairs(state.turrets) do
        local turret = mount.component
        local body = mount.body
        local weapon = ShipWeaponRegistry:get(turret.weaponKey)
        assert(weapon, "unregistered ship weapon: " .. tostring(turret.weaponKey))
        weaponByMount[mount.mountId] = weapon
        turret.cooldown = math.max(0, turret.cooldown - dt)

        local sourcePosition = body:getPos()
        local sourceVelocity = body:getVelocity()
        local solution = self:solveIntercept(
            sourcePosition,
            sourceVelocity,
            targetPosition,
            targetVelocity,
            weapon.projectileSpeed)

        local aimPosition = solution and solution.position or targetPosition
        local aim = self:aimTurret(turret, sourcePosition, aimPosition, dt, weapon)
        body:setRot(Quat.FromEuler(aim.yaw, aim.pitch, 0))

        local distance = math.sqrt(
            (sourcePosition.x - targetPosition.x) ^ 2
            + (sourcePosition.y - targetPosition.y) ^ 2
            + (sourcePosition.z - targetPosition.z) ^ 2)
        local inRange = distance <= weapon.range
        local hasSight = false
        local sightReason = solution and "none" or "no_solution"
        local sightHitPosition = nil
        if solution ~= nil then
            hasSight, sightReason, sightHitPosition = self:hasLineOfSight(
                state.world,
                body,
                targetBody,
                sourcePosition,
                targetPosition,
                state.losObstacles)
        end
        if not sightReason then
            sightReason = solution and "none" or "no_solution"
        end
        if sightHitPosition and not state.sightHitPosition then
            state.sightHitPosition = sightHitPosition
        end

        if hasSight then
            sightCount = sightCount + 1
        end
        if sightReasons[sightReason] ~= nil then
            sightReasons[sightReason] = sightReasons[sightReason] + 1
        end

        turret.fireSolution = solution
        turret.aimPosition = aimPosition
        turret.inRange = inRange
        turret.hasLineOfSight = hasSight
        readyByMount[mount.mountId] = solution ~= nil
            and inRange
            and hasSight
            and aim.ready
            and turret.cooldown <= 0
        if readyByMount[mount.mountId] then
            readyCount = readyCount + 1
        end
    end

    state.readyByMount = readyByMount
    state.sightCount = sightCount
    state.sightReasons = sightReasons
    state.readyCount = readyCount

    if not control.triggerHeld or control.interShotGap > 0 then
        return
    end

    local order = control.mode == "sequence" and control.sequence or state.mountIds
    local plan = self:planFire(control.mode, order, readyByMount, control.sequenceIndex)
    if #plan.shots == 0 then
        return
    end

    for _, mountId in ipairs(plan.shots) do
        local mount = state.turretsById[mountId]
        local weapon = weaponByMount[mountId]
        if mount and mount.component.fireSolution then
            control.shotSerial = control.shotSerial + 1
            mount.component.cooldown = weapon.cooldown
            state:spawnProjectile(mount, mount.component.fireSolution, weapon, control.shotSerial)
        end
    end

    control.sequenceIndex = plan.nextIndex
    control.interShotGap = weaponByMount[plan.shots[1]].interShotGap
    state.lastShotOrder = plan.shots
end

---@param direction Vec3f
---@return number yaw, number pitch
function WeaponSystem:directionToAngles(direction)
    assert(direction)

    local horizontal = math.sqrt(direction.x * direction.x + direction.z * direction.z)
    if horizontal < ROOT_EPSILON and math.abs(direction.y) < ROOT_EPSILON then
        return 0, 0
    end

    return math.atan2(direction.x, direction.z), math.atan2(direction.y, horizontal)
end

return WeaponSystem()
