local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage") --!temp path

-- Entities
local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                 --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")        --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local MassComponent = require("_ECS_WIP_TEMP.Components.Physics.MassComponent")           --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

-- Utils
local Words = require('Systems.Gen.Words')

---@class StarSystemEntity: Entity
---@overload fun(self: StarSystemEntity, seed: integer): StarSystemEntity subclass internal
---@overload fun(seed: integer): StarSystemEntity subclass external
local StarSystemEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarSystemEntity)

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

return StarSystemEntity
