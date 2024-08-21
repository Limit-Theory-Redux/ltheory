local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")           --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")  --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.Transform")    --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy") --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class AsteroidRingEntity: Entity
---@overload fun(self: AsteroidRingEntity, seed: integer): AsteroidRingEntity subclass internal
---@overload fun(seed: integer): AsteroidRingEntity subclass external
local AsteroidRingEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.AsteroidRingEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return AsteroidRingEntity
