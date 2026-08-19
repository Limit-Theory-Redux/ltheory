local Component = require("Core.ECS.Component")

---@class BeamComponent: Component
---@overload fun(source: Entity, target: Entity, effect: BeamDefinition, damagePerSecond: number, duration: number, targetPoint: Vec3f|nil, visual: table|nil, targetPointLocal: Vec3f|nil, aimAngles: Vec3f|nil, swayPhase: number|nil, swayTime: number|nil, swayBasis: table|nil): BeamComponent
local BeamComponent = Subclass("BeamComponent", Component, function(self, source, target, effect, damagePerSecond, duration, targetPoint, visual, targetPointLocal, aimAngles, swayPhase, swayTime, swayBasis)
    self:setComponentName("Beam")
    self.source = source
    self.target = target
    self.effect = effect
    self.damagePerSecond = damagePerSecond or 0
    self.duration = duration or 0
    self.baseTargetPoint = targetPoint
    self.targetPoint = targetPoint
    self.targetPointLocal = targetPointLocal
    self.aimAngles = aimAngles or Vec3f()
    self.swayPhase = swayPhase or 0
    self.swayTime = swayTime or 0
    self.swayBasis = swayBasis
    self.visual = visual or effect.visual
    self.remainingDuration = self.duration
    local tickInterval = effect and effect.tickInterval or 0
    self.nextTick = tickInterval and tickInterval > 0 and tickInterval or 0
end)

function BeamComponent:getSource()
    return self.source
end

function BeamComponent:getTarget()
    return self.target
end

function BeamComponent:getTargetPoint()
    return self.targetPoint
end

function BeamComponent:getBaseTargetPoint()
    return self.baseTargetPoint
end

function BeamComponent:getTargetPointLocal()
    return self.targetPointLocal
end

function BeamComponent:getEffect()
    return self.effect
end

function BeamComponent:getVisual()
    return self.visual
end

function BeamComponent:getDamagePerSecond()
    return self.damagePerSecond
end

function BeamComponent:getRemainingDuration()
    return self.remainingDuration
end

return BeamComponent
