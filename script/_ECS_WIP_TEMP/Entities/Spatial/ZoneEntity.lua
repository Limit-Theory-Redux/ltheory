local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")           --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.Transform")    --!temp path
local ShapeComponent = require("_ECS_WIP_TEMP.Components.Spatial.ShapeComponent")   --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy") --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

---@class ZoneEntity: Entity
---@overload fun(self: ZoneEntity): ZoneEntity subclass internal
---@overload fun(): ZoneEntity subclass external
local ZoneEntity = Subclass(Entity, function(self)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.ZoneEntity)

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

return ZoneEntity
