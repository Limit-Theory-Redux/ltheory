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

    -- The navigation of one engagement: a `NavWorld` with the ship as its only agent, and the
    -- buffers its results are copied into. All nil while the autopilot is off.
    self._nav          = nil     -- NavWorld
    self._agent        = nil     -- the ship's agent id in `_nav`
    self._axes         = nil     -- double[NavWorld.AxisCount()]
    self._arrived      = nil     -- bool[1]

    -- Computed nav data (written by AutoPilotSystem, read by map/HUD)
    self.interceptPos  = nil     -- predicted intercept position
    self.distance      = 0       -- distance to target
    self.eta           = 0       -- estimated time of arrival (seconds)
    self.targetVelX    = 0       -- the tracked target's velocity
    self.targetVelY    = 0
    self.targetVelZ    = 0
end)

-- GETTERS
function AutoPilotComponent:isActive()         return self._active end
function AutoPilotComponent:getTargetEntity()  return self._targetEntity end
function AutoPilotComponent:getTargetPos()     return self._targetPos end
function AutoPilotComponent:getArrivalRange()  return self._arrivalRange end
function AutoPilotComponent:getElapsed()       return self._elapsed end
function AutoPilotComponent:getNav()           return self._nav end
function AutoPilotComponent:getAgent()         return self._agent end
function AutoPilotComponent:getAxes()          return self._axes end
function AutoPilotComponent:getArrived()       return self._arrived end

-- SETTERS
function AutoPilotComponent:setActive(v)        self._active = v end
function AutoPilotComponent:setTargetEntity(v)  self._targetEntity = v end
function AutoPilotComponent:setTargetPos(v)     self._targetPos = v end
function AutoPilotComponent:setArrivalRange(v)  self._arrivalRange = v end
function AutoPilotComponent:setElapsed(v)       self._elapsed = v end

--- Keep the navigation of an engagement and its result buffers.
---@param nav NavWorld
---@param agent integer
---@param axes ffi.cdata*
---@param arrived ffi.cdata*
function AutoPilotComponent:setNavigation(nav, agent, axes, arrived)
    self._nav, self._agent, self._axes, self._arrived = nav, agent, axes, arrived
end

--- Drop the navigation of the engagement.
function AutoPilotComponent:clearNavigation()
    self._nav, self._agent, self._axes, self._arrived = nil, nil, nil, nil
end

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
