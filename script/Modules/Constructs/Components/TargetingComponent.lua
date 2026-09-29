local Component = require("Core.ECS.Component")

---@class TargetingComponent: Component
---@overload fun(range?: number): TargetingComponent
local TargetingComponent = Subclass("TargetingComponent", Component, function(self, range)
    self:setComponentName("Targeting")
    self.target = nil
    self.range = range or math.huge
    self.autoAcquire = false
end)

function TargetingComponent:getTarget()
    return self.target
end

---@param target Entity|nil
function TargetingComponent:setTarget(target)
    self.target = target
end

function TargetingComponent:getRange()
    return self.range
end

---@param range number
function TargetingComponent:setRange(range)
    assert(range >= 0)
    self.range = range
end

function TargetingComponent:isAutoAcquireEnabled()
    return self.autoAcquire
end

---@param enabled boolean
function TargetingComponent:setAutoAcquireEnabled(enabled)
    self.autoAcquire = enabled == true
end

return TargetingComponent
