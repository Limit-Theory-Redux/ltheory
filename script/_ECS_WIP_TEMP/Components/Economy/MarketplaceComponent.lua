local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class MarketplaceComponent: Component
---@overload fun(self: MarketplaceComponent, playerId: integer|nil): MarketplaceComponent subclass internal
---@overload fun(playerId: integer|nil): MarketplaceComponent subclass external
local MarketplaceComponent = Subclass(Component, function(self)
    self:setComponentName("EconomyMarketplace")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.MarketplaceComponent)

    self:init()
end)

function MarketplaceComponent:init()
    self.wares = {}
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
---@param itemType string
function MarketplaceComponent:addBid(entityInfo, itemType)
    if not self.wares[itemType] then
        self.wares[itemType] = {
            bids = {},
            asks = {}
        }
    end
    self.wares[itemType].bids[entityInfo.id] = entityInfo
end

---@param entityInfo EntityInfo
---@param itemType string
function MarketplaceComponent:addAsk(entityInfo, itemType)
    if not self.wares[itemType] then
        self.wares[itemType] = {
            bids = {},
            asks = {}
        }
    end
    self.wares[itemType].asks[entityInfo.id] = entityInfo
end

---@param entityInfo EntityInfo
---@param itemType string
---@return boolean success
function MarketplaceComponent:removeBid(entityInfo, itemType)
    if not self.wares[itemType] then
        return false
    end

    if not self.wares[itemType].bids[entityInfo.id] then
        return false
    end

    self.wares[itemType].bids[entityInfo.id] = nil
    return true
end

---@param entityInfo EntityInfo
---@param itemType string
---@return boolean success
function MarketplaceComponent:removeAsk(entityInfo, itemType)
    if not self.wares[itemType] then
        return false
    end

    if not self.wares[itemType].asks[entityInfo.id] then
        return false
    end

    self.wares[itemType].asks[entityInfo.id] = nil
    return true
end

return MarketplaceComponent
