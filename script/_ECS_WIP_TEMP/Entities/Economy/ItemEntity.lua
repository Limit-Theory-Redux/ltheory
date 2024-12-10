local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")               --!temp path
local MassComponent = require("_ECS_WIP_TEMP.Components.Physics.MassComponent")         --!temp path
local QuantityComponent = require("_ECS_WIP_TEMP.Components.Economy.QuantityComponent") --!temp path

---@class ItemEntity: Entity
---@overload fun(self: ItemEntity, definition: ItemDefinition, quantity: number): ItemEntity subclass internal
---@overload fun(definition: ItemDefinition, quantity: number): ItemEntity subclass external
local ItemEntity = Subclass(Entity, function(self, definition, quantity)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.ItemEntity)

    -- Name Component
    self:addComponent(NameComponent(definition.name))

    -- Mass Component
    self:addComponent(MassComponent(definition.mass))

    -- QuantityComponent
    self:addComponent(QuantityComponent(quantity))
end)

return ItemEntity
