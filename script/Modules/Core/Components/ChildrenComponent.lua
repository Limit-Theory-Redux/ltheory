local Component = require("Core.ECS.Component")

---@class ChildrenComponent: Component
---@field children Entity[]
---@overload fun(self: ChildrenComponent) : ChildrenComponent subclass internal
---@overload fun(): ChildrenComponent subclass external
local ChildrenComponent = Subclass("ChildrenComponent", Component, function(self)
    self:setComponentName("Children")

    self.children = {}
end)

---@param entity Entity
function ChildrenComponent:addChild(entity)
    insert(self.children, entity)
end

---@param entity Entity
function ChildrenComponent:removeChild(entity)
    for i, v in ipairs(self.children) do
        if v == entity then
            remove(self.children, i)
            break
        end
    end
end

---@return Iterator<Entity>
function ChildrenComponent:iterChildren()
    return Iterator(self.children)
end

return ChildrenComponent
