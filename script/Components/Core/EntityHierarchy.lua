local Component = require('Components.Component')

-- Systems
local Registry = require('Systems.Storage.Registry')

---@class EntityHierarchyComponent: Component
---@overload fun(self: EntityHierarchyComponent, parentEntity: EntityId|nil) : EntityHierarchyComponent subclass internal
---@overload fun(parentEntity: EntityId|nil): EntityHierarchyComponent subclass external
local EntityHierarchyComponent = Subclass("EntityHierarchyComponent", Component, function(self, parentEntityId)
    self:setComponentName("EntityHierarchy")

    self:addHierarchy(parentEntityId)
end)

---@param parentEntityId EntityId|nil
function EntityHierarchyComponent:addHierarchy(parentEntityId)
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
function EntityHierarchyComponent:addChild(entityId)
    insert(self.hierarchy.children, entityId)
    return #self.hierarchy.children
end

---@param childIndex integer
function EntityHierarchyComponent:removeChild(childIndex)
    remove(self.hierarchy.children, childIndex)
end

---@param archetype EntityArchetype
---@return table<Entity|nil> QueryResults
function EntityHierarchyComponent:findChildrenOfArchetype(archetype)
    local queryResults = {}
    ---@param childEntityId EntityId
    for _, childEntityId in ipairs(self.hierarchy.children) do
        if Registry:getEntity(childEntityId):getArchetype() == archetype then
            local entity = Registry:getEntity(childEntityId)
            insert(queryResults, entity)
        end
    end
    return queryResults
end

---@return Iterator<Entity|nil>
function EntityHierarchyComponent:iterChildren()
    local entities = {}

    ---@param childEntityId EntityId
    for _, childEntityId in ipairs(self.hierarchy.children) do
        local entity = Registry:getEntity(childEntityId)
        insert(entities, entity)
    end
    return Iterator(entities)
end

---@param entityId EntityId
function EntityHierarchyComponent:setParent(entityId)
    self.hierarchy.parent = entityId
end

---@return Entity|nil
function EntityHierarchyComponent:getParent()
    return Registry:getEntity(self.hierarchy.parent)
end

return EntityHierarchyComponent
