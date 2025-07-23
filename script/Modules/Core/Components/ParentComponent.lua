local Component = require("Core.ECS.Component")

---@class ParentComponent: Component
---@field parent EntityId
---@overload fun(self: ParentComponent, parent: EntityId) : ParentComponent subclass internal
---@overload fun(parent: EntityId): ParentComponent subclass external
local ParentComponent = Subclass("ParentComponent", Component, function(self, parent)
    self:setComponentName("Parent")

    self.parent = parent
end)

---@param parent EntityId
function ParentComponent:setParent(parent)
    self.parent = parent
end

---@return EntityId
function ParentComponent:getParent()
    return self.parent
end

return ParentComponent
