local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local Items = require("Shared.Registries.Items")

local Economy = require("Modules.Economy.Components")
local CoreComponents = require("Modules.Core.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")

---@class MarketplaceSystem
local MarketplaceSystem = Class("MarketplaceSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function MarketplaceSystem:registerVars()
    self.rng = RNG.FromTime()
    self.updateRate = 2
    self.maxUpdateRateDeviation = 0.5

    -- Economic parameters
    self.priceHistory = {}       -- Track price history per item type per marketplace
    self.priceMemory = 20        -- Number of trades to remember
    self.volatility = 0.05       -- Base market volatility (5%)
    self.spreadPercentage = 0.02 -- Bid-ask spread (2%)

    -- Emergent economy settings
    self.useEmergentPricing = true -- Set to true for real economy without base prices
    self.priceDiscoveryPeriod = 10 -- Trades needed before price stabilizes
    self.defaultStartPrice = 50    -- Only used if no market history exists
    self.minPrice = 1              -- Absolute minimum price
    self.maxPriceChange = 0.25     -- Max 25% price change per update
end

function MarketplaceSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

function MarketplaceSystem:onPreRender()
    local now = TimeStamp.Now()
    for _, marketplace in Registry:iterEntities(Economy.Marketplace) do
        local nextUpdate = marketplace:getNextUpdate()
        if not nextUpdate then
            nextUpdate = TimeStamp.GetFuture(self:updateDeviation())
            marketplace:setNextUpdate(nextUpdate)
        end

        if now:getDifference(nextUpdate) <= 0 then
            self:updateMarketplace(marketplace)
            marketplace:setNextUpdate(TimeStamp.GetFuture(self:updateDeviation()))
        end
    end
end

function MarketplaceSystem:updateDeviation()
    return self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation)
end

---@param marketplace MarketplaceComponent
function MarketplaceSystem:updateMarketplace(marketplace)
    -- Sort bids and asks for efficient matching
    local bids = self:sortBids(marketplace:getBids())
    local asks = self:sortAsks(marketplace:getAsks())

    -- Process trades using order matching
    self:processTrades(marketplace, bids, asks)

    -- Pull fresh order book after trades (entities may have been destroyed)
    bids = self:sortBids(marketplace:getBids())
    asks = self:sortAsks(marketplace:getAsks())

    -- Update market prices based on supply/demand
    self:updateMarketPrices(marketplace, bids, asks)

    -- Update market condition tags on all orders
    self:updateMarketConditionTags(marketplace, bids, asks)
end

---Sort bids by price (highest first)
---@param bids Entity[]
---@return Entity[]
function MarketplaceSystem:sortBids(bids)
    local sortedBids = {}
    for bid in Iterator(bids) do
        table.insert(sortedBids, bid)
    end

    table.sort(sortedBids, function(a, b)
        local priceA = a:get(Economy.Price):getPrice()
        local priceB = b:get(Economy.Price):getPrice()
        return priceA > priceB -- Highest bid first
    end)

    return sortedBids
end

---Sort asks by price (lowest first)
---@param asks Entity[]
---@return Entity[]
function MarketplaceSystem:sortAsks(asks)
    local sortedAsks = {}
    for ask in Iterator(asks) do
        table.insert(sortedAsks, ask)
    end

    table.sort(sortedAsks, function(a, b)
        local priceA = a:get(Economy.Price):getPrice()
        local priceB = b:get(Economy.Price):getPrice()
        return priceA < priceB -- Lowest ask first
    end)

    return sortedAsks
end

---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:processTrades(marketplace, bids, asks)
    local marketplaceId = tostring(marketplace)

    for _, bid in ipairs(bids) do
        -- Check if bid still exists (might have been destroyed)
        if not bid:isValid() then goto continueBid end

        local bidTag = bid:get(CoreComponents.Tag)
        local bidItemType = bid:get(Economy.ItemType):getItemType()
        local bidPrice = bid:get(Economy.Price):getPrice()
        local bidQty = bid:get(Economy.Quantity):getQuantity()
        local bidOwner = bid:get(Economy.Ownership):getOwner()

        if bidTag:hasTag("Contraband") then goto continueBid end

        for _, ask in ipairs(asks) do
            -- Check if ask still exists (might have been destroyed in previous iteration)
            if not ask:isValid() then goto continueAsk end

            local askTag = ask:get(CoreComponents.Tag)
            local askItemType = ask:get(Economy.ItemType):getItemType()
            local askPrice = ask:get(Economy.Price):getPrice()
            local askQty = ask:get(Economy.Quantity):getQuantity()
            local askOwner = ask:get(Economy.Ownership):getOwner()

            -- Skip if contraband or different item types
            if askTag:hasTag("Contraband") then goto continueAsk end
            if bidItemType ~= askItemType then goto continueAsk end

            -- Match orders: bid price must be >= ask price
            if bidPrice < askPrice then goto continueAsk end

            -- Verify seller has the items
            local askOwnerInv = Entity(askOwner):get(CoreComponents.Parent):getParent():get(Economy.Inventory)
            local bidOwnerInv = Entity(bidOwner):get(CoreComponents.Parent):getParent():get(Economy.Inventory)

            -- Calculate trade details
            local tradeQty = math.min(bidQty, askQty)
            local tradePrice = self:calculateTradePrice(bidPrice, askPrice)

            -- Execute trade
            local items = InventoryManager:take(askOwnerInv, askItemType, tradeQty)
            if items then
                InventoryManager:put(bidOwnerInv, items)

                -- Transfer currency (bid price paid to seller)
                self:transferCurrency(bidOwner, askOwner, tradePrice * tradeQty)

                -- Update quantities
                bidQty = bidQty - tradeQty
                askQty = askQty - tradeQty
                bid:get(Economy.Quantity):setQuantity(bidQty)
                ask:get(Economy.Quantity):setQuantity(askQty)

                -- Record transaction for price history
                self:recordTrade(marketplaceId, bidItemType, tradePrice, tradeQty)

                -- Fire transaction event
                --* Transaction event here? Fake it for now
                local app = require("States.Application")
                app:onTransaction({
                    marketplace = marketplace,
                    itemType = bidItemType,
                    quantity = tradeQty,
                    price = tradePrice,
                    buyer = bidOwner,
                    seller = askOwner
                })

                Log.Debug("[Transaction] %s bought %d x %s from %s for %d each (total: %d)",
                    Entity(bidOwner), tradeQty, Items:getDefinition(bidItemType).name,
                    Entity(askOwner), tradePrice, tradePrice * tradeQty)

                -- Remove filled orders
                if bidQty <= 0 then
                    marketplace:removeBid(bid)
                    Registry:destroyEntity(bid)
                    break
                end
                if askQty <= 0 then
                    marketplace:removeAsk(ask)
                    Registry:destroyEntity(ask)
                    -- Don't break here, continue to next ask
                end
            end
            ::continueAsk::
        end
        ::continueBid::
    end
end

---Calculate the actual trade price (typically midpoint or ask price)
---@param bidPrice number
---@param askPrice number
---@return number
function MarketplaceSystem:calculateTradePrice(bidPrice, askPrice)
    -- Use midpoint pricing for fair market execution
    return math.floor((bidPrice + askPrice) / 2)
end

---Record a trade in price history
---@param marketplaceId string
---@param itemType string
---@param price number
---@param quantity number
function MarketplaceSystem:recordTrade(marketplaceId, itemType, price, quantity)
    if not self.priceHistory[marketplaceId] then
        self.priceHistory[marketplaceId] = {}
    end
    if not self.priceHistory[marketplaceId][itemType] then
        self.priceHistory[marketplaceId][itemType] = {}
    end

    local history = self.priceHistory[marketplaceId][itemType]
    table.insert(history, {
        price = price,
        quantity = quantity,
        timestamp = TimeStamp.Now()
    })

    -- Keep only recent history
    while #history > self.priceMemory do
        table.remove(history, 1)
    end
end

---Update market prices based on supply and demand
---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:updateMarketPrices(marketplace, bids, asks)
    local marketplaceId = tostring(marketplace)
    local itemTypes = self:getActiveItemTypes(bids, asks)

    for itemType, _ in pairs(itemTypes) do
        local supplyDemand = self:calculateSupplyDemand(bids, asks, itemType)
        local marketPrice = self:calculateEmergentPrice(marketplaceId, itemType, bids, asks, supplyDemand)

        -- Store market price in marketplace component
        if marketplace.setMarketPrice then
            marketplace:setMarketPrice(itemType, marketPrice)
        end
    end
end

---Get all unique item types from bids and asks
---@param bids Entity[]
---@param asks Entity[]
---@return table<string, boolean>
function MarketplaceSystem:getActiveItemTypes(bids, asks)
    local itemTypes = {}

    for _, bid in ipairs(bids) do
        local itemType = bid:get(Economy.ItemType):getItemType()
        itemTypes[itemType] = true
    end

    for _, ask in ipairs(asks) do
        local itemType = ask:get(Economy.ItemType):getItemType()
        itemTypes[itemType] = true
    end

    return itemTypes
end

---Calculate supply/demand ratio for an item type
---@param bids Entity[]
---@param asks Entity[]
---@param itemType string
---@return number
function MarketplaceSystem:calculateSupplyDemand(bids, asks, itemType)
    local totalDemand = 0
    local totalSupply = 0

    for _, bid in ipairs(bids) do
        if bid:get(Economy.ItemType):getItemType() == itemType then
            totalDemand = totalDemand + bid:get(Economy.Quantity):getQuantity()
        end
    end

    for _, ask in ipairs(asks) do
        if ask:get(Economy.ItemType):getItemType() == itemType then
            totalSupply = totalSupply + ask:get(Economy.Quantity):getQuantity()
        end
    end

    if totalSupply == 0 then return 2.0 end -- High demand, no supply
    if totalDemand == 0 then return 0.5 end -- Supply but no demand

    return totalDemand / totalSupply
end

---Calculate emergent market price from order book and history
---@param marketplaceId string
---@param itemType string
---@param bids Entity[]
---@param asks Entity[]
---@param supplyDemandRatio number
---@return number
function MarketplaceSystem:calculateEmergentPrice(marketplaceId, itemType, bids, asks, supplyDemandRatio)
    -- Get current order book prices
    local bestBid = self:getBestBidPrice(bids, itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)

    -- Get historical price
    local historicalPrice = self:getHistoricalAverage(marketplaceId, itemType)
    local tradeCount = self:getTradeCount(marketplaceId, itemType)

    -- Calculate base price from multiple sources
    local basePrice = nil

    if historicalPrice and tradeCount >= self.priceDiscoveryPeriod then
        -- Market is mature, use historical weighted heavily
        basePrice = historicalPrice
    elseif bestBid and bestAsk then
        -- Use midpoint of best bid/ask
        basePrice = (bestBid + bestAsk) / 2
    elseif bestBid then
        -- Only bids exist, use bid as baseline
        basePrice = bestBid
    elseif bestAsk then
        -- Only asks exist, use ask as baseline
        basePrice = bestAsk
    elseif historicalPrice then
        -- Young market with some history
        basePrice = historicalPrice
    else
        -- Brand new market, use default
        basePrice = self.defaultStartPrice
    end

    -- Apply supply/demand pressure
    local demandFactor = self:calculateDemandFactor(supplyDemandRatio)

    -- Calculate target price
    local targetPrice = basePrice * demandFactor

    -- Smooth price changes if we have history
    if historicalPrice then
        local maxChange = historicalPrice * self.maxPriceChange
        targetPrice = math.max(historicalPrice - maxChange,
            math.min(historicalPrice + maxChange, targetPrice))
    end

    -- Add market volatility (decreases as market matures)
    local maturityFactor = math.min(1.0, tradeCount / self.priceDiscoveryPeriod)
    local adjustedVolatility = self.volatility * (1.0 - maturityFactor * 0.5)
    local volatilityFactor = 1.0 + self.rng:getUniformRange(-adjustedVolatility, adjustedVolatility)
    targetPrice = targetPrice * volatilityFactor

    return math.max(self.minPrice, math.floor(targetPrice))
end

---Calculate demand pressure factor
---@param supplyDemandRatio number
---@return number
function MarketplaceSystem:calculateDemandFactor(supplyDemandRatio)
    local demandFactor = 1.0

    if supplyDemandRatio > 1.0 then
        -- Demand exceeds supply: price increases
        -- Using logarithmic scaling for realistic price pressure
        demandFactor = 1.0 + math.log(supplyDemandRatio) * 0.4
    elseif supplyDemandRatio < 1.0 then
        -- Supply exceeds demand: price decreases
        local oversupply = 1.0 / supplyDemandRatio
        demandFactor = 1.0 / (1.0 + math.log(oversupply) * 0.4)
    end

    return demandFactor
end

---Get best (highest) bid price for an item
---@param bids Entity[]
---@param itemType string
---@return number|nil
function MarketplaceSystem:getBestBidPrice(bids, itemType)
    local bestPrice = nil
    for _, bid in ipairs(bids) do
        if bid:get(Economy.ItemType):getItemType() == itemType then
            local price = bid:get(Economy.Price):getPrice()
            if not bestPrice or price > bestPrice then
                bestPrice = price
            end
        end
    end
    return bestPrice
end

---Get best (lowest) ask price for an item
---@param asks Entity[]
---@param itemType string
---@return number|nil
function MarketplaceSystem:getBestAskPrice(asks, itemType)
    local bestPrice = nil
    for _, ask in ipairs(asks) do
        if ask:get(Economy.ItemType):getItemType() == itemType then
            local price = ask:get(Economy.Price):getPrice()
            if not bestPrice or price < bestPrice then
                bestPrice = price
            end
        end
    end
    return bestPrice
end

---Get number of trades recorded
---@param marketplaceId string
---@param itemType string
---@return number
function MarketplaceSystem:getTradeCount(marketplaceId, itemType)
    if not self.priceHistory[marketplaceId] then return 0 end
    if not self.priceHistory[marketplaceId][itemType] then return 0 end
    return #self.priceHistory[marketplaceId][itemType]
end

---Get historical average price (volume-weighted)
---@param marketplaceId string
---@param itemType string
---@return number|nil
function MarketplaceSystem:getHistoricalAverage(marketplaceId, itemType)
    if not self.priceHistory[marketplaceId] then return nil end
    if not self.priceHistory[marketplaceId][itemType] then return nil end

    local history = self.priceHistory[marketplaceId][itemType]
    if #history == 0 then return nil end

    -- Volume-weighted average with recency bias
    local totalPrice = 0
    local totalWeight = 0

    for i, trade in ipairs(history) do
        -- Weight combines recency and volume
        local recencyWeight = i / #history                -- Linear: recent trades weighted more
        local volumeWeight = math.log(trade.quantity + 1) -- Log scale for volume
        local weight = recencyWeight * volumeWeight

        totalPrice = totalPrice + (trade.price * weight)
        totalWeight = totalWeight + weight
    end

    return totalWeight > 0 and (totalPrice / totalWeight) or nil
end

---Calculate suggested bid price for an item (emergent)
---@param marketplace MarketplaceComponent
---@param itemType string
---@return number
function MarketplaceSystem:getSuggestedBidPrice(marketplace, itemType)
    local marketplaceId = tostring(marketplace)
    local bids = self:sortBids(marketplace:getBids())
    local asks = self:sortAsks(marketplace:getAsks())

    -- Get current market state
    local bestBid = self:getBestBidPrice(bids, itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplaceId, itemType)

    local suggestedPrice

    if bestAsk then
        -- Bid slightly below best ask to be competitive
        suggestedPrice = math.floor(bestAsk * 0.98)
    elseif bestBid then
        -- Bid slightly above current best bid
        suggestedPrice = math.ceil(bestBid * 1.02)
    elseif historicalPrice then
        -- Use historical with spread
        suggestedPrice = math.floor(historicalPrice * (1.0 - self.spreadPercentage))
    else
        -- New market, use default
        suggestedPrice = math.floor(self.defaultStartPrice * 0.95)
    end

    return math.max(self.minPrice, suggestedPrice)
end

---Calculate suggested ask price for an item (emergent)
---@param marketplace MarketplaceComponent
---@param itemType string
---@return number
function MarketplaceSystem:getSuggestedAskPrice(marketplace, itemType)
    local marketplaceId = tostring(marketplace)
    local bids = self:sortBids(marketplace:getBids())
    local asks = self:sortAsks(marketplace:getAsks())

    -- Get current market state
    local bestBid = self:getBestBidPrice(bids, itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplaceId, itemType)

    local suggestedPrice

    if bestBid then
        -- Ask slightly above best bid to get filled
        suggestedPrice = math.ceil(bestBid * 1.02)
    elseif bestAsk then
        -- Ask slightly below current best ask to be competitive
        suggestedPrice = math.floor(bestAsk * 0.98)
    elseif historicalPrice then
        -- Use historical with spread
        suggestedPrice = math.ceil(historicalPrice * (1.0 + self.spreadPercentage))
    else
        -- New market, use default
        suggestedPrice = math.ceil(self.defaultStartPrice * 1.05)
    end

    return math.max(self.minPrice, suggestedPrice)
end

---Get market depth (total quantity at each price level)
---@param marketplace MarketplaceComponent
---@param itemType string
---@return table, table
function MarketplaceSystem:getMarketDepth(marketplace, itemType)
    local bids = marketplace:getBids()
    local asks = marketplace:getAsks()

    local bidDepth = {} -- price -> quantity
    local askDepth = {} -- price -> quantity

    for bid in Iterator(bids) do
        if bid:get(Economy.ItemType):getItemType() == itemType then
            local price = bid:get(Economy.Price):getPrice()
            local qty = bid:get(Economy.Quantity):getQuantity()
            bidDepth[price] = (bidDepth[price] or 0) + qty
        end
    end

    for ask in Iterator(asks) do
        if ask:get(Economy.ItemType):getItemType() == itemType then
            local price = ask:get(Economy.Price):getPrice()
            local qty = ask:get(Economy.Quantity):getQuantity()
            askDepth[price] = (askDepth[price] or 0) + qty
        end
    end

    return bidDepth, askDepth
end

---Transfer currency between entities
---@param fromEntity Entity
---@param toEntity Entity
---@param amount number
function MarketplaceSystem:transferCurrency(fromEntity, toEntity, amount)
    local fromWallet = Entity(fromEntity):get(CoreComponents.Parent):getParent():get(Economy.Wallet)
    local toWallet = Entity(toEntity):get(CoreComponents.Parent):getParent():get(Economy.Wallet)

    if fromWallet and toWallet then
        fromWallet:subtract(amount)
        toWallet:add(amount)
    end
end

---Update market condition tags on orders based on market state
---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:updateMarketConditionTags(marketplace, bids, asks)
    local marketplaceId = tostring(marketplace)
    local itemTypes = self:getActiveItemTypes(bids, asks)

    for itemType, _ in pairs(itemTypes) do
        local conditions = self:analyzeMarketConditions(marketplaceId, itemType, bids, asks)

        -- Apply tags to all orders of this item type
        for _, bid in ipairs(bids) do
            if bid:get(Economy.ItemType):getItemType() == itemType then
                self:applyConditionTags(bid, conditions)
            end
        end

        for _, ask in ipairs(asks) do
            if ask:get(Economy.ItemType):getItemType() == itemType then
                self:applyConditionTags(ask, conditions)
            end
        end
    end
end

---Analyze market conditions for an item type
---@param marketplaceId string
---@param itemType string
---@param bids Entity[]
---@param asks Entity[]
---@return table
function MarketplaceSystem:analyzeMarketConditions(marketplaceId, itemType, bids, asks)
    local conditions = {
        highDemand = false,
        lowSupply = false,
        volatile = false
    }

    -- Calculate supply and demand
    local totalDemand = 0
    local totalSupply = 0

    for _, bid in ipairs(bids) do
        if bid:get(Economy.ItemType):getItemType() == itemType then
            totalDemand = totalDemand + bid:get(Economy.Quantity):getQuantity()
        end
    end

    for _, ask in ipairs(asks) do
        if ask:get(Economy.ItemType):getItemType() == itemType then
            totalSupply = totalSupply + ask:get(Economy.Quantity):getQuantity()
        end
    end

    -- High demand: demand significantly exceeds supply
    if totalSupply > 0 and totalDemand / totalSupply > 1.5 then
        conditions.highDemand = true
    elseif totalSupply == 0 and totalDemand > 0 then
        conditions.highDemand = true
        conditions.lowSupply = true
    end

    -- Low supply: supply significantly lower than demand
    if totalDemand > 0 and totalSupply / totalDemand < 0.5 then
        conditions.lowSupply = true
    end

    -- Volatile: check price variance in recent history
    if self.priceHistory[marketplaceId] and self.priceHistory[marketplaceId][itemType] then
        local history = self.priceHistory[marketplaceId][itemType]
        if #history >= 5 then
            local priceVariance = self:calculatePriceVariance(history)
            local avgPrice = self:getHistoricalAverage(marketplaceId, itemType)

            if avgPrice and priceVariance / avgPrice > 0.15 then -- 15% coefficient of variation
                conditions.volatile = true
            end
        end
    end

    return conditions
end

---Calculate price variance from history
---@param history table[]
---@return number
function MarketplaceSystem:calculatePriceVariance(history)
    if #history < 2 then return 0 end

    -- Calculate mean
    local sum = 0
    for _, trade in ipairs(history) do
        sum = sum + trade.price
    end
    local mean = sum / #history

    -- Calculate variance
    local varianceSum = 0
    for _, trade in ipairs(history) do
        local diff = trade.price - mean
        varianceSum = varianceSum + (diff * diff)
    end

    return math.sqrt(varianceSum / #history) -- Standard deviation
end

---Apply condition tags to an order entity
---@param order Entity
---@param conditions table
function MarketplaceSystem:applyConditionTags(order, conditions)
    local tagComponent = order:get(CoreComponents.Tag)
    if not tagComponent then return end

    -- Remove old condition tags
    tagComponent:removeTag("HighDemand")
    tagComponent:removeTag("LowSupply")
    tagComponent:removeTag("Volatile")

    -- Apply new condition tags
    if conditions.highDemand then
        tagComponent:addTag("HighDemand")
    end

    if conditions.lowSupply then
        tagComponent:addTag("LowSupply")
    end

    if conditions.volatile then
        tagComponent:addTag("Volatile")
    end
end

---Apply market modifiers based on tags (for AI agents pricing decisions)
---@param tagComponent TagComponent
---@param price number
---@return number
function MarketplaceSystem:applyMarketModifiers(tagComponent, price)
    -- Market condition modifiers
    if tagComponent:hasTag("HighDemand") then
        price = price * 1.2
    end

    if tagComponent:hasTag("LowSupply") then
        price = price * 1.15
    end

    if tagComponent:hasTag("Volatile") then
        -- Volatile markets: add risk premium
        price = price * 1.1
    end

    -- Item characteristic modifiers (from item definition, not market state)
    if tagComponent:hasTag("Luxury") then
        price = price * 1.5
    end

    if tagComponent:hasTag("RawMaterial") then
        price = price * 0.85 -- Raw materials typically cheaper
    end

    if tagComponent:hasTag("Consumable") then
        price = price * 0.95 -- Consumables have steady demand
    end

    return price
end

return MarketplaceSystem()
