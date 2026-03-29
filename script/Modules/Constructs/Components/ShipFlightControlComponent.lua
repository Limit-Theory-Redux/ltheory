local Component = require("Core.ECS.Component")

---@class ShipFlightControlComponent: Component
---@overload fun(): ShipFlightControlComponent
local ShipFlightControlComponent = Subclass("ShipFlightControlComponent", Component, function(self)
    self:setComponentName("ShipFlightControl")

    self._targetYaw   = 0
    self._targetPitch = 0
end)

-- GETTERS
function ShipFlightControlComponent:getTargetYaw()   return self._targetYaw end
function ShipFlightControlComponent:getTargetPitch()  return self._targetPitch end

-- SETTERS
function ShipFlightControlComponent:setTargetYaw(v)   self._targetYaw = v end
function ShipFlightControlComponent:setTargetPitch(v)  self._targetPitch = v end

return ShipFlightControlComponent
