---Projectile movement, guidance, collision, damage, lifetime.
---
---Runs after WeaponSystem:update in the tick order owned by the host
---state's Sim handler (see AIWeaponSystem.lua header).
---@class ProjectileSystem
local PhysicsComponents = require("Modules.Physics.Components")
---@overload fun(): ProjectileSystem
local ProjectileSystem = Class("ProjectileSystem", function() end)

local EPSILON = 0.000001

---@param startPosition Vec3f
---@param endPosition Vec3f
---@param center Vec3f
---@param radius number
---@return table|nil
function ProjectileSystem:segmentSphereHit(startPosition, endPosition, center, radius)
    assert(startPosition and endPosition and center and radius >= 0)

    local dx = endPosition.x - startPosition.x
    local dy = endPosition.y - startPosition.y
    local dz = endPosition.z - startPosition.z
    local fx = startPosition.x - center.x
    local fy = startPosition.y - center.y
    local fz = startPosition.z - center.z

    local a = dx * dx + dy * dy + dz * dz
    local c = fx * fx + fy * fy + fz * fz - radius * radius

    if c <= 0 then
        return { t = 0, position = Vec3f(startPosition.x, startPosition.y, startPosition.z) }
    end
    if a < EPSILON then
        return nil
    end

    local b = 2 * (fx * dx + fy * dy + fz * dz)
    local discriminant = b * b - 4 * a * c
    if discriminant < 0 then
        return nil
    end

    local root = math.sqrt(math.max(0, discriminant))
    local first = (-b - root) / (2 * a)
    local second = (-b + root) / (2 * a)
    local t = nil

    if first >= 0 and first <= 1 then
        t = first
    end
    if second >= 0 and second <= 1 and (not t or second < t) then
        t = second
    end
    if not t then
        return nil
    end

    return {
        t = t,
        position = Vec3f(
            startPosition.x + dx * t,
            startPosition.y + dy * t,
            startPosition.z + dz * t
        )
    }
end

local function length(vector)
    return math.sqrt(vector.x * vector.x + vector.y * vector.y + vector.z * vector.z)
end

local function normalize(vector)
    local magnitude = length(vector)
    if magnitude < EPSILON then
        return Vec3f(0, 0, -1)
    end
    return Vec3f(vector.x / magnitude, vector.y / magnitude, vector.z / magnitude)
end

local function scale(vector, amount)
    return Vec3f(vector.x * amount, vector.y * amount, vector.z * amount)
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

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
end

local function rotateDirection(current, desired, maximumAngle)
    maximumAngle = math.max(0, maximumAngle or 0)
    local cosine = clamp(dot(current, desired), -1, 1)
    local angle = math.acos(cosine)
    if angle <= maximumAngle or angle < EPSILON then
        return desired
    end

    local axis = cross(current, desired)
    local axisLength = length(axis)
    if axisLength < EPSILON then
        local basis = math.abs(current.x) < 0.8 and Vec3f(1, 0, 0) or Vec3f(0, 1, 0)
        axis = normalize(cross(current, basis))
    else
        axis = scale(axis, 1 / axisLength)
    end
    local cosineStep = math.cos(maximumAngle)
    local sineStep = math.sin(maximumAngle)
    return normalize(Vec3f(
        current.x * cosineStep + cross(axis, current).x * sineStep,
        current.y * cosineStep + cross(axis, current).y * sineStep,
        current.z * cosineStep + cross(axis, current).z * sineStep))
end

---@param component ProjectileComponent
---@param position Vec3f
---@param dt number
function ProjectileSystem:updateGuidance(component, position, dt)
    local guidance = component.guidance
    local targetBody = component.targetBody
    local targetEntity = component.targetEntity
    component.guidanceState = component.guidanceState or {}
    if not guidance or not targetBody or not targetBody.getPos then
        return
    end
    if targetEntity and targetEntity.isValid and not targetEntity:isValid() then
        component.guidanceState.targetLost = true
        component.targetBody = nil
        component.targetEntity = nil
        return
    end
    if component.guidanceFuel and component.guidanceFuel <= 0 then
        component.guidanceState.fuelExpired = true
        return
    end

    local targetPosition = targetBody:getPos()
    local targetVelocity = targetBody.getVelocity and targetBody:getVelocity() or Vec3f()
    local seekerLead = guidance.seekerLeadTime or 0.35
    local desiredPosition = Position(
        targetPosition.x + targetVelocity.x * seekerLead,
        targetPosition.y + targetVelocity.y * seekerLead,
        targetPosition.z + targetVelocity.z * seekerLead)
    local desiredDirection = normalize(Vec3f(
        desiredPosition.x - position.x,
        desiredPosition.y - position.y,
        desiredPosition.z - position.z))
    local velocity = component:getVelocity()
    local speed = length(velocity)
    local currentDirection = normalize(velocity)
    local maximumTurnRate = guidance.maximumTurnRate or math.rad(45)
    local direction = rotateDirection(currentDirection, desiredDirection, maximumTurnRate * dt)
    local acceleration = guidance.maximumAcceleration or guidance.thrust or 0
    local maximumSpeed = guidance.maximumSpeed or (speed + acceleration * math.max(dt, 0.1))
    local newSpeed = math.min(maximumSpeed, speed + acceleration * dt)
    component:setVelocity(Vec3f(direction.x * newSpeed, direction.y * newSpeed, direction.z * newSpeed))
    if component.guidanceFuel then
        component.guidanceFuel = math.max(0, component.guidanceFuel - dt)
        if component.guidanceFuel <= 0 then
            component.guidanceState.fuelExpired = true
        end
    end
    component.guidanceState.targetPosition = desiredPosition
    component.guidanceState.direction = direction
    component.guidanceState.turnRate = math.min(maximumTurnRate,
        math.acos(clamp(dot(currentDirection, direction), -1, 1)) / math.max(dt, EPSILON))
end

---@param projectile table
---@return boolean
function ProjectileSystem:beginDissipation(projectile)
    if projectile.dissipating then
        return true
    end

    local component = projectile.component
    local duration = component and component.dissipationDuration or 0
    if duration <= 0 then
        return false
    end

    projectile.dissipating = true
    projectile.dissipationDuration = duration
    projectile.dissipationRemaining = duration
    projectile.lightBaseIntensity = projectile.lightBaseIntensity
        or (projectile.entity
            and projectile.entity:get(require("Modules.Rendering.Components").PointLight)
            and projectile.entity:get(require("Modules.Rendering.Components").PointLight):getIntensity())
        or projectile.lightIntensity

    if projectile.effect then
        projectile.effect.lifeMax = duration
        projectile.effect.life = duration
        projectile.effect.vel = Vec3f()
    end
    return true
end

---@param state table
---@param dt number
function ProjectileSystem:update(state, dt)
    local targetDestroyed = false
    for index = #state.projectiles, 1, -1 do
        local projectile = state.projectiles[index]
        local component = projectile.component

        if projectile.dissipating then
            projectile.dissipationRemaining = projectile.dissipationRemaining - dt
            local fraction = math.max(0,
                projectile.dissipationRemaining / projectile.dissipationDuration)
            if projectile.effect then
                projectile.effect.life = math.max(0, projectile.dissipationRemaining)
                projectile.effect.vel = Vec3f()
            end
            if projectile.entity then
                local pointLight = projectile.entity:get(
                    require("Modules.Rendering.Components").PointLight)
                if pointLight then
                    pointLight:setIntensity((projectile.lightBaseIntensity or 0) * fraction)
                end
            end
            if projectile.dissipationRemaining <= 0 then
                state:removeProjectile(index)
            end
            goto continue
        end

        local body = projectile.body
        local startPosition = component.previousPosition or body:getPos()
        self:updateGuidance(component, startPosition, dt)
        local velocity = component:getVelocity()
        local endPosition = Position(
            startPosition.x + velocity.x * dt,
            startPosition.y + velocity.y * dt,
            startPosition.z + velocity.z * dt)

        body:setPos(endPosition)
        component.previousPosition = endPosition

        local stepDx = endPosition.x - startPosition.x
        local stepDy = endPosition.y - startPosition.y
        local stepDz = endPosition.z - startPosition.z
        local stepDistance = math.sqrt(stepDx * stepDx + stepDy * stepDy + stepDz * stepDz)
        local effect = projectile.effect
        if effect then
            effect.pos = endPosition
            effect.vel = velocity
            if stepDistance > EPSILON then
                effect.dir = Vec3f(
                    stepDx / stepDistance,
                    stepDy / stepDistance,
                    stepDz / stepDistance)
            end
            projectile.pulseDistance = (projectile.pulseDistance or 0) + stepDistance
            effect.dist = projectile.pulseDistance
            effect.life = math.max(0, component.remainingLifetime)
        end

        -- Hit detection sweeps ALL live Targetable entities in the ECS
        -- registry (multi-contact): the closest segment-sphere intersection
        -- along this step wins. Guided projectiles keep their proximity
        -- bonus against their own target; unguided shots hit whatever they
        -- actually touch. The projectile's own source entity is excluded.
        local Registry = require("Core.ECS.Registry")
        local CoreComponents = require("Modules.Core.Components")
        local ConstructComponents = require("Modules.Constructs.Components")
        local bestHit, bestHitEntity, bestHitT = nil, nil, math.huge
        for entity, targetable in
            Registry:iterEntities(ConstructComponents.Targetable)
        do
            if entity ~= component.source
                and targetable:isEnabled()
            then
                local health = entity:get(CoreComponents.Health)
                if not health or not health:isDestroyed() then
                    local rbComponent = entity:get(PhysicsComponents.RigidBody)
                    local contactRadius = rbComponent and rbComponent:getRadius() or 0
                    local guidance = component.guidance
                    if guidance and guidance.proximityRadius
                        and component.targetEntity == entity
                    then
                        contactRadius = math.max(
                            contactRadius, guidance.proximityRadius)
                    end
                    if contactRadius > 0 then
                        local contactBody = rbComponent
                            and rbComponent:getRigidBody()
                        if contactBody then
                            local hit = self:segmentSphereHit(
                                startPosition,
                                endPosition,
                                contactBody:getPos(),
                                contactRadius)
                            if hit and (hit.t or 1) < bestHitT then
                                bestHit = hit
                                bestHitEntity = entity
                                bestHitT = hit.t or 1
                            end
                        end
                    end
                end
            end
        end
        if bestHit then
            local hitPosition = bestHit.position
            body:setPos(Position(hitPosition.x, hitPosition.y, hitPosition.z))
            component.previousPosition = Position(
                hitPosition.x,
                hitPosition.y,
                hitPosition.z)
            if effect then
                local hitDistance = stepDistance * bestHitT
                projectile.pulseDistance = math.max(0,
                    (projectile.pulseDistance or 0) - stepDistance + hitDistance)
                effect.pos = Position(hitPosition.x, hitPosition.y, hitPosition.z)
                effect.dist = projectile.pulseDistance
            end
            -- Damage the contact actually struck.
            local impactHealth = bestHitEntity
                and bestHitEntity:get(require("Modules.Core.Components").Health)
            if impactHealth then
                self:applyDamage(component, impactHealth)
                state.lastImpact = {
                    shotSerial = projectile.shotSerial,
                    mountId = projectile.mountId,
                    position = hitPosition,
                }
                if impactHealth:isDestroyed() then
                    Log.Info("WeaponSystem projectile destroyed target entity "
                        .. tostring(bestHitEntity and bestHitEntity.id or "?"))
                end
                targetDestroyed = impactHealth:isDestroyed() or targetDestroyed
            end
            if not self:beginDissipation(projectile) then
                state:removeProjectile(index)
            end
            goto continue
        end

        component.remainingLifetime = component.remainingLifetime - dt
        if component.remainingLifetime <= 0 then
            if not self:beginDissipation(projectile) then
                state:removeProjectile(index)
            end
        end

        ::continue::
    end
    if targetDestroyed and state.onTargetDestroyed then
        state:onTargetDestroyed()
    end
end

---@param state table
---@param projectile table
---@param hit table
---@param deferTargetDestroyed boolean|nil
---@return boolean
function ProjectileSystem:applyImpact(state, projectile, hit, deferTargetDestroyed)
    assert(state and projectile and projectile.component and hit and hit.position)
    local component = projectile.component
    local health = component.targetEntity
        and component.targetEntity.isValid
        and component.targetEntity:isValid()
        and component.targetEntity:get(require("Modules.Core.Components").Health)
    if not health or not self:applyDamage(projectile.component, health) then
        return false
    end

    state.lastImpact = {
        shotSerial = projectile.shotSerial,
        mountId = projectile.mountId,
        position = hit.position,
    }

    if health:isDestroyed() and not deferTargetDestroyed and state.onTargetDestroyed then
        state:onTargetDestroyed()
    end
    return true
end

---@param projectile ProjectileComponent
---@param health HealthComponent
---@return boolean
function ProjectileSystem:applyDamage(projectile, health)
    if not projectile or not health or health:isDestroyed() then
        return false
    end

    health:setCurrentHealth(health:getCurrentHealth() - projectile:getDamage())
    return true
end

return ProjectileSystem()
