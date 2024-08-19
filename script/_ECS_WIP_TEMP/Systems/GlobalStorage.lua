---@class GlobalStorage
---@field entities table<EntityArchetypeStorage>
---@field components table<ComponentArchetypeStorage>

---@class EntityArchetypeStorage
---@field [integer] Entity

---@class ComponentArchetypeStorage
---@field [integer] Component

---@class GlobalStorage
local GlobalStorage = Class(function(self)
    ---@cast self GlobalStorage
    self:initStorage()
end)

function GlobalStorage:initStorage()
    self.entities = {}
    self.components = {}

    for archetype in Iterator(Enums.EntityArchetype) do
        self.entities[archetype] = {}
    end

    for archetype in Iterator(Enums.ComponentArchetype) do
        self.components[archetype] = {}
    end
end

---@param entity Entity
function GlobalStorage:storeEntity(entity)
    if not entity:getArchetype() and self.entities[entity:getArchetype()] then
        Log.Error("Did not provide a valid archetype for entity: " .. entity:getGuid())
    end
    insert(self.entities[entity:getArchetype()], entity:getGuid(), entity)
end

---@param archetype EntityArchetype
---@param entityId integer
---@return boolean wasSuccessful
function GlobalStorage:dropEntity(archetype, entityId)
    local entity = self.entities[archetype][entityId]

    if entity then
        --entity:destroy() --* how will we clean up?
        self.entities[archetype][entityId] = nil
        return true
    end
    return false
end

---@param component Component
function GlobalStorage:storeComponent(component)
    if not component:getArchetype() and self.components[component:getArchetype()] then
        Log.Error("Did not provide a valid archetype for component: " .. component:getGuid())
    end
    insert(self.components[component:getArchetype()], component:getGuid(), component)
end

---@param archetype ComponentArchetype
---@param componentId integer
---@return boolean wasSuccessful
function GlobalStorage:dropComponent(archetype, componentId)
    local component = self.components[archetype][componentId]

    if component then
        --component:destroy() --* how will we clean up?
        self.components[archetype][componentId] = nil
        return true
    end
    return false
end

---@param archetype EntityArchetype
---@param entityId integer
---@return Entity|nil
function GlobalStorage:getEntity(archetype, entityId)
    ---@type EntityArchetypeStorage
    local archetypeStorage = self.entities[archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for entity: " .. entityId)
    end

    return archetypeStorage[entityId]
end

---@param archetype ComponentArchetype
---@param componentId integer
---@return Component|nil
function GlobalStorage:getComponentData(archetype, componentId)
    ---@type ComponentArchetypeStorage
    local archetypeStorage = self.components[archetype]

    if not archetypeStorage then
        Log.Error("Did not provide a valid archetype for component: " .. componentId)
    end

    return archetypeStorage[componentId]
end

return GlobalStorage
