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
        self.bids[arg] = arg
    end
end

---@param ... Entity
function MarketplaceComponent:addAsk(...)
    local args = { ... }
    for _, arg in ipairs(args) do
        self.asks[arg] = arg
    end
end

---@param entity Entity
---@return boolean success
function MarketplaceComponent:removeBid(entity)
    if not self.bids[entity] then
        return false
    end

    self.bids[entity] = nil
    return true
end

---@param entity Entity
---@return boolean success
function MarketplaceComponent:removeAsk(entity)
    if not self.asks[entity] then
        return false
    end

    self.asks[entity] = nil
    return true
end

---@return table<Entity>
function MarketplaceComponent:getBids()
    return self.bids
end

---@return table<Entity>
function MarketplaceComponent:getAsks()
    return self.asks
end

return MarketplaceComponent
