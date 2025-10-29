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
    self.updateRate = 0.01
    self.maxUpdateRateDeviation = 0
    self.dt = 0

    -- Economic parameters
    self.priceHistory = {}
    self.priceMemory = 1000
    self.volatility = 0.05
    self.priceDiscoveryPeriod = 1000
    self.defaultStartPrice = 100
    self.minPrice = 1
    self.maxPriceChange = 0.02

    -- New equilibrium pull parameters
    self.shiftingEquilibrium = {}     -- marketplaceId -> itemType -> value
    self.pullStrength = 0.1           -- Base pull factor
    self.pullScale = 100              -- Controls exponential pull sensitivity
    self.shiftAlpha = 0.01            -- EMA alpha for shifting equilibrium
    self.pullTimeout = 7200           -- 2 hours for long timeout scenario
    self.largeMovementThreshold = 2.0 -- 200% deviation from shiftingEq
    self.pullTimeoutData = {}         -- marketplaceId -> itemType -> {timestamp, cooldownUntil}
    self.timeoutCooldown = 3600       -- 1 hour cooldown after timeout ends
    self.pullRecoveryRate = 0.0001    -- Exponential recovery speed (tuned for ~10 min to reach ~63% pull strength)

    -- Pending trades buffer
    self.pendingTrades = {}
end

function MarketplaceSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

---@param e EventData
function MarketplaceSystem:onPreRender(e)
    self.dt = e:deltaTime()

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
    self:removeExpiredOrders(marketplace)
    local bids = self:sortBids(marketplace:getBids())
    local asks = self:sortAsks(marketplace:getAsks())
    self:processTrades(marketplace, bids, asks)
    bids = self:sortBids(marketplace:getBids())
    asks = self:sortAsks(marketplace:getAsks())
    self:processPendingTrades()
    self:updateMarketConditionTags(marketplace, bids, asks)
    self:updateMarketPrices(marketplace, bids, asks)
end

---@param marketplace MarketplaceComponent
function MarketplaceSystem:removeExpiredOrders(marketplace)
    local now = TimeStamp.Now()
    local bids = marketplace:getBids()
    local asks = marketplace:getAsks()

    for bid in Iterator(bids) do
        local expiryComp = bid:get(Economy.Expiry)
        if expiryComp and expiryComp:getSecondsUntilExpire() <= 0 then
            marketplace:removeBid(bid)
            Registry:destroyEntity(bid)
            Log.Debug("[MarketplaceSystem] Removed expired bid: %s", tostring(bid))
        end
    end

    for ask in Iterator(asks) do
        local expiryComp = ask:get(Economy.Expiry)
        if expiryComp and expiryComp:getSecondsUntilExpire() <= 0 then
            marketplace:removeAsk(ask)
            Registry:destroyEntity(ask)
            Log.Debug("[MarketplaceSystem] Removed expired ask: %s", tostring(ask))
        end
    end
end

---@param bids Entity[]
function MarketplaceSystem:sortBids(bids)
    local sortedBids = {}
    for bid in Iterator(bids) do
        table.insert(sortedBids, bid)
    end
    table.sort(sortedBids, function(a, b)
        local priceA = a:get(Economy.Price):getPrice()
        local priceB = b:get(Economy.Price):getPrice()
        return priceA > priceB
    end)
    return sortedBids
end

---@param asks Entity[]
function MarketplaceSystem:sortAsks(asks)
    local sortedAsks = {}
    for ask in Iterator(asks) do
        table.insert(sortedAsks, ask)
    end
    table.sort(sortedAsks, function(a, b)
        local priceA = a:get(Economy.Price):getPrice()
        local priceB = b:get(Economy.Price):getPrice()
        return priceA < priceB
    end)
    return sortedAsks
end

---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:processTrades(marketplace, bids, asks)
    local marketplaceId = tostring(marketplace)
    self.pendingTrades = self.pendingTrades or {}

    for _, bid in ipairs(bids) do
        if not bid:isValid() then goto continueBid end
        local bidTag = bid:get(CoreComponents.Tag)
        local bidItemType = bid:get(Economy.ItemType):getItemType()
        local bidPrice = bid:get(Economy.Price):getPrice()
        local bidQty = bid:get(Economy.Quantity):getQuantity()
        local bidOwner = bid:get(Economy.Ownership):getOwner()

        if bidTag:hasTag("Contraband") then goto continueBid end

        for _, ask in ipairs(asks) do
            if not ask:isValid() then goto continueAsk end
            local askTag = ask:get(CoreComponents.Tag)
            local askItemType = ask:get(Economy.ItemType):getItemType()
            local askPrice = ask:get(Economy.Price):getPrice()
            local askQty = ask:get(Economy.Quantity):getQuantity()
            local askOwner = ask:get(Economy.Ownership):getOwner()

            if askTag:hasTag("Contraband") then goto continueAsk end
            if bidItemType ~= askItemType then goto continueAsk end
            if bidPrice < askPrice then goto continueAsk end

            local tradeQty = math.min(bidQty, askQty)
            local tradePrice = self:calculateTradePrice(bidPrice, askPrice)

            table.insert(self.pendingTrades, {
                marketplace = marketplace,
                itemType = bidItemType,
                quantity = tradeQty,
                price = tradePrice,
                buyer = bidOwner,
                seller = askOwner
            })

            bidQty = bidQty - tradeQty
            askQty = askQty - tradeQty
            bid:get(Economy.Quantity):setQuantity(bidQty)
            ask:get(Economy.Quantity):setQuantity(askQty)

            if bidQty <= 0 then
                marketplace:removeBid(bid)
                Registry:destroyEntity(bid)
                break
            end
            if askQty <= 0 then
                marketplace:removeAsk(ask)
                Registry:destroyEntity(ask)
            end

            ::continueAsk::
        end
        ::continueBid::
    end
end

function MarketplaceSystem:processPendingTrades()
    if not self.pendingTrades or #self.pendingTrades == 0 then return end

    local successfulTotals = {}
    local now = TimeStamp.Now()

    for _, trade in ipairs(self.pendingTrades) do
        local sellerInv = Entity(trade.seller):get(CoreComponents.Parent):getParent():get(Economy.Inventory)
        local buyerInv = Entity(trade.buyer):get(CoreComponents.Parent):getParent():get(Economy.Inventory)

        local items = InventoryManager:take(sellerInv, trade.itemType, trade.quantity)
        if items then
            InventoryManager:put(buyerInv, items)
            self:transferCurrency(trade.buyer, trade.seller, trade.price * trade.quantity)

            local app = require("States.Application")
            app:onTransaction(trade)

            Log.Debug("[Transaction] %s bought %d x %s from %s for %d each (total: %d)",
                Entity(trade.buyer), trade.quantity, Items:getDefinition(trade.itemType).name,
                Entity(trade.seller), trade.price, trade.price * trade.quantity)

            local marketplaceId = tostring(trade.marketplace)
            self:recordTrade(marketplaceId, trade.itemType, trade.price, trade.quantity)

            -- Check for large movement using shiftingEq
            local startEq = Items:getDefinition(trade.itemType).startEquilibriumPrice or self.defaultStartPrice
            local shiftingEq = self.shiftingEquilibrium[marketplaceId] and self.shiftingEquilibrium[marketplaceId][trade.itemType] or startEq
            local deviation = math.abs(trade.price - shiftingEq) / shiftingEq
            self.pullTimeoutData[marketplaceId] = self.pullTimeoutData[marketplaceId] or {}
            local timeoutData = self.pullTimeoutData[marketplaceId][trade.itemType] or {}

            -- Only trigger timeout if not in cooldown
            if not timeoutData.cooldownUntil or now:getDifference(timeoutData.cooldownUntil) <= 0 then
                if deviation >= self.largeMovementThreshold then
                    timeoutData.timestamp = now
                    timeoutData.cooldownUntil = TimeStamp.GetFuture(self.pullTimeout + self.timeoutCooldown)
                    self.pullTimeoutData[marketplaceId][trade.itemType] = timeoutData
                    Log.Debug(
                        "[MarketplaceSystem] Large movement detected for %s: price=%d, deviation=%.2f%% from shiftingEq=%.2f, pull timeout started",
                        Items:getDefinition(trade.itemType).name, trade.price, deviation * 100, shiftingEq)
                end
            end

            successfulTotals[marketplaceId] = successfulTotals[marketplaceId] or {}
            local entry = successfulTotals[marketplaceId][trade.itemType] or { quantity = 0, totalPrice = 0 }
            entry.quantity = entry.quantity + trade.quantity
            entry.totalPrice = entry.totalPrice + (trade.price * trade.quantity)
            successfulTotals[marketplaceId][trade.itemType] = entry
        end
    end

    for marketplaceId, items in pairs(successfulTotals) do
        local marketplace = nil
        for m in Registry:iterEntities(Economy.Marketplace) do
            if tostring(m) == marketplaceId then
                marketplace = m
                break
            end
        end
        if not marketplace then goto continueMarketplace end

        for itemType, data in pairs(items) do
            local avgPrice = math.floor(data.totalPrice / data.quantity)
            if marketplace.setMarketPrice then
                marketplace:setMarketPrice(itemType, avgPrice)
            end

            -- Update shiftingEquilibriumPrice
            self.shiftingEquilibrium[marketplaceId] = self.shiftingEquilibrium[marketplaceId] or {}
            local shiftingEq = self.shiftingEquilibrium[marketplaceId][itemType] or
                (Items:getDefinition(itemType).startEquilibriumPrice or self.defaultStartPrice)
            local historicalAvg = self:getHistoricalAverage(marketplaceId, itemType) or shiftingEq
            shiftingEq = (self.shiftAlpha * historicalAvg) + ((1 - self.shiftAlpha) * shiftingEq)
            self.shiftingEquilibrium[marketplaceId][itemType] = shiftingEq
        end
        ::continueMarketplace::
    end

    self.pendingTrades = {}
end

---@param bidPrice integer
---@param askPrice integer
function MarketplaceSystem:calculateTradePrice(bidPrice, askPrice)
    return math.floor((bidPrice + askPrice) / 2)
end

---@param marketplaceId integer
---@param itemType integer
---@param price integer
---@param quantity integer
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

    while #history > self.priceMemory do
        table.remove(history, 1)
    end
end

---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:updateMarketPrices(marketplace, bids, asks)
    local marketplaceId = tostring(marketplace)
    local itemTypes = self:getActiveItemTypes(bids, asks)

    for itemType, _ in pairs(itemTypes) do
        local supplyDemand = self:calculateSupplyDemand(bids, asks, itemType)
        local marketPrice = self:calculateEmergentPrice(marketplace, itemType, bids, asks, supplyDemand)
        if marketplace.setMarketPrice then
            marketplace:setMarketPrice(itemType, marketPrice)
        end
    end
end

---@param bids Entity[]
---@param asks Entity[]
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

---@param bids Entity[]
---@param asks Entity[]
---@param itemType integer
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
    if totalSupply == 0 then return 2.0 end
    if totalDemand == 0 then return 0.5 end
    return totalDemand / totalSupply
end

---@param marketplace MarketplaceComponent
---@param itemType integer
---@param bids Entity[]
---@param asks Entity[]
---@param supplyDemandRatio number
function MarketplaceSystem:calculateEmergentPrice(marketplace, itemType, bids, asks, supplyDemandRatio)
    local marketplaceId = tostring(marketplace)
    self.shiftingEquilibrium[marketplaceId] = self.shiftingEquilibrium[marketplaceId] or {}
    if not self.shiftingEquilibrium[marketplaceId][itemType] then
        self.shiftingEquilibrium[marketplaceId][itemType] = Items:getDefinition(itemType).startEquilibriumPrice or self.defaultStartPrice
    end
    local shiftingEq = self.shiftingEquilibrium[marketplaceId][itemType]
    local startEq = Items:getDefinition(itemType).startEquilibriumPrice or self.defaultStartPrice

    local bestBid = self:getBestBidPrice(bids, itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplaceId, itemType)
    local tradeCount = self:getTradeCount(marketplaceId, itemType)

    local basePrice
    if historicalPrice and tradeCount >= self.priceDiscoveryPeriod then
        basePrice = historicalPrice
    elseif bestBid and bestAsk then
        basePrice = (bestBid + bestAsk) / 2
    elseif bestBid then
        basePrice = bestBid
    elseif bestAsk then
        basePrice = bestAsk
    elseif historicalPrice then
        basePrice = historicalPrice
    else
        basePrice = startEq
    end

    local sdRatio = supplyDemandRatio
    if sdRatio < 0.2 then sdRatio = 0.2 end
    if sdRatio > 5.0 then sdRatio = 5.0 end

    local demandFactor
    if sdRatio > 1.0 then
        demandFactor = 1.0 + math.log(sdRatio) * 0.2
    else
        demandFactor = 1.0 / (1.0 + math.log(1 / sdRatio) * 0.2)
    end

    local targetPrice = basePrice * demandFactor

    -- Check for pull timeout
    local pullFactor = 1.0
    self.pullTimeoutData[marketplaceId] = self.pullTimeoutData[marketplaceId] or {}
    local timeoutData = self.pullTimeoutData[marketplaceId][itemType] or {}

    if timeoutData.timestamp then
        local secondsSinceMovement = timeoutData.timestamp:getElapsed()
        local logInterval = 0
        if secondsSinceMovement < self.pullTimeout then
            pullFactor = 0.0
            local timeoutRemaining = self.pullTimeout - secondsSinceMovement
            if timeoutRemaining > 60 then
                logInterval = 300             -- Log every 5 minutes when >60s remaining
            elseif timeoutRemaining > 30 then
                logInterval = 30              -- Log every 30s when >30s remaining
            else
                logInterval = self.updateRate -- Log every frame (0.01s) when <=30s
            end
            if self.logThrottle == nil or self.logThrottle:getElapsed() >= logInterval then
                Log.Debug("[MarketplaceSystem] Pull disabled for %s (timeout remaining: %.1fs)",
                    Items:getDefinition(itemType).name, timeoutRemaining)
                self.logThrottle = TimeStamp.Now()
            end
        elseif secondsSinceMovement < self.pullTimeout + self.timeoutCooldown then
            local timeSinceTimeout = secondsSinceMovement - self.pullTimeout
            pullFactor = 1.0 - math.exp(-self.pullRecoveryRate * timeSinceTimeout)
            local cooldownRemaining = (self.pullTimeout + self.timeoutCooldown) - secondsSinceMovement
            if cooldownRemaining > 60 then
                logInterval = 300             -- Log every 5 minutes when >60s remaining
            elseif cooldownRemaining > 30 then
                logInterval = 30              -- Log every 30s when >30s remaining
            else
                logInterval = self.updateRate -- Log every frame (0.01s) when <=30s
            end
            if self.logThrottle == nil or self.logThrottle:getElapsed() >= logInterval then
                Log.Debug("[MarketplaceSystem] Exponential pull for %s: pullFactor=%.3f (time since timeout: %.1fs)",
                    Items:getDefinition(itemType).name, pullFactor, timeSinceTimeout)
                self.logThrottle = TimeStamp.Now()
            end
        end
    end

    -- Apply pulls towards equilibria
    if pullFactor > 0 then
        local deviationStart = targetPrice - startEq
        local pullStart = pullFactor * self.pullStrength * 0.3 * (1 - math.exp(-math.abs(deviationStart) / self.pullScale)) *
            (deviationStart > 0 and -1 or 1)
        local deviationShift = targetPrice - shiftingEq
        local pullShift = pullFactor * self.pullStrength * 0.7 * (1 - math.exp(-math.abs(deviationShift) / self.pullScale)) *
            (deviationShift > 0 and -1 or 1)
        targetPrice = targetPrice + pullStart + pullShift
    end

    if historicalPrice then
        local maxChange = 0.1
        if sdRatio > 2.0 or sdRatio < 0.5 then
            maxChange = 0.25
        end
        local logHist = math.log(historicalPrice + 1)
        local logTarget = math.log(targetPrice + 1)
        logTarget = math.max(logHist - math.log(1 + maxChange), math.min(logHist + math.log(1 + maxChange), logTarget))
        targetPrice = math.exp(logTarget) - 1
    end

    local maturity = math.min(1.0, tradeCount / self.priceDiscoveryPeriod)
    local vol = self.volatility * (1.0 - maturity * 0.5)
    targetPrice = targetPrice * math.exp(self.rng:getUniformRange(-vol, vol))
    targetPrice = math.max(self.minPrice, math.floor(targetPrice))

    return targetPrice
end

---@param bids Entity[]
---@param itemType integer
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

---@param asks Entity[]
---@param itemType integer
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

---@param marketplaceId integer
---@param itemType integer
function MarketplaceSystem:getTradeCount(marketplaceId, itemType)
    if not self.priceHistory[marketplaceId] then return 0 end
    if not self.priceHistory[marketplaceId][itemType] then return 0 end
    return #self.priceHistory[marketplaceId][itemType]
end

---@param marketplaceId integer
---@param itemType integer
function MarketplaceSystem:getHistoricalAverage(marketplaceId, itemType)
    if not self.priceHistory[marketplaceId] then return nil end
    if not self.priceHistory[marketplaceId][itemType] then return nil end

    local history = self.priceHistory[marketplaceId][itemType]
    if #history == 0 then return nil end

    local totalPrice = 0
    local totalWeight = 0
    for i, trade in ipairs(history) do
        local recencyWeight = i / #history
        local volumeWeight = math.log(trade.quantity + 1)
        local weight = recencyWeight * volumeWeight
        totalPrice = totalPrice + (trade.price * weight)
        totalWeight = totalWeight + weight
    end
    return totalWeight > 0 and (totalPrice / totalWeight) or nil
end

---@param marketplace MarketplaceComponent
---@param itemType integer
function MarketplaceSystem:getSuggestedBidPrice(marketplace, itemType)
    local marketplaceId = tostring(marketplace)
    local bids = self:sortBids(marketplace:getBids())
    local asks = self:sortAsks(marketplace:getAsks())
    local emergentPrice = marketplace:getMarketPrice(itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplaceId, itemType)
    local startEq = Items:getDefinition(itemType).startEquilibriumPrice or self.defaultStartPrice
    local basePrice
    if emergentPrice then
        basePrice = math.floor(emergentPrice * 0.98)
    elseif bestAsk then
        basePrice = math.floor(bestAsk * 0.98)
    elseif historicalPrice then
        basePrice = math.floor(historicalPrice)
    else
        basePrice = math.floor(startEq * 0.95)
    end
    local conditions = self:analyzeMarketConditions(marketplaceId, itemType, bids, asks)
    local suggestedPrice = self:applyMarketModifiers(conditions, basePrice)
    return math.max(self.minPrice, suggestedPrice)
end

---@param marketplace MarketplaceComponent
---@param itemType integer
function MarketplaceSystem:getSuggestedAskPrice(marketplace, itemType)
    local marketplaceId = tostring(marketplace)
    local bids = self:sortBids(marketplace:getBids())
    local asks = self:sortAsks(marketplace:getAsks())
    local emergentPrice = marketplace:getMarketPrice(itemType)
    local bestBid = self:getBestBidPrice(bids, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplaceId, itemType)
    local startEq = Items:getDefinition(itemType).startEquilibriumPrice or self.defaultStartPrice
    local basePrice
    if emergentPrice then
        basePrice = math.ceil(emergentPrice * 1.02)
    elseif bestBid then
        basePrice = math.ceil(bestBid * 1.02)
    elseif historicalPrice then
        basePrice = math.ceil(historicalPrice)
    else
        basePrice = math.ceil(startEq * 1.05)
    end
    local conditions = self:analyzeMarketConditions(marketplaceId, itemType, bids, asks)
    local suggestedPrice = self:applyMarketModifiers(conditions, basePrice)
    return math.max(self.minPrice, suggestedPrice)
end

---@param marketplace MarketplaceComponent
---@param itemType integer
function MarketplaceSystem:getMarketDepth(marketplace, itemType)
    local bids = marketplace:getBids()
    local asks = marketplace:getAsks()
    local bidDepth = {}
    local askDepth = {}
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

---@param fromEntity EntityId
---@param toEntity EntityId
function MarketplaceSystem:transferCurrency(fromEntity, toEntity, amount)
    local fromWallet = Entity(fromEntity):get(CoreComponents.Parent):getParent():get(Economy.Wallet)
    local toWallet = Entity(toEntity):get(CoreComponents.Parent):getParent():get(Economy.Wallet)
    if fromWallet and toWallet then
        fromWallet:subtract(amount)
        toWallet:add(amount)
    end
end

---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:updateMarketConditionTags(marketplace, bids, asks)
    local marketplaceId = tostring(marketplace)
    local itemTypes = self:getActiveItemTypes(bids, asks)
    for itemType, _ in pairs(itemTypes) do
        local conditions = self:analyzeMarketConditions(marketplaceId, itemType, bids, asks)
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

---@param marketplaceId integer
---@param itemType integer
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:analyzeMarketConditions(marketplaceId, itemType, bids, asks)
    local conditions = {
        highDemand = false,
        lowSupply = false,
        volatile = false,
        lowDemand = false,
        highSupply = false
    }
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
    if totalSupply > 0 and totalDemand / totalSupply > 1.5 then
        conditions.highDemand = true
    elseif totalSupply == 0 and totalDemand > 0 then
        conditions.highDemand = true
        conditions.lowSupply = true
    elseif totalDemand / totalSupply < 0.75 then
        conditions.lowDemand = true
    end
    if totalDemand > 0 and totalSupply / totalDemand < 0.5 then
        conditions.lowSupply = true
    elseif totalSupply / totalDemand > 1.5 then
        conditions.highSupply = true
    end
    if self.priceHistory[marketplaceId] and self.priceHistory[marketplaceId][itemType] then
        local history = self.priceHistory[marketplaceId][itemType]
        if #history >= 5 then
            local priceVariance = self:calculatePriceVariance(history)
            local avgPrice = self:getHistoricalAverage(marketplaceId, itemType)
            if avgPrice and priceVariance / avgPrice > 0.15 then
                conditions.volatile = true
            end
        end
    end
    return conditions
end

---@param history table[]
function MarketplaceSystem:calculatePriceVariance(history)
    if #history < 2 then return 0 end
    local sum = 0
    for _, trade in ipairs(history) do
        sum = sum + trade.price
    end
    local mean = sum / #history
    local varianceSum = 0
    for _, trade in ipairs(history) do
        local diff = trade.price - mean
        varianceSum = varianceSum + (diff * diff)
    end
    return math.sqrt(varianceSum / #history)
end

---@param order Entity
---@param conditions table
function MarketplaceSystem:applyConditionTags(order, conditions)
    local tagComponent = order:get(CoreComponents.Tag)
    if not tagComponent then return end
    tagComponent:removeTag("HighDemand")
    tagComponent:removeTag("LowSupply")
    tagComponent:removeTag("Volatile")
    tagComponent:removeTag("LowDemand")
    tagComponent:removeTag("HighSupply")
    if conditions.highDemand then tagComponent:addTag("HighDemand") end
    if conditions.lowSupply then tagComponent:addTag("LowSupply") end
    if conditions.volatile then tagComponent:addTag("Volatile") end
    if conditions.lowDemand then tagComponent:addTag("LowDemand") end
    if conditions.highSupply then tagComponent:addTag("HighSupply") end
end

---@param conditions table
---@param price integer
function MarketplaceSystem:applyMarketModifiers(conditions, price)
    local modifier = 0
    if conditions.highDemand then modifier = modifier + 0.08 end
    if conditions.lowSupply then modifier = modifier + 0.05 end
    if conditions.volatile then modifier = modifier + 0.03 end
    if conditions.lowDemand then modifier = modifier - 0.05 end
    if conditions.highSupply then modifier = modifier - 0.03 end
    modifier = math.max(-0.15, math.min(0.15, modifier))
    local basePrice = math.log(price + 1)
    local adjusted = basePrice + modifier
    local newPrice = math.exp(adjusted) - 1
    return math.max(self.minPrice, math.floor(newPrice))
end

return MarketplaceSystem()
