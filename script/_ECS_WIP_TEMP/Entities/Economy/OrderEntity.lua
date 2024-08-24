local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                     --!temp path
local QuantityComponent = require("_ECS_WIP_TEMP.Components.Economy.QuantityComponent")       --!temp path
local OrderStatusComponent = require("_ECS_WIP_TEMP.Components.Economy.OrderStatusComponent") --!temp path
local ExpiryComponent = require("_ECS_WIP_TEMP.Components.Economy.ExpiryComponent")           --!temp path

---@class OrderEntity: Entity
---@overload fun(self: OrderEntity): OrderEntity subclass internal
---@overload fun(): OrderEntity subclass external
local OrderEntity = Subclass(Entity, function(self)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.OrderEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Quantity Component
    self:addComponent(QuantityComponent())

    -- Order Status Component
    self:addComponent(OrderStatusComponent())

    -- Expiry Component
    self:addComponent(ExpiryComponent())
end)

return OrderEntity
