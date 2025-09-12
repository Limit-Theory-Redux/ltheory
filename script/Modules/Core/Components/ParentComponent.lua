local Component = require("Core.ECS.Component")

---@class ParentComponent: Component
---@field parent Entity
---@overload fun(self: ParentComponent, parent: Entity) : ParentComponent subclass internal
---@overload fun(parent: Entity): ParentComponent subclass external
local ParentComponent = Subclass("ParentComponent", Component, function(self, parent)
    self:setComponentName("Parent")

    self.parent = parent
end)

---@param parent Entity
function ParentComponent:setParent(parent)
    self.parent = parent
end

---@return Entity
function ParentComponent:getParent()
    return self.parent
end

return ParentComponent
