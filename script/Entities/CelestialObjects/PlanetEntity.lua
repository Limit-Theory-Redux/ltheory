local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class PlanetEntity: Entity
---@overload fun(self: PlanetEntity, seed: integer): PlanetEntity subclass internal
---@overload fun(seed: integer): PlanetEntity subclass external
local PlanetEntity = Subclass("PlanetEntity", Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.PlanetEntity)

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

return PlanetEntity
