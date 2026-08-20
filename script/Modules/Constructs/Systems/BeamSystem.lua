---Beam lifecycle: endpoint validation, damage ticks, duration/cleanup.
---
---Runs after ProjectileSystem in the tick order owned by the host state's
---Sim handler (see AIWeaponSystem.lua header).
local CoreComponents = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local RenderingComponents = require("Modules.Rendering.Components")
local RaycastHelper = require("Shared.Helpers.RaycastHelper")
local BeamAimHelper = require("Shared.Helpers.BeamAimHelper")
local ffi = require("ffi")

local NULL_RIGID_BODY = ffi.cast("RigidBody*", nil)
local TICK_EPSILON = 1e-9

---@class BeamSystem
---@overload fun(): BeamSystem
local BeamSystem = Class("BeamSystem", function() end)

---@param entity Entity|nil
---@return RigidBody|nil
local function getRigidBody(entity)
    if not entity or not entity.get then
        return nil
    end
    local bodyComponent = entity:get(PhysicsComponents.RigidBody)
    return bodyComponent and bodyComponent:getRigidBody()
end

---@param entity Entity|nil
---@return Vec3f|nil
local function getEntityPosition(entity)
    local body = getRigidBody(entity)
    if body and body.getPos then
        return body:getPos()
    end
    if not entity or not entity.get then
        return nil
    end
    local transform = entity:get(PhysicsComponents.Transform)
    return transform and transform.getPos and transform:getPos() or nil
end

---@param beam table
---@param component BeamComponent
---@param sourcePosition Vec3f|nil
---@param targetPoint Vec3f|nil
---@param hitPosition Vec3f|nil
local function updateBeamLightTransform(beam, component, sourcePosition, targetPoint, hitPosition)
    local entity = beam and beam.entity
    if not entity or (entity.isValid and not entity:isValid()) then
        return
    end
    local transform = entity:get(PhysicsComponents.Transform)
    local pointLight = entity:get(RenderingComponents.PointLight)
    local endpoint = hitPosition or targetPoint
    if not endpoint then
        return
    end

    local visual = component:getVisual() or {}
    local radius = math.max(0, visual.lightRadius or 0)
    local standoff = radius > 0
        and math.max(0.01, math.min(radius * 0.25, 0.12))
        or 0.12
    local dx = (sourcePosition and sourcePosition.x or endpoint.x) - endpoint.x
    local dy = (sourcePosition and sourcePosition.y or endpoint.y) - endpoint.y
    local dz = (sourcePosition and sourcePosition.z or endpoint.z) - endpoint.z
    local length = math.sqrt(dx * dx + dy * dy + dz * dz)
    local endpointPosition = endpoint
    if length > 1e-6 then
        endpointPosition = Vec3f(
            endpoint.x + dx / length * standoff,
            endpoint.y + dy / length * standoff,
            endpoint.z + dz / length * standoff)
    end

    if transform then
        transform:setPos(Position(
            endpointPosition.x,
            endpointPosition.y,
            endpointPosition.z))
    end
    if not pointLight then
        return
    end

    -- The visible beam already supplies the lengthwise additive treatment. Keep
    -- deferred point lighting at the actual impact endpoint only; a synthetic
    -- source or midpoint light reads as an unrelated floating bulb and does
    -- not reliably illuminate a hull.
    pointLight:setSources(nil)
end

---@param component BeamComponent
---@return Vec3f|nil base target point
local function updateTargetPoint(component)
    local targetPointLocal = component:getTargetPointLocal()
    local target = component:getTarget()
    local baseTargetPoint = component.getBaseTargetPoint
        and component:getBaseTargetPoint()
        or component:getTargetPoint()
    if not targetPointLocal or not target
        or (target.isValid and not target:isValid())
    then
        return baseTargetPoint
    end

    local targetBodyComponent = target:get(PhysicsComponents.RigidBody)
    local targetBody = targetBodyComponent and targetBodyComponent:getRigidBody()
    if not targetBody then
        return baseTargetPoint
    end

    local scale = targetBody.getScale and targetBody:getScale() or 1
    local localPosition = Vec3f(
        targetPointLocal.x * scale,
        targetPointLocal.y * scale,
        targetPointLocal.z * scale)
    local rotation = targetBody.getRot and targetBody:getRot() or nil
    local rotated = rotation and rotation:mulV(localPosition) or localPosition
    local center = targetBody:getPos()
    local targetPoint = Vec3f(
        center.x + rotated.x,
        center.y + rotated.y,
        center.z + rotated.z)
    component.baseTargetPoint = targetPoint
    return targetPoint
end

---@param component BeamComponent
---@param duration number
---@return number damageDuration
local function getDamageDuration(component, duration)
    local effect = component:getEffect()
    local tickInterval = effect and effect.tickInterval or 0
    if not tickInterval or tickInterval <= 0 then
        return duration
    end

    local nextTick = component.nextTick
    if not nextTick or nextTick <= 0 then
        nextTick = tickInterval
    end

    local remaining = duration
    local damageDuration = 0
    while remaining > TICK_EPSILON do
        local step = math.min(remaining, nextTick)
        remaining = remaining - step
        nextTick = nextTick - step
        if nextTick <= TICK_EPSILON then
            damageDuration = damageDuration + tickInterval
            nextTick = tickInterval
        end
    end

    local beamEnds = component.remainingDuration <= duration + TICK_EPSILON
    if beamEnds and nextTick < tickInterval - TICK_EPSILON then
        damageDuration = damageDuration + nextTick
        nextTick = tickInterval
    end

    component.nextTick = nextTick
    return damageDuration
end

---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@param effect BeamDefinition|nil
---@param time number
---@param phase number
---@return Vec3f
function BeamSystem:computeSwayedTargetPoint(sourcePosition, targetPosition, effect, time, phase)
    return BeamAimHelper.computeEndpoint(
        sourcePosition,
        targetPosition,
        effect,
        time,
        phase,
        Vec3f())
end

---@param world Physics
---@param sourceBody RigidBody
---@param targetBody RigidBody
---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@return boolean, string, Vec3f|nil
function BeamSystem:validateHit(world, sourceBody, targetBody, sourcePosition, targetPosition)
    assert(world and sourceBody and targetBody and sourcePosition and targetPosition)
    local hit = RaycastHelper:castSegment(
        world,
        sourcePosition,
        targetPosition,
        { ignoreBody = sourceBody })
    local hitBody = hit and hit.body
    if hitBody == nil or hitBody == NULL_RIGID_BODY then
        return false, "none", hit and hit.position
    end
    if hitBody == targetBody then
        return true, "target", hit.position
    end
    return false, "blocked", hit.position
end

---@param state table
---@param sourceBody RigidBody|nil
---@param targetBody RigidBody|nil
---@param sourcePosition Vec3f|nil
---@param targetPoint Vec3f|nil
---@return boolean, string, Vec3f|nil
local function validateBeam(state, sourceBody, targetBody, sourcePosition, targetPoint)
    if not state.world then
        -- Isolated component tests do not have a physics world. The live
        -- testbed always supplies one and therefore takes the fail-closed path.
        return true, "unvalidated", nil
    end
    if not sourceBody or not targetBody or not sourcePosition or not targetPoint then
        return false, "missing_context", nil
    end
    return BeamSystem:validateHit(
        state.world,
        sourceBody,
        targetBody,
        sourcePosition,
        targetPoint)
end

---@param state table
---@param dt number
function BeamSystem:update(state, dt)
    local destroyedTargets = {}
    local destroyedTargetOrder = {}
    for index = #state.beams, 1, -1 do
        local beam = state.beams[index]
        local component = beam.component
        local duration = math.min(dt, math.max(0, component.remainingDuration))
        local baseTargetPoint = updateTargetPoint(component)
        local target = component:getTarget()
        local targetHealth = target and target:get(CoreComponents.Health)
        local targetBodyComponent = target and target:get(PhysicsComponents.RigidBody)
        local targetBody = beam.targetBody
            or (targetBodyComponent and targetBodyComponent:getRigidBody())
        if not baseTargetPoint and targetBody and targetBody.getPos then
            baseTargetPoint = targetBody:getPos()
            component.baseTargetPoint = baseTargetPoint
        end

        local sourceBody = beam.sourceBody or getRigidBody(component:getSource())
        local sourcePosition = beam.sourcePosition
            or (sourceBody and sourceBody.getPos and sourceBody:getPos())
            or getEntityPosition(component:getSource())
        local swayTime = component.swayTime or 0
        local targetPoint = baseTargetPoint
        if sourcePosition and baseTargetPoint then
            local swayBasis = BeamAimHelper.getBasis(
                sourcePosition,
                baseTargetPoint,
                component.swayBasis)
            targetPoint = BeamAimHelper.computeEndpoint(
                sourcePosition,
                baseTargetPoint,
                component:getEffect(),
                swayTime,
                component.swayPhase or 0,
                component.aimAngles,
                swayBasis)
            component.swayBasis = swayBasis
        end
        component.swayTime = swayTime + duration
        component.targetPoint = targetPoint
        beam.baseTargetPoint = baseTargetPoint
        beam.targetPoint = targetPoint

        -- Damage validation raycasts to the TRUE hull point; sway only
        -- offsets the visual endpoint so beams don't "miss" a target they
        -- are locked onto.
        local hitValidated, hitReason, hitPosition = validateBeam(
            state,
            sourceBody,
            targetBody,
            sourcePosition,
            baseTargetPoint or targetPoint)
        component.hitValidated = hitValidated
        component.hitReason = hitReason
        beam.hitValidated = hitValidated
        beam.hitReason = hitReason
        beam.hitPosition = hitPosition
        updateBeamLightTransform(beam, component, sourcePosition, targetPoint, hitPosition)
        local damageDuration = getDamageDuration(component, duration)

        if targetHealth and not targetHealth:isDestroyed()
            and damageDuration > 0 and hitValidated
        then
            targetHealth:setCurrentHealth(
                targetHealth:getCurrentHealth() - component.damagePerSecond * damageDuration)
            state.lastBeamImpact = {
                shotSerial = beam.shotSerial,
                mountId = beam.mountId,
                position = hitPosition or targetPoint or (targetBody and targetBody:getPos()),
            }
            if targetHealth:isDestroyed() and not destroyedTargets[target] then
                destroyedTargets[target] = true
                table.insert(destroyedTargetOrder, target)
                Log.Info("WeaponSystem beam destroyed target entity "
                    .. tostring(target.id))
            end
        end
        component.remainingDuration = component.remainingDuration - dt
        if component.remainingDuration <= 0
            or not targetHealth
            or targetHealth:isDestroyed()
        then
            state:removeBeam(index)
        end
    end
    if state.onTargetDestroyed then
        for _, target in ipairs(destroyedTargetOrder) do
            state:onTargetDestroyed(target)
        end
    end
end

---@param beam table
---@param health HealthComponent
---@return boolean
function BeamSystem:applyDamage(beam, health)
    if not beam or not beam.component or not health or health:isDestroyed() then
        return false
    end

    health:setCurrentHealth(health:getCurrentHealth() - beam.component.damagePerSecond)
    return true
end

return BeamSystem()
