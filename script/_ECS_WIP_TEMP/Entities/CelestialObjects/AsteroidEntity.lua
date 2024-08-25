local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components

local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                 --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")        --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local MassComponent = require("_ECS_WIP_TEMP.Components.Physics.MassComponent")           --!temp path
local RigidBodyComponent = require("_ECS_WIP_TEMP.Components.Physics.RigidBodyComponent")          --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class AsteroidEntity: Entity
---@overload fun(self: AsteroidEntity, seed: integer): AsteroidEntity subclass internal
---@overload fun(seed: integer): AsteroidEntity subclass external
local AsteroidEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.AsteroidEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Mass Component
    self:addComponent(MassComponent())

    -- RigidBody Component
    self:addComponent(RigidBodyComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return AsteroidEntity
