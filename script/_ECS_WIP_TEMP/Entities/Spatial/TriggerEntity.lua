local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                 --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local ShapeComponent = require("_ECS_WIP_TEMP.Components.Spatial.ShapeComponent")         --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class TriggerEntity: Entity
---@overload fun(self: TriggerEntity): TriggerEntity subclass internal
---@overload fun(): TriggerEntity subclass external
local TriggerEntity = Subclass(Entity, function(self)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.TriggerEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Shape Component
    self:addComponent(ShapeComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return TriggerEntity
