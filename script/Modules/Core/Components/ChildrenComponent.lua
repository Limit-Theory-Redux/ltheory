local Component = require("Core.ECS.Component")

---@class ChildrenComponent: Component
---@field children EntityId[]
---@overload fun(self: ChildrenComponent) : ChildrenComponent subclass internal
---@overload fun(): ChildrenComponent subclass external
local ChildrenComponent = Subclass("ChildrenComponent", Component, function(self)
    self:setComponentName("Children")

    self.children = {}
end)

---@param entityId EntityId
function ChildrenComponent:addChild(entityId)
    insert(self.children, entityId)
end

---@param entityId EntityId
function ChildrenComponent:removeChild(entityId)
    for i, v in ipairs(self.children) do
        if v == entityId then
            remove(self.children, i)
            break
        end
    end
end

---@return Iterator<EntityId>
function ChildrenComponent:iterChildren()
    return Iterator(self.children)
end

return ChildrenComponent
