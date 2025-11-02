local Component = require("Core.ECS.Component")

---@class CloudMotionComponent: Component
---@overload fun(self: CloudMotionComponent, speed: number, phase: number): CloudMotionComponent subclass internal
---@overload fun(speed: number, phase: number): CloudMotionComponent subclass external
local CloudMotionComponent = Subclass("CloudMotionComponent", Component, function(self, speed, phase)
    self:setComponentName("CelestialCloudMotionComponent")

    self.time = 0
    self:setSpeed(speed)
    self:setPhase(phase)
    self:registerEvents()
end)

function CloudMotionComponent:setSpeed(speed)
    self.speed = speed
end

function CloudMotionComponent:setPhase(phase)
    self.phase = phase
end

function CloudMotionComponent:getTime()
    return self.time
end

function CloudMotionComponent:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender) --TODO: move to a system
end

---@param data EventData
function CloudMotionComponent:onPreRender(data)
    self.time = self.time + data:deltaTime() * self.speed
end

return CloudMotionComponent
