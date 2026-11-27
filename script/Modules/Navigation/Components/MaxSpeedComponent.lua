local Component = require("Core.ECS.Component")

---@class MaxSpeedComponent: Component
---@overload fun(self: MaxSpeedComponent, speed: number): MaxSpeedComponent subclass internal
---@overload fun(speed: number): MaxSpeedComponent subclass external
--- The highest speed a ship's orders fly it at, in game units per second, whatever its flight
--- profile allows (a formation's leader holding its wing's pace). `Nav.setMaxSpeed` sets it.
local MaxSpeedComponent = Subclass("MaxSpeedComponent", Component, function(self, speed)
    self:setComponentName("MaxSpeed")
    self:setSpeed(speed)
end)

---@param speed number Greater than zero
function MaxSpeedComponent:setSpeed(speed)
    assert(type(speed) == "number" and speed > 0, "MaxSpeed: the speed must be a number greater than zero")
    self._speed = speed
end

---@return number
function MaxSpeedComponent:getSpeed()
    return self._speed
end

return MaxSpeedComponent
