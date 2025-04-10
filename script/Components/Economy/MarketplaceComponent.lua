local Component = require('Components.Component')

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

---@param trader EntityId
function MarketplaceComponent:setTrader(trader)
    self.trader = trader
end

---@return EntityId Trader
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

---@param entityId EntityId
function MarketplaceComponent:addBid(entityId)
    self.bids[entityId] = entityId
end

---@param entityId EntityId
function MarketplaceComponent:addAsk(entityId)
    self.asks[entityId] = entityId
end

---@param entityId EntityId
---@return boolean success
function MarketplaceComponent:removeBid(entityId)
    if not self.bids[entityId] then
        return false
    end

    self.bids[entityId] = nil
    return true
end

---@param entityId EntityId
---@return boolean success
function MarketplaceComponent:removeAsk(entityId)
    if not self.asks[entityId] then
        return false
    end

    self.asks[entityId] = nil
    return true
end

---@return table<EntityId>
function MarketplaceComponent:getBids()
    return self.bids
end

---@return table<EntityId>
function MarketplaceComponent:getAsks()
    return self.asks
end

return MarketplaceComponent
