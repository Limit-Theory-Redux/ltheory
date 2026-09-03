local Component = require("Core.ECS.Component")

---@class ProjectileComponent: Component
---@overload fun(source: Entity, velocity: Vec3f, damage: number, lifetime: number, guidance?: table, targetBody?: table, targetEntity?: Entity, visual?: table): ProjectileComponent
local ProjectileComponent = Subclass("ProjectileComponent", Component, function(self, source, velocity, damage, lifetime, guidance, targetBody, targetEntity, visual)
    self:setComponentName("Projectile")
    self.source = source
    self.velocity = velocity or Vec3f()
    self.damage = damage or 0
    self.remainingLifetime = lifetime or 0
    self.guidance = guidance
    self.targetBody = targetBody
    self.targetEntity = targetEntity
    self.visual = visual or {}
    self.dissipationDuration = math.max(0, self.visual.dissipationDuration or 0)
    self.guidanceFuel = guidance and guidance.fuelLifetime or nil
    self.guidanceState = {}
    self.previousPosition = nil
end)

function ProjectileComponent:getSource()
    return self.source
end

function ProjectileComponent:getVelocity()
    return self.velocity
end

---@param velocity Vec3f
function ProjectileComponent:setVelocity(velocity)
    self.velocity = velocity
end

function ProjectileComponent:getDamage()
    return self.damage
end

function ProjectileComponent:getRemainingLifetime()
    return self.remainingLifetime
end

---@param lifetime number
function ProjectileComponent:setRemainingLifetime(lifetime)
    self.remainingLifetime = lifetime
end

return ProjectileComponent
