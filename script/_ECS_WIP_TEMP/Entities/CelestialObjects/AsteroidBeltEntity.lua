local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                 --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")        --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local MassComponent = require("_ECS_WIP_TEMP.Components.Physics.MassComponent")           --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class AsteroidBeltEntity: Entity
---@overload fun(self: AsteroidBeltEntity, seed: integer): AsteroidBeltEntity subclass internal
---@overload fun(seed: integer): AsteroidBeltEntity subclass external
local AsteroidBeltEntity = Subclass("AsteroidBeltEntity", Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.AsteroidBeltEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Mass Component
    self:addComponent(MassComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return AsteroidBeltEntity
