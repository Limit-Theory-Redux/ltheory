---@class GlobalStorage
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

---@class GlobalStorage
---@overload fun(self: GlobalStorage): GlobalStorage class internal
---@overload fun(): GlobalStorage class external
local GlobalStorage = Class(function(self)
    -- Ensure initialization only happens once
    if self.initialized then
        Log.Error("You are trying to reinitialize the GlobalStorage, this should not happen.")
        return
    end

    self:initStorage()

    -- Mark as initialized
    self.initialized = true
end)

function GlobalStorage:initStorage()
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

    Log.Info("Initialized GlobalStorage")
end

---@param entity Entity
---@return EntityInfo
function GlobalStorage:storeEntity(entity)
    if not entity:getArchetype() or not self.entities[entity:getArchetype()] then
        Log.Error("Did not provide a valid archetype for entity: " .. tostring(entity:getGuid()))
    end
    self.entities[entity:getArchetype()][entity:getGuid()] = entity
    return EntityInfo { id = entity:getGuid(), archetype = entity:getArchetype() }
end

---@param archetype EntityArchetype
---@param entityId integer
---@return boolean wasSuccessful
function GlobalStorage:dropEntity(archetype, entityId)
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
function GlobalStorage:storeComponent(component)
    if not component:getArchetype() or not self.components[component:getArchetype()] then
        Log.Error("Did not provide a valid archetype for component: " .. tostring(component:getGuid()))
    end
    self.components[component:getArchetype()][component:getGuid()] = component
    return ComponentInfo { id = component:getGuid(), archetype = component:getArchetype(), entity = component:getEntity() }
end

---@param archetype ComponentArchetype
---@param componentId integer
---@return boolean wasSuccessful
function GlobalStorage:dropComponent(archetype, componentId)
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
function GlobalStorage:getEntity(entityInfo)
    ---@type EntityStorage
    local archetypeStorage = self.entities[entityInfo.archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for entity: " .. entityInfo.id)
    end

    return archetypeStorage[entityInfo.id]
end

---@param componentInfo ComponentInfo
---@return Component|nil
function GlobalStorage:getComponentData(componentInfo)
    ---@type ComponentStorage
    local archetypeStorage = self.components[componentInfo.archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for component: " .. componentInfo.id)
    end

    return archetypeStorage[componentInfo.id]
end

---@param archetype EntityArchetype
---@return table<Entity>|nil
function GlobalStorage:getEntitiesFromArchetype(archetype)
    if self.entities[archetype] then
        return self.entities[archetype]
    end
end

---@param archetype ComponentArchetype
---@return table<Component>|nil
function GlobalStorage:getComponentsFromArchetype(archetype)
    if self.components[archetype] then
        return self.components[archetype]
    end
end

-- if you for some reason want all entities, should only be used for debugging
function GlobalStorage:getEntities()
    return self.entities
end

-- if you for some reason want all components, should only be used for debugging
function GlobalStorage:getComponents()
    return self.components
end

function GlobalStorage:getEntityCount()
    local count = 0
    for _, archetype in pairs(self.entities) do
        count = count + #archetype
    end
    return count
end

function GlobalStorage:getComponentCount()
    local count = 0
    for _, archetype in pairs(self.components) do
        count = count + #archetype
    end
    return count
end

function GlobalStorage:clear()
    self:initStorage()
end

return GlobalStorage()
