local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local TransformComponent = require("Components.Physics.TransformComponent")
local ShapeComponent = require("Components.Spatial.ShapeComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class TriggerEntity: Entity
---@overload fun(self: TriggerEntity): TriggerEntity subclass internal
---@overload fun(): TriggerEntity subclass external
local TriggerEntity = Subclass("TriggerEntity", Entity, function(self)
    -- Name Component
    self:addComponent(NameComponent())

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Shape Component
    self:addComponent(ShapeComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return TriggerEntity
