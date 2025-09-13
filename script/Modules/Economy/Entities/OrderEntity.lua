local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")

---@param issuerId number
---@param itemType integer
---@param quantity number
---@param price number
---@param expiresAt TimeStamp|nil
---@return Entity
return function(issuerId, itemType, quantity, price, expiresAt)
    return Entity.Create(
        "OrderEntity",
        Economy.Ownership(issuerId),
        Economy.OrderItemType(itemType),
        Economy.Quantity(quantity),
        Economy.Price(price),
        Economy.OrderStatus(),
        Economy.Expiry(expiresAt)
    )
end
