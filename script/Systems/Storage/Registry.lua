---@class Registry
---@field entities table<EntityId, Entity>
---@field components table<any, ComponentStorage>

---@alias ComponentStorage table<EntityId, Component>

---@alias EntityId integer

--- Types
local ComponentInfo = require("Shared.Types.ComponentInfo")

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

---@param entity Entity
---@return EntityId
function Registry:storeEntity(entity)
    self.entities[entity:getEntityId()] = entity
    return entity:getEntityId()
end

---@param entityId EntityId
---@return boolean wasSuccessful
function Registry:dropEntity(entityId)
    local entity = self.entities[entityId]
    ---@cast entity Entity

    if entity then
        self.entities[entityId] = nil
        return true
    end
    return false
end

---@param component Component
---@return ComponentInfo
function Registry:storeComponent(component)
    -- Lazily initialize this component's storage.
    local archetype = component:getArchetype()
    if not self.components[archetype] then
        self.components[archetype] = {}
        SetLengthMetamethod(self.components[archetype])
    end
    self.components[archetype][component:getEntityId()] = component
    return ComponentInfo { archetype = archetype, entity = component:getEntityId() }
end

---@param componentInfo ComponentInfo
---@return boolean wasSuccessful
function Registry:dropComponent(componentInfo)
    if not self.components[componentInfo.archetype] then
        return false
    end

    local component = self.components[componentInfo.archetype][componentInfo.entity]
    if not component then
        return false
    end
    
    self.components[componentInfo.archetype][componentInfo.entity] = nil
    return true
end

---@param entityId EntityId
---@return Entity|nil
function Registry:getEntity(entityId)
    return self.entities[entityId]
end

---@param componentInfo ComponentInfo
---@return Component|nil
function Registry:getComponentData(componentInfo)
    local archetypeStorage = self.components[componentInfo.archetype]
    if not archetypeStorage then
        -- No components with this archetype exist.
        return nil
    end

    return archetypeStorage[componentInfo.entity]
end

---@generic T
---@param archetype T
---@return table<EntityId, T>|nil
function Registry:getComponentsFromArchetype(archetype)
    return self.components[archetype]
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

-- if you for some reason want all entities, should only be used for debugging
function Registry:getEntities()
    return self.entities
end

-- if you for some reason want all components, should only be used for debugging
function Registry:getComponents()
    return self.components
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
