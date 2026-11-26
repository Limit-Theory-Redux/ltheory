local Entity = require("Core.ECS.Entity")
local CoreComponents = require("Modules.Core.Components")
local Economy = require("Modules.Economy.Components")

---A market order. The issuer is the entity that trades it (a ship or a station, with an inventory):
---the goods of an ask come out of its inventory, a bid's credits out of its account, and what the
---order buys or earns goes back to it. Whether it is a bid or an ask is the set of the marketplace
---it is in. Post orders with `MarketplaceSystem:postBid`/`postAsk`, which hold their escrow.
---@param issuer Entity
---@param itemDefinition ItemDefinition
---@param quantity number
---@param price number Per unit
---@param expiresAt number|nil Simulated seconds (`MarketplaceSystem:now()`); nil never expires
---@return Entity
return function(issuer, itemDefinition, quantity, price, expiresAt)
    return Entity.Create(
        "OrderEntity",
        CoreComponents.Tag(),
        Economy.Ownership(issuer.id),
        Economy.Item(itemDefinition.id),
        Economy.Quantity(quantity),
        Economy.Price(price),
        Economy.OrderStatus(),
        Economy.Expiry(expiresAt)
    )
end
