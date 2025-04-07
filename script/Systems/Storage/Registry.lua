---@class Registry
---@field entities table<EntityStorage>
---@field components table<ComponentStorage>

---@class EntityStorage
---@field [EntityArchetype] Entity

---@class ComponentStorage
---@field [EntityArchetype] Component

--- Types
local EntityInfo = require("Shared.Types.EntityInfo")
local ComponentInfo = require("Shared.Types.ComponentInfo")

---@class Registry
---@overload fun(self: Registry): Registry class internal
---@overload fun(): Registry class external
local Registry = Class("Registry", function(self)
    self:clear()
end)

function Registry:clear()
    self.entities = {}
    self.components = {}

    for _, archetype in pairs(Enums.EntityArchetype) do
        self.entities[archetype] = {}
        SetLengthMetamethod(self.entities[archetype])
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
    -- Lazily initialize this component's storage.
    local archetype = component:getArchetype()
    if not self.components[archetype] then
        self.components[archetype] = {}
        SetLengthMetamethod(self.components[archetype])
    end
    self.components[archetype][component:getGuid()] = component
    return ComponentInfo { id = component:getGuid(), archetype = archetype, entity = component:getEntity() }
end

---@param archetype any
---@param componentId integer
---@return boolean wasSuccessful
function Registry:dropComponent(archetype, componentId)
    if not self.components[archetype] then
        return false
    end

    local component = self.components[archetype][componentId]
    ---@cast component Component
    if not component then
        return false
    end
    
    self.components[archetype][componentId] = nil
    return true
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

return Registry()
