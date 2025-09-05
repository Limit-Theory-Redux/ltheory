local EntityComponent = require("Components.Core.EntityComponent")
local ChildrenComponent = require("Components.Core.ChildrenComponent")
local ParentComponent = require("Components.Core.ParentComponent")

---@class Registry
---@field entities table<EntityId, table<any, true>>
---@field components table<any, table<EntityId, Component>>

---@alias EntityId integer

---@class Registry
---@overload fun(self: Registry): Registry class internal
---@overload fun(): Registry class external
local Registry = Class("Registry", function(self)
    self:clear()
end)

function Registry:clear()
    self.entities = {}
    SetLengthMetamethod(self.entities)
    self.components = {}
    SetLengthMetamethod(self.components)

    Log.Info("Initialized Registry")
end

---@return EntityId A new unique entity ID
function Registry:createEntity()
    local id = Guid.Create()
    self.entities[id] = {}
    return id
end

---@param entityId EntityId
---@return boolean wasSuccessful
function Registry:destroyEntity(entityId)
    local entityComponentIndex = self.entities[entityId]
    if not entityComponentIndex then
        return false
    end

    -- Remove all components associated with this entity.
    for componentArchetype in pairs(entityComponentIndex) do
        local store = self.components[componentArchetype]
        if store then
            store[entityId] = nil
        end
    end

    self.entities[entityId] = nil
    return true
end

---@param entityId EntityId
---@return boolean
function Registry:hasEntity(entityId)
    return self.entities[entityId] ~= nil
end 

---@param entityId EntityId
---@return EntityId The cloned entity ID
function Registry:cloneEntity(entityId)
    local clonedEntity = self:createEntity()
    for component in self:iterComponents(entityId) do
        ---@type Component
        local clonedComponent = DeepClone(component)
        clonedComponent:addGuid()
        self:add(clonedEntity, clonedComponent)
    end

    return clonedEntity
end

---@param parentEntityId EntityId
---@param childEntityId EntityId
---@return boolean if successful
function Registry:attachEntity(parentEntityId, childEntityId)
    if not self:hasEntity(parentEntityId) then
        Log.Warn("Registry:attachEntity - parent entity not found: %s", tostring(parentEntityId))
        return false
    end
    if not self:hasEntity(childEntityId) then
        Log.Warn("Registry:attachEntity - child entity not found: %s", tostring(childEntityId))
        return false
    end

    -- Ensure that the parent process is set correctly.
    local parentComponent = self:get(childEntityId, ParentComponent)
    if parentComponent then
        local existingParent = parentComponent:getParent()
        -- If the child already has a parent, detach it from the existing parent.
        if self:hasEntity(existingParent) then
            self:detachEntity(existingParent, childEntityId)
        else
            Log.Warn("Registry:attachEntity - existing parent entity not found: %s, skipping", tostring(existingParent))
        end
        parentComponent:setParent(parentEntityId)
    else
        self:add(childEntityId, ParentComponent(parentEntityId))
    end
    
    -- Update children.
    local childrenComponent = self:get(parentEntityId, ChildrenComponent)
    if not childrenComponent then
        childrenComponent = self:add(parentEntityId, ChildrenComponent())
    end
    childrenComponent:addChild(childEntityId)
    return true
end

---@param parentEntityId EntityId
---@param childEntityId EntityId
---@return boolean if successful
function Registry:detachEntity(parentEntityId, childEntityId)
    if not self:hasEntity(parentEntityId) then
        Log.Warn("Registry:detachEntity - parent entity not found: %s", tostring(parentEntityId))
        return false
    end
    if not self:hasEntity(childEntityId) then
        Log.Warn("Registry:detachEntity - child entity not found: %s", tostring(childEntityId))
        return false
    end

    -- Remove child from parent's ChildrenComponent, and remove the ChildrenComponent if it becomes empty.
    local childrenComponent = self:get(parentEntityId, ChildrenComponent)
    if childrenComponent then
        childrenComponent:removeChild(childEntityId)
        if #childrenComponent.children == 0 then
            self:remove(parentEntityId, ChildrenComponent)
        end
    end

    -- Remove ParentComponent from child.
    self:remove(childEntityId, ParentComponent)
    return true
end

---@return Entity|nil
function Registry:getEntity(entityId)
    local entityComponent = self:get(entityId, EntityComponent)
    if entityComponent then
        return entityComponent.entity
    end
    return nil
end

---@generic T
---@param entityId EntityId
---@param component T
---@return T|nil
function Registry:add(entityId, component)
    local entityComponentIndex = self.entities[entityId]
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
    self.components[archetype][entityId] = component
    component:setEntityId(entityId)
    return component
end

---@param entityId EntityId
---@param componentType any
---@return boolean wasSuccessful
function Registry:remove(entityId, componentType)
    if not self.components[componentType] then
        return false
    end

    -- Detach the component from this entity.
    local entityComponentIndex = self.entities[entityId]
    if not entityComponentIndex or not entityComponentIndex[componentType] then
        return false
    end
    entityComponentIndex[componentType] = nil

    -- Remove this component.
    self.components[componentType][entityId] = nil
    return true
end

---@generic T
---@param entityId EntityId
---@param componentType T
---@return T|nil
function Registry:get(entityId, componentType)
    local archetypeStorage = self.components[componentType]
    if not archetypeStorage then
        -- No components with this archetype exist.
        return nil
    end

    return archetypeStorage[entityId]
end

---@param entityId EntityId
---@param componentType any
---@return boolean
function Registry:has(entityId, componentType)
    return Registry:get(entityId, componentType) ~= nil
end

---@param entityId EntityId
---@return fun(): Component|nil An iterator that yields all components for the given entity ID 
function Registry:iterComponents(entityId)
    local entityComponentIndex = self.entities[entityId]
    if not entityComponentIndex then
        return function() return nil end
    end

    local components = {}
    for componentType in pairs(entityComponentIndex) do
        table.insert(components, self.components[componentType][entityId])
    end
    return Iterator(components)
end

---@generic T1, T2, T3, T4, T5
---@param ... T1, T2, T3, T4, T5 A variable list of component types
---@return fun(): EntityId, T1, T2, T3, T4, T5 An iterator that yield the entity ID and the requested components as a tuple
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
                -- Yield the entity ID and components as a tuple
                coroutine.yield(entityId, table.unpack(components))
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

return Registry()
