local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")       --!temp path
local MassComponent = require("_ECS_WIP_TEMP.Components.Physics.MassComponent") --!temp path

---@class ItemEntity: Entity
---@overload fun(self: ItemEntity): ItemEntity subclass internal
---@overload fun(): ItemEntity subclass external
local ItemEntity = Subclass(Entity, function(self)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.ItemEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Mass Component
    self:addComponent(MassComponent())
end)

return ItemEntity
