local Component = require("Core.ECS.Component")

---@class AutoPilotComponent: Component
---@overload fun(): AutoPilotComponent
local AutoPilotComponent = Subclass("AutoPilotComponent", Component, function(self)
    self:setComponentName("AutoPilot")

    self._active       = false
    self._targetEntity = nil
    self._targetPos    = nil     -- Position (f64)
    self._arrivalRange = 500
    self._elapsed      = 0
    self._localOffsetX = nil     -- for tracking targets on orbiting bodies
    self._localOffsetZ = nil
    self._parentEntity = nil

    -- Computed nav data (written by AutoPilotSystem, read by map/HUD)
    self.interceptPos  = nil     -- predicted intercept position
    self.distance      = 0       -- distance to target
    self.eta           = 0       -- estimated time of arrival (seconds)
    self.targetVelX    = 0       -- computed target velocity
    self.targetVelY    = 0
    self.targetVelZ    = 0
    self._prevTargetPos = nil    -- for velocity computation
end)

-- GETTERS
function AutoPilotComponent:isActive()         return self._active end
function AutoPilotComponent:getTargetEntity()  return self._targetEntity end
function AutoPilotComponent:getTargetPos()     return self._targetPos end
function AutoPilotComponent:getArrivalRange()  return self._arrivalRange end
function AutoPilotComponent:getElapsed()       return self._elapsed end

-- SETTERS
function AutoPilotComponent:setActive(v)        self._active = v end
function AutoPilotComponent:setTargetEntity(v)  self._targetEntity = v end
function AutoPilotComponent:setTargetPos(v)     self._targetPos = v end
function AutoPilotComponent:setArrivalRange(v)  self._arrivalRange = v end
function AutoPilotComponent:setElapsed(v)       self._elapsed = v end

--- Get target name for HUD display
---@return string
function AutoPilotComponent:getTargetName()
    if self._targetEntity then
        return tostring(self._targetEntity)
    elseif self._targetPos then
        return "Position"
    end
    return ""
end

return AutoPilotComponent
