local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class MarketplaceComponent: Component
---@overload fun(self: MarketplaceComponent, playerId: integer|nil): MarketplaceComponent subclass internal
---@overload fun(playerId: integer|nil): MarketplaceComponent subclass external
local MarketplaceComponent = Subclass("MarketplaceComponent", Component, function(self)
    self:setComponentName("EconomyMarketplace")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.MarketplaceComponent)

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

---@param trader EntityInfo
function MarketplaceComponent:setTrader(trader)
    self.trader = trader
end

---@return EntityInfo Trader
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

---@param entityInfo EntityInfo
function MarketplaceComponent:addBid(entityInfo)
    self.bids[entityInfo.id] = entityInfo
end

---@param entityInfo EntityInfo
function MarketplaceComponent:addAsk(entityInfo)
    self.asks[entityInfo.id] = entityInfo
end

---@param entityInfo EntityInfo
---@return boolean success
function MarketplaceComponent:removeBid(entityInfo)
    if not self.bids[entityInfo.id] then
        return false
    end

    self.bids[entityInfo.id] = nil
    return true
end

---@param entityInfo EntityInfo
---@return boolean success
function MarketplaceComponent:removeAsk(entityInfo)
    if not self.asks[entityInfo.id] then
        return false
    end

    self.asks[entityInfo.id] = nil
    return true
end

---@return table<EntityInfo>
function MarketplaceComponent:getBids()
    return self.bids
end

---@return table<EntityInfo>
function MarketplaceComponent:getAsks()
    return self.asks
end

return MarketplaceComponent
