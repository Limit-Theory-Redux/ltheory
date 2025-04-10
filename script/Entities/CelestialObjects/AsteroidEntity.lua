local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local RenderComponent = require("Components.Rendering.RenderComponent")
local RigidBodyComponent = require("Components.Physics.RigidBodyComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

-- Types
local Materials = require("Shared.Registries.Materials")

---@class AsteroidEntity: Entity
---@overload fun(self: AsteroidEntity, seed: integer): AsteroidEntity subclass internal
---@overload fun(seed: integer): AsteroidEntity subclass external
local AsteroidEntity = Subclass("AsteroidEntity", Entity, function(self, seed)
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
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return AsteroidEntity
