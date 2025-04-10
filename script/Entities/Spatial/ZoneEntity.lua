local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local TransformComponent = require("Components.Physics.TransformComponent")
local ShapeComponent = require("Components.Spatial.ShapeComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class ZoneEntity: Entity
---@overload fun(self: ZoneEntity): ZoneEntity subclass internal
---@overload fun(): ZoneEntity subclass external
local ZoneEntity = Subclass("ZoneEntity", Entity, function(self)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.ZoneEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Shape Component
    self:addComponent(ShapeComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return ZoneEntity
