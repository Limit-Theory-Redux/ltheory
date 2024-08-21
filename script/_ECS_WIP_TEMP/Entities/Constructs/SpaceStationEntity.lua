local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")           --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")  --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.Transform")    --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy") --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class SpaceStationEntity: Entity
---@overload fun(self: SpaceStationEntity, seed: integer): SpaceStationEntity subclass internal
---@overload fun(seed: integer): SpaceStationEntity subclass external
local SpaceStationEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.SpaceStationEntity)

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

return SpaceStationEntity
