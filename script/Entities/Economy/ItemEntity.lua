local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local MassComponent = require("Components.Physics.MassComponent")
local QuantityComponent = require("Components.Economy.QuantityComponent")

---@class ItemEntity: Entity
---@overload fun(self: ItemEntity, definition: ItemDefinition, quantity: number): ItemEntity subclass internal
---@overload fun(definition: ItemDefinition, quantity: number): ItemEntity subclass external
local ItemEntity = Subclass("ItemEntity", Entity, function(self, definition, quantity)
    -- Name Component
    self:addComponent(NameComponent(definition.name))

    -- Mass Component
    self:addComponent(MassComponent(definition.mass))

    -- QuantityComponent
    self:addComponent(QuantityComponent(quantity))
end)

return ItemEntity
