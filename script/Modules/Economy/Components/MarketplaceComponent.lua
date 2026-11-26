local Entity = require("Core.ECS.Entity")
local Component = require("Core.ECS.Component")

---@class MarketTrade
---@field price integer
---@field quantity integer
---@field time number Simulated seconds

---@class MarketPullTimeout
---@field startedAt number Simulated seconds of the large movement that disabled the pull
---@field cooldownUntil number Simulated seconds until which no new timeout starts

---@class MarketConditions
---@field highDemand boolean
---@field lowSupply boolean
---@field volatile boolean
---@field lowDemand boolean
---@field highSupply boolean

---A market at an entity (a station): its open bids and asks (order entities), and the price state
---of each item traded there (trade history, emergent price, shifting equilibrium, pull timeout,
---conditions). `MarketplaceSystem` matches the orders and keeps the price state. The orders are
---destroyed with the market.
---@class MarketplaceComponent: Component
---@overload fun(self: MarketplaceComponent): MarketplaceComponent subclass internal
---@overload fun(): MarketplaceComponent subclass external
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
    ---@type table<integer, MarketTrade[]>
    self.history = {}
    ---@type table<integer, number>
    self.shiftingEquilibrium = {}
    ---@type table<integer, MarketPullTimeout>
    self.pullTimeouts = {}
    ---@type table<integer, MarketConditions>
    self.conditions = {}
end

---@param time number Simulated seconds
function MarketplaceComponent:setLastUpdated(time)
    self.lastUpdated = time
end

---@return number|nil lastUpdated Simulated seconds
function MarketplaceComponent:getLastUpdated()
    return self.lastUpdated
end

---@param time number Simulated seconds
function MarketplaceComponent:setNextUpdate(time)
    self.nextUpdate = time
end

---@return number|nil nextUpdate Simulated seconds
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

---@param entity Entity
---@return boolean
function MarketplaceComponent:hasBid(entity)
    return self.bids[entity.id] ~= nil
end

---@param entity Entity
---@return boolean
function MarketplaceComponent:hasAsk(entity)
    return self.asks[entity.id] ~= nil
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
---@return integer|nil
function MarketplaceComponent:getMarketPrice(itemType)
    return self.marketPrices[itemType]
end

---@return table<integer, integer> marketPrices Item id -> emergent price
function MarketplaceComponent:getMarketPrices()
    return self.marketPrices
end

---@param itemType integer
---@return MarketTrade[] history Oldest first; empty when the item has not traded here
function MarketplaceComponent:getHistory(itemType)
    return self.history[itemType] or {}
end

---Appends a trade, keeping at most `memory` of them.
---@param itemType integer
---@param trade MarketTrade
---@param memory integer
function MarketplaceComponent:recordTrade(itemType, trade, memory)
    local history = self.history[itemType]
    if not history then
        history = {}
        self.history[itemType] = history
    end
    history[#history + 1] = trade
    while #history > memory do
        table.remove(history, 1)
    end
end

---@param itemType integer
---@return number|nil
function MarketplaceComponent:getShiftingEquilibrium(itemType)
    return self.shiftingEquilibrium[itemType]
end

---@param itemType integer
---@param price number
function MarketplaceComponent:setShiftingEquilibrium(itemType, price)
    self.shiftingEquilibrium[itemType] = price
end

---@param itemType integer
---@return MarketPullTimeout|nil
function MarketplaceComponent:getPullTimeout(itemType)
    return self.pullTimeouts[itemType]
end

---@param itemType integer
---@param timeout MarketPullTimeout|nil
function MarketplaceComponent:setPullTimeout(itemType, timeout)
    self.pullTimeouts[itemType] = timeout
end

---@param itemType integer
---@return MarketConditions|nil
function MarketplaceComponent:getConditions(itemType)
    return self.conditions[itemType]
end

---@param itemType integer
---@param conditions MarketConditions
function MarketplaceComponent:setConditions(itemType, conditions)
    self.conditions[itemType] = conditions
end

---Destroys the open orders with the market. Their escrow goes with them.
function MarketplaceComponent:onDestroy()
    local Registry = require("Core.ECS.Registry")
    for _, set in ipairs({ self.bids, self.asks }) do
        for id in Iterator(set) do
            local order = Entity(id)
            if Registry:hasEntity(order) then Registry:destroyEntity(order) end
        end
    end
    self.bids, self.asks = {}, {}
end

return MarketplaceComponent
