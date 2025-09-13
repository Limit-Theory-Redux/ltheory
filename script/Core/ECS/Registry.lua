local ChildrenComponent = require("Modules.Core.Components.ChildrenComponent")
local ParentComponent = require("Modules.Core.Components.ParentComponent")
local NameComponent = require("Modules.Core.Components.NameComponent")

-- Entity and Registry are so tightly coupled that they need to be defined together in the same file.

---@alias EntityId integer

---@class Entity A non-owning handle to an entity in the ECS.
---@field id EntityId
---@overload fun(self: Entity, id: EntityId): Entity class internal
---@overload fun(id: EntityId): Entity class external
local Entity = Class("Entity", function(self, id)
    self.id = id
end)

---@class Registry
---@field entities table<EntityId, table<any, true>>
---@field components table<any, table<EntityId, Component>>

---@class Registry
---@overload fun(self: Registry): Registry class internal
---@overload fun(): Registry class external
local Registry = Class("Registry", function(self)
    self:clear()
end)

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
---@return boolean wasSuccessful
function Registry:destroyEntity(entity)
    local entityComponentIndex = self.entities[entity.id]
    if not entityComponentIndex then
        return false
    end

    -- Remove all components associated with this entity.
    for componentArchetype in pairs(entityComponentIndex) do
        local store = self.components[componentArchetype]
        if store then
            store[entity.id] = nil
        end
    end

    self.entities[entity.id] = nil
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
        -- If the child already has a parent, detach it from the existing parent.
        if self:hasEntity(existingParent) then
            self:detachEntity(existingParent, childEntity)
        else
            Log.Warn("Registry:attachEntity - existing parent entity not found: %s, skipping", tostring(existingParent))
        end
        parentComponent:setParent(parentEntity)
    else
        self:add(childEntity, ParentComponent(parentEntity))
    end
    
    -- Update children.
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

    -- Remove child from parent's ChildrenComponent, and remove the ChildrenComponent if it becomes empty.
    local childrenComponent = self:get(parentEntity, ChildrenComponent)
    if childrenComponent then
        childrenComponent:removeChild(childEntity)
        if #childrenComponent.children == 0 then
            self:remove(parentEntity, ChildrenComponent)
        end
    end

    -- Remove ParentComponent from child.
    self:remove(childEntity, ParentComponent)
    return true
end

---@generic T
---@param entity Entity
---@param component T
---@return T|nil
function Registry:add(entity, component)
    local entityComponentIndex = self.entities[entity.id]
    if not entityComponentIndex then
        return nil
    end

    local archetype = component:getArchetype()

    entityComponentIndex[archetype] = true

    -- Lazily initialize this component's storage.
    if not self.components[archetype] then
        self.components[archetype] = {}
        SetLengthMetamethod(self.components[archetype])
    end
    self.components[archetype][entity.id] = component
    component:setEntityId(entity.id)
    return component
end

---@param entity Entity
---@param componentType any
---@return boolean wasSuccessful
function Registry:remove(entity, componentType)
    if not self.components[componentType] then
        return false
    end

    -- Detach the component from this entity.
    local entityComponentIndex = self.entities[entity.id]
    if not entityComponentIndex or not entityComponentIndex[componentType] then
        return false
    end
    entityComponentIndex[componentType] = nil

    -- Remove this component.
    self.components[componentType][entity.id] = nil
    return true
end

---@generic T
---@param entity Entity
---@param componentType T
---@return T|nil
function Registry:get(entity, componentType)
    local archetypeStorage = self.components[componentType]
    if not archetypeStorage then
        -- No components with this archetype exist.
        return nil
    end

    return archetypeStorage[entity.id]
end

---@param entity Entity
---@param componentType any
---@return boolean
function Registry:has(entity, componentType)
    return self:get(entity, componentType) ~= nil
end

---@param entity Entity
---@return fun(): Component|nil An iterator that yields all components for the given entity
function Registry:iterComponents(entity)
    local entityComponentIndex = self.entities[entity.id]
    if not entityComponentIndex then
        return function() return nil end
    end

    local components = {}
    for componentType in pairs(entityComponentIndex) do
        table.insert(components, self.components[componentType][entity.id])
    end
    return Iterator(components)
end

---@generic T1, T2, T3, T4, T5
---@param ... T1, T2, T3, T4, T5 A variable list of component types
---@return fun(): Entity, T1, T2, T3, T4, T5 An iterator that yield the entity ID and the requested components as a tuple
function Registry:iterEntities(...)
    local componentTypes = { ... } -- Collect the variable arguments into a table
    if #componentTypes == 0 then
        return function() end -- Return an empty iterator if no component types are provided
    end

    -- This method works by taking the first component type, then listing all entities that have that
    -- (by indexing `self.components[primaryComponentType]`), then only yielding for entities that
    -- also include the other components.
    return coroutine.wrap(function()
        local primaryComponentType = componentTypes[1]
        local primaryComponentStorage = self.components[primaryComponentType]

        -- No entities have the primary component type
        if not primaryComponentStorage then
            return 
        end

        for entityId, primaryComponent in pairs(primaryComponentStorage) do
            local components = { primaryComponent }
            local hasAllComponents = true

            for i = 2, #componentTypes do
                local componentType = componentTypes[i]
                local componentStorage = self.components[componentType]

                if not componentStorage or not componentStorage[entityId] then
                    hasAllComponents = false
                    break
                end

                components[i] = componentStorage[entityId]
            end

            if hasAllComponents then
                -- Yield the entity handle and components as a tuple
                coroutine.yield(Entity(entityId), table.unpack(components))
            end
        end
    end)
end

function Registry:getEntityCount()
    return #self.entities
end

function Registry:getComponentCount()
    local count = 0
    for _, archetypeStorage in pairs(self.components) do
        count = count + #archetypeStorage
    end
    return count
end

Registry.Instance = Registry()
Registry.EntityType = Entity

return Registry.Instance
