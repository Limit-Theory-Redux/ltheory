local Component = require('Components.Component')

-- Systems
local Registry = require('Systems.Storage.Registry')

---@class HierarchyComponent: Component
---@overload fun(self: HierarchyComponent, parentEntity: EntityId|nil) : HierarchyComponent subclass internal
---@overload fun(parentEntity: EntityId|nil): HierarchyComponent subclass external
local HierarchyComponent = Subclass("HierarchyComponent", Component, function(self, parentEntityId)
    self:setComponentName("EntityHierarchy")

    self:addHierarchy(parentEntityId)
end)

---@param parentEntityId EntityId|nil
function HierarchyComponent:addHierarchy(parentEntityId)
    if self.hierarchy then
        Log.Warn("This entity already has its own hierarchy, are you sure that you want to reinitialize?")
    end
    self.hierarchy = {
        children = {},
        parent = parentEntityId
    }
end

---@param entityId EntityId
---@return integer childIndex
function HierarchyComponent:addChild(entityId)
    insert(self.hierarchy.children, entityId)
    return #self.hierarchy.children
end

---@param childIndex integer
function HierarchyComponent:removeChild(childIndex)
    remove(self.hierarchy.children, childIndex)
end

---@return Iterator<Entity|nil>
function HierarchyComponent:iterChildren()
    local entities = {}

    ---@param childEntityId EntityId
    for _, childEntityId in ipairs(self.hierarchy.children) do
        local entity = Registry:getEntity(childEntityId)
        insert(entities, entity)
    end
    return Iterator(entities)
end

---@param entityId EntityId
function HierarchyComponent:setParent(entityId)
    self.hierarchy.parent = entityId
end

---@return Entity|nil
function HierarchyComponent:getParent()
    return Registry:getEntity(self.hierarchy.parent)
end

return HierarchyComponent
