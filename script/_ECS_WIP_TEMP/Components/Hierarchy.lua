local Component = require('Component')

---@class HierarchyComponent: Component
---@overload fun(parentEntity: Entity|nil): HierarchyComponent subclass external
local HierarchyComponent = Subclass(Component, function(self, parentEntity)
    ---@cast self HierarchyComponent
    self:setComponentName("Hierarchy")
    self:addHierarchy(parentEntity)
end)

---@param parentEntity Entity|nil
function HierarchyComponent:addHierarchy(parentEntity)
    if self.hierarchy then
        Log.Warn("This entity already has its own hierarchy, are you sure that you want to reinitialize?")
    end
    self.hierarchy = {
        children = {},
        parent = parentEntity
    }
end

---@param entity Entity
---@return integer childIndex
function HierarchyComponent:addChild(entity)
    insert(self.hierarchy.children, entity)
    return #self.hierarchy.children
end

---@param childIndex integer
function HierarchyComponent:removeChild(childIndex)
    remove(self.hierarchy.children, childIndex)
end

---@param childIndex integer
---@return Entity
function HierarchyComponent:getChild(childIndex)
    return self.hierarchy.children[childIndex]
end

---@return Entity|nil QueryResult
function HierarchyComponent:findChild(query)
    local queryResults = {}
    for index, entity in ipairs(self.hierarchy.children) do
        if string.find(entity:getName(), query) then
            insert(queryResults, index)
        end
    end

    if queryResults > 1 then
        Log.Error("Found more than one entity for your query. Please be more specific.")
    end
    return queryResults[1]
end

---@return Entity|nil QueryResult
function HierarchyComponent:findChildByGuid(guid)
    for index, entity in ipairs(self.hierarchy.children) do
        if entity:getGuid() == guid then
            return entity
        end
    end
end

---@param entityType EntityType
---@return table<Component|nil> QueryResults
function HierarchyComponent:findChildrenOfType(entityType)
    local queryResults = {}
    for index, entity in ipairs(self.hierarchy.children) do
        if entity:getType() == entityType then
            insert(queryResults, index)
        end
    end
    return queryResults
end

---@return IteratorIndexed
function HierarchyComponent:iterChildren()
    return IteratorIndexed(self.hierarchy.children)
end

---@return table<Entity>
function HierarchyComponent:getChildren()
    return self.hierarchy.children
end

---@param entity Entity
function HierarchyComponent:setParent(entity)
    self.hierarchy.parent = entity
end

---@return Entity|nil
function HierarchyComponent:getParent()
    return self.hierarchy.parent
end

return HierarchyComponent
