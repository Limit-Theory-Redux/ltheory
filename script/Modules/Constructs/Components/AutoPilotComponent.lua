local Component = require("Core.ECS.Component")

---@class AutoPilotComponent: Component
---@overload fun(): AutoPilotComponent
--- What the player's autopilot flies: the MoveTo order it issued to the ship, and the entity it
--- was given as the target (for its name). Both are nil while the autopilot is off. A cancelled
--- order stays in `ending` until the order system has ended it, so that it can be released.
local AutoPilotComponent = Subclass("AutoPilotComponent", Component, function(self)
    self:setComponentName("AutoPilot")

    self._order        = nil     -- Entity: the MoveTo order
    self._targetEntity = nil     -- Entity: the target of the order, nil for a point
    self._ending       = {}      -- Entity[]: cancelled orders that have not ended yet
end)

---@return boolean
function AutoPilotComponent:isActive()         return self._order ~= nil end
---@return Entity|nil
function AutoPilotComponent:getOrder()         return self._order end
---@return Entity|nil
function AutoPilotComponent:getTargetEntity()  return self._targetEntity end
---@return Entity[]
function AutoPilotComponent:getEnding()        return self._ending end

--- Keep the order of an engagement and its target.
---@param order Entity
---@param targetEntity Entity|nil Nil when the target is a point
function AutoPilotComponent:setEngagement(order, targetEntity)
    self._order, self._targetEntity = order, targetEntity
end

--- Let go of the engagement, and wait for its order to end.
function AutoPilotComponent:endEngagement()
    if self._order then self._ending[#self._ending + 1] = self._order end
    self._order, self._targetEntity = nil, nil
end

--- Get target name for HUD display
---@return string
function AutoPilotComponent:getTargetName()
    if self._targetEntity then
        return tostring(self._targetEntity)
    elseif self._order then
        return "Position"
    end
    return ""
end

return AutoPilotComponent
