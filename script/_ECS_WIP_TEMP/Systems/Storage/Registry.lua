---@class Registry
---@field entities table<EntityStorage>
---@field components table<ComponentStorage>
---@field initialized boolean

---@class EntityStorage
---@field [EntityArchetype] Entity

---@class ComponentStorage
---@field [EntityArchetype] Component

--- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")
local ComponentInfo = require("_ECS_WIP_TEMP.Shared.Types.ComponentInfo")

---@class Registry
---@overload fun(self: Registry): Registry class internal
---@overload fun(): Registry class external
local Registry = Class(function(self)
    -- Ensure initialization only happens once
    if self.initialized then
        Log.Error("You are trying to reinitialize the Registry, this should not happen.")
        return
    end

    self:initStorage()

    -- Mark as initialized
    self.initialized = true
end)

function Registry:initStorage()
    self.entities = {}
    self.components = {}

    for _, archetype in pairs(Enums.EntityArchetype) do
        self.entities[archetype] = {}
        SetLengthMetamethod(self.entities[archetype])
    end

    for _, archetype in pairs(Enums.ComponentArchetype) do
        self.components[archetype] = {}
        SetLengthMetamethod(self.components[archetype])
    end

    Log.Info("Initialized Registry")
end

---@param entity Entity
---@return EntityInfo
function Registry:storeEntity(entity)
    if not entity:getArchetype() or not self.entities[entity:getArchetype()] then
        Log.Error("Did not provide a valid archetype for entity: " .. tostring(entity:getGuid()))
    end
    self.entities[entity:getArchetype()][entity:getGuid()] = entity
    return EntityInfo { id = entity:getGuid(), archetype = entity:getArchetype() }
end

---@param archetype EntityArchetype
---@param entityId integer
---@return boolean wasSuccessful
function Registry:dropEntity(archetype, entityId)
    local entity = self.entities[archetype][entityId]
    ---@cast entity Entity

    if entity then
        self.entities[archetype][entityId] = nil
        return true
    end
    return false
end

---@param component Component
---@return ComponentInfo
function Registry:storeComponent(component)
    if not component:getArchetype() or not self.components[component:getArchetype()] then
        Log.Error("Did not provide a valid archetype for component: " .. tostring(component:getGuid()))
    end
    self.components[component:getArchetype()][component:getGuid()] = component
    return ComponentInfo { id = component:getGuid(), archetype = component:getArchetype(), entity = component:getEntity() }
end

---@param archetype ComponentArchetype
---@param componentId integer
---@return boolean wasSuccessful
function Registry:dropComponent(archetype, componentId)
    local component = self.components[archetype][componentId]
    ---@cast component Component

    if component then
        self.components[archetype][componentId] = nil
        return true
    end
    return false
end

---@param entityInfo EntityInfo
---@return Entity|nil
function Registry:getEntity(entityInfo)
    ---@type EntityStorage
    local archetypeStorage = self.entities[entityInfo.archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for entity: " .. entityInfo.id)
    end

    return archetypeStorage[entityInfo.id]
end

---@param componentInfo ComponentInfo
---@return Component|nil
function Registry:getComponentData(componentInfo)
    ---@type ComponentStorage
    local archetypeStorage = self.components[componentInfo.archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for component: " .. componentInfo.id)
    end

    return archetypeStorage[componentInfo.id]
end

---@param archetype EntityArchetype
---@return table<Entity>|nil
function Registry:getEntitiesFromArchetype(archetype)
    if self.entities[archetype] then
        return self.entities[archetype]
    end
end

---@param archetype ComponentArchetype
---@return table<Component>|nil
function Registry:getComponentsFromArchetype(archetype)
    if self.components[archetype] then
        return self.components[archetype]
    end
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
    local count = 0
    for _, archetype in pairs(self.entities) do
        count = count + #archetype
    end
    return count
end

function Registry:getComponentCount()
    local count = 0
    for _, archetype in pairs(self.components) do
        count = count + #archetype
    end
    return count
end

function Registry:clear()
    self:initStorage()
end

return Registry()
