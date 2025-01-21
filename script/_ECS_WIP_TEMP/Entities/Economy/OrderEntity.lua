local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local OwnershipComponent = require("_ECS_WIP_TEMP.Components.Economy.OwnershipComponent")         --!temp path
local OrderItemTypeComponent = require("_ECS_WIP_TEMP.Components.Economy.OrderItemTypeComponent") --!temp path
local QuantityComponent = require("_ECS_WIP_TEMP.Components.Economy.QuantityComponent")           --!temp path
local PriceComponent = require("_ECS_WIP_TEMP.Components.Economy.PriceComponent")                 --!temp path
local OrderStatusComponent = require("_ECS_WIP_TEMP.Components.Economy.OrderStatusComponent")     --!temp path
local ExpiryComponent = require("_ECS_WIP_TEMP.Components.Economy.ExpiryComponent")               --!temp path

---@class OrderEntity: Entity
---@overload fun(self: OrderEntity, issuerId: number, itemType: integer, quantity: number, price: number, expiresAt: TimeStamp|nil): OrderEntity subclass internal
---@overload fun(issuerId: number, itemType: integer, quantity: number, price: number, expiresAt: TimeStamp|nil): OrderEntity subclass external
local OrderEntity = Subclass(Entity, function(self, issuerId, itemType, quantity, price, expiresAt)
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
