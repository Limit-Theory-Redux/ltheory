---@class GlobalStorage
---@field entities table<EntityArchetypeStorage>
---@field components table<ComponentArchetypeStorage>
---@field initialized boolean
local GlobalStorage = {initialized = false}

---@class EntityArchetypeStorage
---@field [integer] Entity
local entities = {}

---@class ComponentArchetypeStorage
---@field [integer] Component
local components = {}

---@class EntityInfo
---@field id integer
---@field archetype EntityArchetype

---@class ComponentInfo
---@field id integer
---@field archetype ComponentArchetype

function GlobalStorage.initStorage()
    if GlobalStorage.initialized then
        Log.Error("You are trying to reinitialize the GlobalStorage, this should not happen.")
        return
    end

    for _, archetype in pairs(Enums.EntityArchetype) do
        entities[archetype] = {}
    end

    for _, archetype in pairs(Enums.ComponentArchetype) do
        components[archetype] = {}
    end

    Log.Info("Initialized GlobalStorage")
    GlobalStorage.initialized = true
end

---@param entity Entity
function GlobalStorage.storeEntity(entity)
    if not entity:getArchetype() or not entities[entity:getArchetype()] then
        Log.Error("Did not provide a valid archetype for entity: " .. tostring(entity:getGuid()))
    end
    entities[entity:getArchetype()][entity:getGuid()] = entity
end

---@param archetype EntityArchetype
---@param entityId integer
---@return boolean wasSuccessful
function GlobalStorage.dropEntity(archetype, entityId)
    local entity = entities[archetype][entityId]

    if entity then
        --entity:destroy() --* how will we clean up?
        entities[archetype][entityId] = nil
        return true
    end
    return false
end

---@param component Component
function GlobalStorage.storeComponent(component)
    if not component:getArchetype() or not components[component:getArchetype()] then
        Log.Error("Did not provide a valid archetype for component: " .. tostring(component:getGuid()))
    end
    components[component:getArchetype()][component:getGuid()] = component
end

---@param archetype ComponentArchetype
---@param componentId integer
---@return boolean wasSuccessful
function GlobalStorage.dropComponent(archetype, componentId)
    local component = components[archetype][componentId]

    if component then
        --component:destroy() --* how will we clean up?
        components[archetype][componentId] = nil
        return true
    end
    return false
end

---@param entityInfo EntityInfo
---@return Entity|nil
function GlobalStorage.getEntity(entityInfo)
    ---@type EntityArchetypeStorage
    local archetypeStorage = entities[entityInfo.archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for entity: " .. entityInfo.id)
    end

    return archetypeStorage[entityInfo.id]
end

---@param componentInfo ComponentInfo
---@return Component|nil
function GlobalStorage.getComponentData(componentInfo)
    ---@type ComponentArchetypeStorage
    local archetypeStorage = components[componentInfo.archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for component: " .. componentInfo.id)
    end

    return archetypeStorage[componentInfo.id]
end

-- if you for some reason want all entities, should only be used for debugging
function GlobalStorage.getEntities()
    return entities
end

-- if you for some reason want all components, should only be used for debugging
function GlobalStorage.getComponents()
    return components
end

return GlobalStorage
