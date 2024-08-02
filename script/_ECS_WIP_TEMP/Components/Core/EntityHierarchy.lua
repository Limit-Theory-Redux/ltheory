local Component = require('Component')
local GlobalStorage = require("Systems.GlobalStorage")

---@class EntityHierarchyComponent: Component
---@overload fun(parentEntity: Entity|nil): EntityHierarchyComponent subclass external
local EntityHierarchyComponent = Subclass(Component, function(self, parentEntity)
    ---@cast self EntityHierarchyComponent
    self:setComponentName("Hierarchy")
    self:addHierarchy(parentEntity)
end)

---@param parentEntityInfo EntityInfo|nil
function EntityHierarchyComponent:addHierarchy(parentEntityInfo)
    if self.hierarchy then
        Log.Warn("This entity already has its own hierarchy, are you sure that you want to reinitialize?")
    end
    self.hierarchy = {
        children = {},
        parent = parentEntityInfo
    }
end

---@param entityInfo EntityInfo
---@return integer childInfoIndex
function EntityHierarchyComponent:addChild(entityInfo)
    insert(self.hierarchy.children, entityInfo)
    return #self.hierarchy.children
end

---@param childInfoIndex integer
function EntityHierarchyComponent:removeChild(childInfoIndex)
    remove(self.hierarchy.children, childInfoIndex)
end

---@param archetype EntityArchetype
---@return table<Component|nil> QueryResults
function EntityHierarchyComponent:findChildrenOfArchetype(archetype)
    local queryResults = {}
    ---@param childEntityInfo EntityInfo
    for _, childEntityInfo in ipairs(self.hierarchy.children) do
        if childEntityInfo.archetype == archetype then
            local component = GlobalStorage:getEntity(childEntityInfo)
            insert(queryResults, component)
        end
    end
    return queryResults
end

---@return Iterator<Entity|nil>
function EntityHierarchyComponent:iterChildren()
    local entities = {}

    ---@param childEntityInfo EntityInfo
    for _, childEntityInfo in ipairs(self.hierarchy.children) do
        local entity = GlobalStorage:getEntity(childEntityInfo)
        insert(entities, entity)
    end
    return Iterator(entities)
end

---@param entityInfo EntityInfo
function EntityHierarchyComponent:setParent(entityInfo)
    self.hierarchy.parent = entityInfo
end

---@return Entity|nil
function EntityHierarchyComponent:getParent()
    return GlobalStorage:getEntity(self.hierarchy.parent)
end

return EntityHierarchyComponent
