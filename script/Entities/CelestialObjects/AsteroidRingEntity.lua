local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class AsteroidRingEntity: Entity
---@overload fun(self: AsteroidRingEntity, seed: integer): AsteroidRingEntity subclass internal
---@overload fun(seed: integer): AsteroidRingEntity subclass external
local AsteroidRingEntity = Subclass("AsteroidRingEntity", Entity, function(self, seed)
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

return AsteroidRingEntity
