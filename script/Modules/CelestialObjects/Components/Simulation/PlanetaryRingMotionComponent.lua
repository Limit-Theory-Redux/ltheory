local Component = require("Core.ECS.Component")

---@class PlanetaryRingMotionComponent: Component
---@overload fun(self: PlanetaryRingMotionComponent, speed: number, phase: number): PlanetaryRingMotionComponent subclass internal
---@overload fun(speed: number, phase: number): PlanetaryRingMotionComponent subclass external
local PlanetaryRingMotionComponent = Subclass("PlanetaryRingMotionComponent", Component, function(self, speed, phase)
    self:setComponentName("CelestialPlanetaryRingMotionComponent")

    self.time = 0
    self:setSpeed(speed)
    self:setPhase(phase)
    self:registerEvents()
end)

function PlanetaryRingMotionComponent:setSpeed(speed)
    self.speed = speed
end

function PlanetaryRingMotionComponent:setPhase(phase)
    self.phase = phase
end

function PlanetaryRingMotionComponent:getTime()
    return self.time
end

function PlanetaryRingMotionComponent:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender) --TODO: move to a system
end

---@param data EventData
function PlanetaryRingMotionComponent:onPreRender(data)
    self.time = self.time + data:deltaTime() * self.speed
end

return PlanetaryRingMotionComponent
