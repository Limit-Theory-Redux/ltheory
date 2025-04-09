local Entity = require("Entities.Entity")

-- Components
local OwnershipComponent = require("Components.Economy.OwnershipComponent")
local OrderItemTypeComponent = require("Components.Economy.OrderItemTypeComponent")
local QuantityComponent = require("Components.Economy.QuantityComponent")
local PriceComponent = require("Components.Economy.PriceComponent")
local OrderStatusComponent = require("Components.Economy.OrderStatusComponent")
local ExpiryComponent = require("Components.Economy.ExpiryComponent")

---@class OrderEntity: Entity
---@overload fun(self: OrderEntity, issuerId: number, itemType: integer, quantity: number, price: number, expiresAt: TimeStamp|nil): OrderEntity subclass internal
---@overload fun(issuerId: number, itemType: integer, quantity: number, price: number, expiresAt: TimeStamp|nil): OrderEntity subclass external
local OrderEntity = Subclass("OrderEntity", Entity, function(self, issuerId, itemType, quantity, price, expiresAt)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.OrderEntity)

    -- Ownership Component
    self:addComponent(OwnershipComponent(issuerId))

    -- ItemType Component
    self:addComponent(OrderItemTypeComponent(itemType))

    -- Quantity Component
    self:addComponent(QuantityComponent(quantity))

    -- Price Component
    self:addComponent(PriceComponent(price))

    -- Order Status Component
    self:addComponent(OrderStatusComponent())

    -- Expiry Component
    self:addComponent(ExpiryComponent(expiresAt))
end)

return OrderEntity
