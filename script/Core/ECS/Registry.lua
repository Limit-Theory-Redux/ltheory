local ChildrenComponent = require("Modules.Core.Components.ChildrenComponent")
local ParentComponent   = require("Modules.Core.Components.ParentComponent")
local NameComponent     = require("Modules.Core.Components.NameComponent")

-- Entity and Registry are so tightly coupled that they need to be defined together in the same file.

---@alias EntityId integer

---@class Entity A non-owning handle to an entity in the ECS.
---@field id EntityId
---@overload fun(self: Entity, id: EntityId): Entity class internal
---@overload fun(id: EntityId): Entity class external
local Entity            = Class("Entity", function(self, id)
    self.id = id
end)

---@class ComponentStorage
---@field sparse table<EntityId, Component>
---@field dense  Component[]
---@field entityMap table<EntityId, integer>  -- entityId -> dense index

---@class Registry
---@field entities table<EntityId, table<any, true>>
---@field components table<any, ComponentStorage>

---@class Registry
---@overload fun(self: Registry): Registry class internal
---@overload fun(): Registry class external
local Registry          = Class("Registry", function(self)
    self:clear()
end)

Registry.DESTROY_MODE   = {
    KEEP_CHILDREN    = 0, -- children stay alive, parent link removed
    DESTROY_CHILDREN = 1, -- recursively destroy the whole subtree
}

function Entity:__tostring()
    local nameComponent = self:get(NameComponent)
    if not nameComponent then
        return format("unnamed(%d)", self.id)
    end
    return format("%s(%d)", self:get(NameComponent):getName(), self.id)
end

---@generic T
---@param component T
---@return T|nil
function Entity:add(component)
    return Registry.Instance:add(self, component)
end

---@generic T
---@param componentType T
---@return T|nil
function Entity:get(componentType)
    if not self:isValid() then
        return nil
    end
    return Registry.Instance:get(self, componentType)
end

---@generic T
---@param componentType T
---@return T|nil
function Entity:remove(componentType)
    return Registry.Instance:remove(self, componentType)
end

-- This function constructs a new entity with the specified name and list of components.
---@param name string
---@param ... any
---@return Entity
function Entity.Create(name, ...)
    local entity = Registry.Instance:createEntity()
    entity:add(NameComponent(name or "Entity"))
    for _, component in ipairs({ ... }) do
        entity:add(component)
    end
    return entity
end

function Entity:isValid()
    return Registry.Instance:hasEntity(self)
end

function Registry:clear()
    self.entities = {}
    SetLengthMetamethod(self.entities)
    self.components = {}
    SetLengthMetamethod(self.components)

    Log.Info("Initialized Registry")
end

---@return Entity A new unique entity
function Registry:createEntity()
    local id = Guid.Create()
    self.entities[id] = {}
    return Entity(id)
end

---@param entity Entity
---@param mode? EntityDestroyMode
---@return boolean wasSuccessful
function Registry:destroyEntity(entity, mode)
    mode = mode or self.DESTROY_MODE.KEEP_CHILDREN

    local entityId = entity.id
    local entityComponentIndex = self.entities[entityId]
    if not entityComponentIndex then
        return false
    end

    local childrenComp = self:get(entity, ChildrenComponent)
    if childrenComp then
        local kids = { table.unpack(childrenComp.children) } -- copy list
        for _, child in ipairs(kids) do
            if mode == self.DESTROY_MODE.DESTROY_CHILDREN then
                self:destroyEntity(child, mode) -- cascade
            else
                -- just orphan the child
                self:remove(child, ParentComponent)
            end
        end
        -- clean up the now-empty ChildrenComponent
        self:remove(entity, ChildrenComponent)
    end

    for componentArchetype in pairs(entityComponentIndex) do
        local store = self.components[componentArchetype]
        if store then
            local sparse    = store.sparse
            local dense     = store.dense
            local entityMap = store.entityMap

            -- dense swap-remove
            local idx       = entityMap and entityMap[entityId]
            if idx then
                local lastComp    = dense[#dense]
                local lastId      = lastComp:getEntityId()
                dense[idx]        = lastComp
                entityMap[lastId] = idx
                table.remove(dense)
                entityMap[entityId] = nil
            end
            sparse[entityId] = nil
        end
    end

    self.entities[entityId] = nil
    return true
end

---@param entity Entity
---@return boolean
function Registry:hasEntity(entity)
    return self.entities[entity.id] ~= nil
end

---@param entity Entity
---@return Entity The cloned entity
function Registry:cloneEntity(entity)
    local clonedEntity = self:createEntity()
    for component in self:iterComponents(entity) do
        ---@type Component
        local clonedComponent = DeepClone(component)
        clonedComponent:addGuid()
        self:add(clonedEntity, clonedComponent)
    end

    return clonedEntity
end

---@param parentEntity Entity
---@param childEntity Entity
---@return boolean if successful
function Registry:attachEntity(parentEntity, childEntity)
    if not self:hasEntity(parentEntity) then
        Log.Warn("Registry:attachEntity - parent entity not found: %s", tostring(parentEntity))
        return false
    end
    if not self:hasEntity(childEntity) then
        Log.Warn("Registry:attachEntity - child entity not found: %s", tostring(childEntity))
        return false
    end

    -- Ensure that the parent process is set correctly.
    local parentComponent = self:get(childEntity, ParentComponent)
    if parentComponent then
        local existingParent = parentComponent:getParent()
        if self:hasEntity(existingParent) then
            self:detachEntity(existingParent, childEntity)
        else
            Log.Warn("Registry:attachEntity - existing parent entity not found: %s, skipping", tostring(existingParent))
        end
        parentComponent:setParent(parentEntity)
    else
        self:add(childEntity, ParentComponent(parentEntity))
    end

    local childrenComponent = self:get(parentEntity, ChildrenComponent)
    if not childrenComponent then
        childrenComponent = self:add(parentEntity, ChildrenComponent())
    end
    childrenComponent:addChild(childEntity)
    return true
end

---@param parentEntity Entity
---@param childEntity Entity
---@return boolean if successful
function Registry:detachEntity(parentEntity, childEntity)
    if not self:hasEntity(parentEntity) then
        Log.Warn("Registry:detachEntity - parent entity not found: %s", tostring(parentEntity))
        return false
    end
    if not self:hasEntity(childEntity) then
        Log.Warn("Registry:detachEntity - child entity not found: %s", tostring(childEntity))
        return false
    end

    local childrenComponent = self:get(parentEntity, ChildrenComponent)
    if childrenComponent then
        childrenComponent:removeChild(childEntity)
        if #childrenComponent.children == 0 then
            self:remove(parentEntity, ChildrenComponent)
        end
    end

    self:remove(childEntity, ParentComponent)
    return true
end

local function ensureDenseStorage(self, componentType)
    local storage = self.components[componentType]
    if not storage then
        storage = { sparse = {}, dense = {}, entityMap = {} }
        self.components[componentType] = storage
        SetLengthMetamethod(storage.sparse)
        SetLengthMetamethod(storage.dense)
        SetLengthMetamethod(storage.entityMap)
    elseif not storage.dense then
        -- Migrate existing sparse → dense
        storage.dense = {}
        storage.entityMap = {}
        local index = 1
        for entityId, comp in pairs(storage.sparse) do
            storage.dense[index] = comp
            storage.entityMap[entityId] = index
            index = index + 1
        end
    end
    return storage
end

---@generic T
---@param entity Entity
---@param component T
---@return T|nil
function Registry:add(entity, component)
    local entityId = entity.id
    local archetype = component:getArchetype()

    local entityComponentIndex = self.entities[entityId]
    if not entityComponentIndex then return nil end
    entityComponentIndex[archetype] = true

    local storage = ensureDenseStorage(self, archetype)
    local sparse = storage.sparse
    local dense = storage.dense
    local entityMap = storage.entityMap

    if not sparse[entityId] then
        table.insert(dense, component)
        entityMap[entityId] = #dense
    end

    sparse[entityId] = component
    component:setEntityId(entityId)
    return component
end

---@param entity Entity
---@param componentType any
---@return boolean wasSuccessful
function Registry:remove(entity, componentType)
    local entityId = entity.id
    local storage = self.components[componentType]
    if not storage or not storage.sparse[entityId] then
        return false
    end

    local sparse = storage.sparse
    local dense = storage.dense
    local entityMap = storage.entityMap

    -- Remove from dense array (swap-remove)
    local index = entityMap[entityId]
    if index then
        local lastComp = dense[#dense]
        local lastEntityId = lastComp:getEntityId()
        dense[index] = lastComp
        entityMap[lastEntityId] = index
        table.remove(dense)
        entityMap[entityId] = nil
    end

    sparse[entityId] = nil

    local entityComponentIndex = self.entities[entityId]
    if entityComponentIndex then
        entityComponentIndex[componentType] = nil
    end

    return true
end

---@generic T
---@param entity Entity
---@param componentType T
---@return T|nil
function Registry:get(entity, componentType)
    local storage = self.components[componentType]
    if not storage then return nil end
    return storage.sparse[entity.id]
end

---@param entity Entity
---@param componentType any
---@return boolean
function Registry:has(entity, componentType)
    local storage = self.components[componentType]
    return storage and storage.sparse[entity.id] ~= nil
end

---@param entity Entity
---@return fun(): Component|nil
function Registry:iterComponents(entity)
    local entityComponentIndex = self.entities[entity.id]
    if not entityComponentIndex then
        return function() return nil end
    end

    local components = {}
    for componentType in pairs(entityComponentIndex) do
        local comp = self.components[componentType]
        if comp then
            table.insert(components, comp.sparse[entity.id])
        end
    end
    return Iterator(components)
end

---@generic T1, T2, T3, T4, T5
---@param ... T1, T2, T3, T4, T5
---@return fun(): Entity, T1, T2, T3, T4, T5
function Registry:iterEntities(...)
    local componentTypes = { ... }
    if #componentTypes == 0 then
        return function() end
    end

    return coroutine.wrap(function()
        local primaryComponentType = componentTypes[1]
        local primaryStorage = self.components[primaryComponentType]
        if not primaryStorage then return end

        for entityId, primaryComponent in pairs(primaryStorage.sparse) do
            local components = { primaryComponent }
            local hasAll = true

            for i = 2, #componentTypes do
                local ct = componentTypes[i]
                local store = self.components[ct]
                if not store or not store.sparse[entityId] then
                    hasAll = false
                    break
                end
                components[i] = store.sparse[entityId]
            end

            if hasAll then
                coroutine.yield(Entity(entityId), table.unpack(components))
            end
        end
    end)
end

---@generic T
---@param componentType T
---@return fun(): Entity, T
function Registry:view(componentType)
    local storage = self.components[componentType]
    if not storage or not storage.dense then
        return function() end
    end

    local dense = storage.dense
    local i = 0
    local n = #dense

    return function()
        i = i + 1
        if i <= n then
            local comp = dense[i]
            return Entity(comp:getEntityId()), comp
        end
    end
end

-- Alias
Registry.iterComponentsByType = Registry.view

---@generic T
---@param componentType T
---@param callback fun(entity: Entity, component: T)
function Registry:each(componentType, callback)
    for entity, comp in self:view(componentType) do
        callback(entity, comp)
    end
end

function Registry:getEntityCount()
    return #self.entities
end

function Registry:getComponentCount()
    local count = 0
    for _, store in pairs(self.components) do
        count = count + #store.dense
    end
    return count
end

function Registry:printHierarchy(entity)
    local function printEntity(ent, prefix, isLast)
        local linePrefix = prefix .. (isLast and "└── " or "├── ")
        Log.Info("%s%s", linePrefix, tostring(ent))

        local childrenComponent = self:get(ent, ChildrenComponent)
        if childrenComponent then
            local children = childrenComponent.children
            for i, childEntity in ipairs(children) do
                local newPrefix = prefix .. (isLast and "    " or "│   ")
                printEntity(childEntity, newPrefix, i == #children)
            end
        end
    end

    Log.Info("Entity Hierarchy for %s:", tostring(entity))
    printEntity(entity, "", true)
end

Registry.Instance = Registry()
Registry.EntityType = Entity

return Registry.Instance
