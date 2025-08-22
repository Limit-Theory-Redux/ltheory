local Entity = require("Core.ECS.Entity")

---@class OrderEntity: Entity
---@param issuerId number
---@param itemType integer
---@param quantity number
---@param price number
---@param expiresAt TimeStamp|nil
return function(issuerId, itemType, quantity, price, expiresAt)
    local Components = require("Modules.Economy").Components

    return Entity(
        "OrderEntity",
        Components.Ownership(issuerId),
        Components.OrderItemType(itemType),
        Components.Quantity(quantity),
        Components.Price(price),
        Components.OrderStatus(),
        Components.Expiry(expiresAt)
    )
end
