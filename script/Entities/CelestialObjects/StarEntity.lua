local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class StarEntity: Entity
---@overload fun(self: StarEntity, seed: integer): StarEntity subclass internal
---@overload fun(seed: integer): StarEntity subclass external
local StarEntity = Subclass("StarEntity", Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarEntity)

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

return StarEntity
