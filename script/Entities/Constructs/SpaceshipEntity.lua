local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class SpaceshipEntity: Entity
---@overload fun(self: SpaceshipEntity, seed: integer): SpaceshipEntity subclass internal
---@overload fun(seed: integer): SpaceshipEntity subclass external
local SpaceshipEntity = Subclass("SpaceshipEntity", Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.SpaceshipEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Mass Component
    self:addComponent(MassComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return SpaceshipEntity
