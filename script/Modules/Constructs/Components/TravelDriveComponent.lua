local Component = require("Core.ECS.Component")

---@class TravelDriveComponent: Component
---@overload fun(): TravelDriveComponent
local TravelDriveComponent = Subclass("TravelDriveComponent", Component, function(self)
    self:setComponentName("TravelDrive")

    self._state      = "idle"  -- idle, charging, active, decelerating
    self._chargeTime  = 0
    self._currentMult = 1.0
    self._activeTime  = 0
end)

-- GETTERS
function TravelDriveComponent:getState()       return self._state end
function TravelDriveComponent:getChargeTime()   return self._chargeTime end
function TravelDriveComponent:getCurrentMult()  return self._currentMult end
function TravelDriveComponent:getActiveTime()   return self._activeTime end

-- SETTERS
function TravelDriveComponent:setState(v)       self._state = v end
function TravelDriveComponent:setChargeTime(v)  self._chargeTime = v end
function TravelDriveComponent:setCurrentMult(v) self._currentMult = v end
function TravelDriveComponent:setActiveTime(v)  self._activeTime = v end

--- Get charge progress (0-1)
---@return number
function TravelDriveComponent:getChargeProgress()
    local cfg = Config.game.travelDrive
    if self._state == "charging" then
        return self._chargeTime / cfg.chargeRequired
    elseif self._state == "active" then
        return 1.0
    end
    return 0.0
end

--- Is the drive currently active (providing speed boost)?
---@return boolean
function TravelDriveComponent:isActive()
    return self._state == "active"
end

return TravelDriveComponent
