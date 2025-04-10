local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

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
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return AsteroidBeltEntity
