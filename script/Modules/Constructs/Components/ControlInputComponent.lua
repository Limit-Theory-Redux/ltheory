local Component = require("Core.ECS.Component")

---@alias ControlAxis number A normalised control value in [-1, 1]

--- What a ship is asked to do this frame. Writers (the player controller, AI) set it;
--- actuation systems read it. Axes are levels the writer sets every frame; the drive
--- request is an edge that the travel drive system consumes.
---@class ControlInputComponent: Component
---@overload fun(): ControlInputComponent
local ControlInputComponent = Subclass("ControlInputComponent", Component, function(self)
    self:setComponentName("ControlInput")

    self._thrustForward = 0
    self._thrustRight   = 0
    self._thrustUp      = 0
    self._pitch         = 0
    self._yaw           = 0
    self._roll          = 0
    self._boost         = 0 -- [0, 1]

    self._driveRequest  = Enums.TravelDriveRequest.None
    self._driveSpeedCap = nil -- speed multiplier >= 1 the drive may reach, nil for no cap
end)

-- GETTERS
---@return ControlAxis
function ControlInputComponent:getThrustForward() return self._thrustForward end
---@return ControlAxis
function ControlInputComponent:getThrustRight() return self._thrustRight end
---@return ControlAxis
function ControlInputComponent:getThrustUp() return self._thrustUp end
---@return ControlAxis
function ControlInputComponent:getPitch() return self._pitch end
---@return ControlAxis
function ControlInputComponent:getYaw() return self._yaw end
---@return ControlAxis
function ControlInputComponent:getRoll() return self._roll end
---@return number
function ControlInputComponent:getBoost() return self._boost end
---@return integer
function ControlInputComponent:getDriveRequest() return self._driveRequest end
---@return number|nil
function ControlInputComponent:getDriveSpeedCap() return self._driveSpeedCap end

-- SETTERS
---@param v ControlAxis
function ControlInputComponent:setThrustForward(v) self._thrustForward = v end
---@param v ControlAxis
function ControlInputComponent:setThrustRight(v) self._thrustRight = v end
---@param v ControlAxis
function ControlInputComponent:setThrustUp(v) self._thrustUp = v end
---@param v ControlAxis
function ControlInputComponent:setPitch(v) self._pitch = v end
---@param v ControlAxis
function ControlInputComponent:setYaw(v) self._yaw = v end
---@param v ControlAxis
function ControlInputComponent:setRoll(v) self._roll = v end
---@param v number
function ControlInputComponent:setBoost(v) self._boost = v end
---@param v integer Enums.TravelDriveRequest
function ControlInputComponent:setDriveRequest(v) self._driveRequest = v end
---@param v number|nil
function ControlInputComponent:setDriveSpeedCap(v) self._driveSpeedCap = v end

--- Zero every axis; the drive request and cap are left alone.
function ControlInputComponent:clearAxes()
    self._thrustForward = 0
    self._thrustRight   = 0
    self._thrustUp      = 0
    self._pitch         = 0
    self._yaw           = 0
    self._roll          = 0
    self._boost         = 0
end

return ControlInputComponent
