local Component = require("Core.ECS.Component")

---@class TargetableComponent: Component
---@overload fun(team?: string, sizeClass?: string): TargetableComponent
local TargetableComponent = Subclass("TargetableComponent", Component, function(self, team, sizeClass)
    self:setComponentName("Targetable")
    self.enabled = true
    self.team = team or "neutral"
    self.sizeClass = sizeClass or "small"
end)

function TargetableComponent:isEnabled()
    return self.enabled
end

---@param enabled boolean
function TargetableComponent:setEnabled(enabled)
    self.enabled = enabled == true
end

function TargetableComponent:getTeam()
    return self.team
end

---@param team string
function TargetableComponent:setTeam(team)
    assert(type(team) == "string")
    self.team = team
end

function TargetableComponent:getSizeClass()
    return self.sizeClass
end

---@param sizeClass string
function TargetableComponent:setSizeClass(sizeClass)
    assert(type(sizeClass) == "string" and #sizeClass > 0)
    self.sizeClass = sizeClass
end

return TargetableComponent
