local Entity = require("Core.ECS.Entity")
local CoreComponents = require("Modules.Core.Components")
local Economy = require("Modules.Economy.Components")

---@param issuer Entity
---@param itemDefinition ItemDefinition
---@param quantity number
---@param price number
---@param expiresAt TimeStamp|nil
---@return Entity
return function(issuer, itemDefinition, quantity, price, expiresAt)
    return Entity.Create(
        "OrderEntity",
        CoreComponents.Tag(),
        Economy.Ownership(issuer),
        Economy.ItemType(itemDefinition.id),
        Economy.Quantity(quantity),
        Economy.Price(price),
        Economy.OrderStatus(),
        Economy.Expiry(expiresAt)
    )
end
