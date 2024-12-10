local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                 --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent")        --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local MassComponent = require("_ECS_WIP_TEMP.Components.Physics.MassComponent")           --!temp path
local RenderComponent = require("_ECS_WIP_TEMP.Components.Rendering.RenderComponent")     --!temp path
local RigidBodyComponent = require("_ECS_WIP_TEMP.Components.Physics.RigidBodyComponent") --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")
local Materials = require("_ECS_WIP_TEMP.Shared.Registries.Materials")

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

    -- Render Component
    self:addComponent(RenderComponent({ Materials.Asteroid }, Enums.MeshType.Asteroid))

    -- RigidBody Component
    self:addComponent(RigidBodyComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return AsteroidEntity
