local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

-- Types
local EntityInfo = require('_ECS_WIP_TEMP.Shared.Types.EntityInfo')

-- Systems
local GlobalStorage = require('_ECS_WIP_TEMP.Systems.Storage.GlobalStorage') --!temp path

---@class EntityHierarchyComponent: Component
---@overload fun(self: EntityHierarchyComponent, parentEntity: EntityInfo|nil) : EntityHierarchyComponent subclass internal
---@overload fun(parentEntity: EntityInfo|nil): EntityHierarchyComponent subclass external
local EntityHierarchyComponent = Subclass(Component, function(self, parentEntityInfo)
    self:setComponentName("EntityHierarchy")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.HierarchyComponent)

    self:addHierarchy(parentEntityInfo)
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
