local Entity = require("Entities.Entity")
local Components = require("Components")

---@param issuerId number
---@param itemType integer
---@param quantity number
---@param price number
---@param expiresAt TimeStamp|nil
---@return Entity
local function OrderEntity(issuerId, itemType, quantity, price, expiresAt)
    return Entity(
        Components.OwnershipComponent(issuerId),
        Components.OrderItemTypeComponent(itemType),
        Components.QuantityComponent(quantity),
        Components.PriceComponent(price),
        Components.OrderStatusComponent(),
        Components.ExpiryComponent(expiresAt)
    )
end

return OrderEntity
