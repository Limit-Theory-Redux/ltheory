---@class ProjectileSystem
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

---@param state table
---@param dt number
function ProjectileSystem:update(state, dt)
    for index = #state.projectiles, 1, -1 do
        local projectile = state.projectiles[index]
        local component = projectile.component
        local body = projectile.body
        local startPosition = component.previousPosition or body:getPos()
        local velocity = component:getVelocity()
        local endPosition = Position(
            startPosition.x + velocity.x * dt,
            startPosition.y + velocity.y * dt,
            startPosition.z + velocity.z * dt)

        body:setPos(endPosition)
        component.previousPosition = endPosition
        component.remainingLifetime = component.remainingLifetime - dt

        local effect = projectile.effect
        if effect then
            local stepDx = endPosition.x - startPosition.x
            local stepDy = endPosition.y - startPosition.y
            local stepDz = endPosition.z - startPosition.z
            local stepDistance = math.sqrt(stepDx * stepDx + stepDy * stepDy + stepDz * stepDz)
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

        local targetAlive = state.targetHealth and not state.targetHealth:isDestroyed()
        if targetAlive then
            local hit = self:segmentSphereHit(
                startPosition,
                endPosition,
                state.targetBody:getPos(),
                state.targetRadius)
            if hit then
                self:applyImpact(state, projectile, hit)
                state:removeProjectile(index)
                goto continue
            end
        end

        if component.remainingLifetime <= 0 then
            state:removeProjectile(index)
        end

        ::continue::
    end
end

---@param state table
---@param projectile table
---@param hit table
---@return boolean
function ProjectileSystem:applyImpact(state, projectile, hit)
    assert(state and projectile and projectile.component and hit and hit.position)
    local health = state.targetHealth
    if not self:applyDamage(projectile.component, health) then
        return false
    end

    state.lastImpact = {
        shotSerial = projectile.shotSerial,
        mountId = projectile.mountId,
        position = hit.position,
    }

    if health:isDestroyed() and state.onTargetDestroyed then
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
