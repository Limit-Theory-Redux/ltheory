local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")           --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy") --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")  --!temp path
local RNGComponent = require("_ECS_WIP_TEMP.Components.Generation.RNGComponent")    --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class UniverseEntity: Entity
---@overload fun(self: UniverseEntity, seed: integer): UniverseEntity subclass interal
---@overload fun(seed: integer): UniverseEntity subclass external
local UniverseEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.UniverseEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Random Number Generator Component
    self:addComponent(RNGComponent(seed))

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return UniverseEntity
