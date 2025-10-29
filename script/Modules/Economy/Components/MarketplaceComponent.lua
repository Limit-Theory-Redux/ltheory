local Entity = require("Core.ECS.Entity")
local Component = require("Core.ECS.Component")

---@class MarketplaceComponent: Component
---@overload fun(self: MarketplaceComponent, playerId: integer|nil): MarketplaceComponent subclass internal
---@overload fun(playerId: integer|nil): MarketplaceComponent subclass external
local MarketplaceComponent = Subclass("MarketplaceComponent", Component, function(self)
    self:setComponentName("EconomyMarketplace")

    self:init()
end)

function MarketplaceComponent:init()
    self.bids = {}
    SetLengthMetamethod(self.bids)
    self.asks = {}
    SetLengthMetamethod(self.asks)
    self.marketPrices = {}
end

---@param timestamp TimeStamp
function MarketplaceComponent:setLastUpdated(timestamp)
    self.lastUpdated = timestamp
end

---@return TimeStamp lastUpdated
function MarketplaceComponent:getLastUpdated()
    return self.lastUpdated
end

---@param timestamp TimeStamp
function MarketplaceComponent:setNextUpdate(timestamp)
    self.nextUpdate = timestamp
end

---@return TimeStamp nextUpdate
function MarketplaceComponent:getNextUpdate()
    return self.nextUpdate
end

---@param trader Entity
function MarketplaceComponent:setTrader(trader)
    self.trader = trader
end

---@return Entity Trader
function MarketplaceComponent:getTrader()
    return self.trader
end

---@param tax number in percent
function MarketplaceComponent:setTax(tax)
    self.tax = tax
end

---@return number tax in percent
function MarketplaceComponent:getTax()
    return self.tax
end

---@param ... Entity
function MarketplaceComponent:addBid(...)
    local args = { ... }
    for _, arg in ipairs(args) do
        self.bids[arg.id] = arg.id
    end
end

---@param ... Entity
function MarketplaceComponent:addAsk(...)
    local args = { ... }
    for _, arg in ipairs(args) do
        self.asks[arg.id] = arg.id
    end
end

---@param entity Entity
---@return boolean success
function MarketplaceComponent:removeBid(entity)
    if not self.bids[entity.id] then
        return false
    end

    self.bids[entity.id] = nil
    return true
end

---@param entity Entity
---@return boolean success
function MarketplaceComponent:removeAsk(entity)
    if not self.asks[entity.id] then
        return false
    end

    self.asks[entity.id] = nil
    return true
end

---@return table<Entity>
function MarketplaceComponent:getBids()
    local bids = {}
    for id in Iterator(self.bids) do
        local entity = Entity(id)
        if entity then
            table.insert(bids, entity)
        end
    end
    return bids
end

---@return table<Entity>
function MarketplaceComponent:getAsks()
    local asks = {}
    for id in Iterator(self.asks) do
        local entity = Entity(id)
        if entity then
            table.insert(asks, entity)
        end
    end
    return asks
end

---@param itemType integer
function MarketplaceComponent:setMarketPrice(itemType, price)
    self.marketPrices[itemType] = price
end

---@param itemType integer
function MarketplaceComponent:getMarketPrice(itemType)
    return self.marketPrices[itemType]
end

return MarketplaceComponent
