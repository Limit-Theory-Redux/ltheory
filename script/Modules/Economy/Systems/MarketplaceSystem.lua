local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local Items = require("Shared.Registries.Items")
local QuickProfiler = require("Shared.Tools.QuickProfiler")

local Economy = require("Modules.Economy.Components")
local EconomyEntities = require("Modules.Economy.Entities")
local CoreComponents = require("Modules.Core.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")

-- Design: doc/script/economy.md

---@class MarketQuote
---@field bestBid integer|nil The highest bid price
---@field bidQuantity integer The quantity bid at `bestBid`
---@field bestAsk integer|nil The lowest ask price
---@field askQuantity integer The quantity asked at `bestAsk`
---@field demand integer Every bid's quantity together
---@field supply integer Every ask's quantity together

---Matches the orders of every market and settles the trades, and keeps each item's emergent price.
---An order holds its escrow from when it is posted: an ask the goods, taken out of the issuer's
---inventory, a bid its quantity times its price in credits. A trade is at the middle of the two
---prices; the buyer gets the goods and the difference to its bid back, the seller the credits. An
---order that is cancelled or expires gives back what is left of its escrow.
---The system keeps its own clock of simulated seconds (advanced at PreSim, so it stops while the
---game is paused); order expiry and the price model run on it.
---@class MarketplaceSystem
local MarketplaceSystem = Class("MarketplaceSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function MarketplaceSystem:registerVars()
    self.profiler = QuickProfiler("MarketplaceSystem", false, false)
    self.rng = RNG.FromTime()
    self.time = 0
    -- Seconds between two updates of a market, and the random extra that spreads markets over frames.
    self.updateRate = 1
    self.maxUpdateRateDeviation = 2

    -- Economic parameters
    self.priceMemory = 1000
    self.volatility = 0.05
    self.priceDiscoveryPeriod = 1000
    self.defaultStartPrice = 100
    self.minPrice = 1

    -- Pull & timeout parameters
    self.pullStrength = 0.1
    self.pullScale = 100
    self.shiftAlpha = 0.01
    self.pullTimeout = 7200
    self.largeMovementThreshold = 2.0
    self.timeoutCooldown = 3600
    self.pullRecoveryRate = 0.0001
end

function MarketplaceSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function MarketplaceSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---Advances the clock and updates the markets that are due. A game that runs frames gets this at
---PreSim; an app that steps the simulation itself calls it.
---@param dt number Seconds
function MarketplaceSystem:update(dt)
    self.time = self.time + dt
    self.profiler:start()
    for entity, marketplace in Registry:iterEntities(Economy.Marketplace) do
        ---@cast marketplace MarketplaceComponent
        local due = marketplace:getNextUpdate()
        if not due then
            marketplace:setNextUpdate(self.time + self.rng:getUniformRange(0, self.updateRate + self.maxUpdateRateDeviation))
        elseif self.time >= due then
            self:updateMarketplace(marketplace)
            marketplace:setNextUpdate(self.time + self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation))
        end
    end
    self.profiler:stop()
end

---@return number now Simulated seconds of the economy clock
function MarketplaceSystem:now()
    return self.time
end

---@param market Entity|MarketplaceComponent
---@return MarketplaceComponent|nil
local function marketplaceOf(market)
    if market.getBids then return market end
    return market:get(Economy.Marketplace)
end

---@param order Entity
---@return Entity|nil holder The order's issuer, when it still exists
local function holderOf(order)
    local holder = Entity(order:get(Economy.Ownership):getOwner())
    if not Registry:hasEntity(holder) then return nil end
    return holder
end

---@param holder Entity|nil
---@return InventoryComponent|nil
local function inventoryOf(holder)
    return holder and holder:get(Economy.Inventory) or nil
end

-- Orders -----------------------------------------------------------------------------------------

---Posts a bid: the issuer buys up to `quantity` units at `price` or less. Its credits are held until
---the bid fills, is cancelled or expires.
---@param market Entity|MarketplaceComponent
---@param holder Entity The issuer: an entity with an inventory and an account (its own or its owner's)
---@param itemId integer
---@param quantity integer
---@param price integer Per unit
---@param ttl number|nil Seconds until it expires; nil never expires
---@return Entity|nil order Nil when the issuer can't pay for it or has no inventory
function MarketplaceSystem:postBid(market, holder, itemId, quantity, price, ttl)
    local marketplace = marketplaceOf(market)
    quantity, price = math.floor(quantity), math.max(self.minPrice, math.floor(price))
    if not marketplace or quantity <= 0 or not inventoryOf(holder) then return nil end
    if not AccountManager:withdraw(AccountManager:accountOf(holder), quantity * price) then return nil end

    local order = EconomyEntities.Order(holder, Items:getDefinition(itemId), quantity, price, ttl and self.time + ttl)
    order:get(CoreComponents.Tag):addTag("BidOrder")
    order:get(Economy.OrderStatus):setStatus(Enums.OrderStatus.Open)
    marketplace:addBid(order)
    return order
end

---Posts an ask: the issuer sells up to `quantity` units at `price` or more. The goods leave its
---inventory until the ask fills, is cancelled or expires.
---@param market Entity|MarketplaceComponent
---@param holder Entity The issuer: an entity with an inventory
---@param itemId integer
---@param quantity integer
---@param price integer Per unit
---@param ttl number|nil Seconds until it expires; nil never expires
---@return Entity|nil order Nil when the issuer doesn't hold the goods
function MarketplaceSystem:postAsk(market, holder, itemId, quantity, price, ttl)
    local marketplace = marketplaceOf(market)
    quantity, price = math.floor(quantity), math.max(self.minPrice, math.floor(price))
    local inventory = inventoryOf(holder)
    if not marketplace or quantity <= 0 or not inventory then return nil end
    if not InventoryManager:remove(inventory, itemId, quantity) then return nil end

    local order = EconomyEntities.Order(holder, Items:getDefinition(itemId), quantity, price, ttl and self.time + ttl)
    order:get(CoreComponents.Tag):addTag("AskOrder")
    order:get(Economy.OrderStatus):setStatus(Enums.OrderStatus.Open)
    marketplace:addAsk(order)
    return order
end

---Gives back what is left of an order's escrow to its issuer, takes it off the market and destroys it.
---@param marketplace MarketplaceComponent
---@param order Entity
---@param status OrderStatus The order's end
function MarketplaceSystem:closeOrder(marketplace, order, status)
    local holder = holderOf(order)
    local itemId = order:get(Economy.Item):getItem()
    local remaining = order:get(Economy.Quantity):getQuantity()
    if marketplace:removeBid(order) then
        if holder and remaining > 0 then
            AccountManager:deposit(AccountManager:accountOf(holder), remaining * order:get(Economy.Price):getPrice())
        end
    elseif marketplace:removeAsk(order) then
        local inventory = inventoryOf(holder)
        if inventory and remaining > 0 and not InventoryManager:add(inventory, itemId, remaining) then
            -- The issuer has filled the room the goods left since; they are lost.
            Log.Warn("[MarketplaceSystem] %s has no room for %d returned %s", tostring(holder), remaining,
                Items:getDefinition(itemId).name)
        end
    end
    order:get(Economy.OrderStatus):setStatus(status)
    Registry:destroyEntity(order)
end

---Cancels an open order, giving back what is left of its escrow.
---@param market Entity|MarketplaceComponent
---@param order Entity
---@return boolean cancelled False when the order is not open on this market
function MarketplaceSystem:cancelOrder(market, order)
    local marketplace = marketplaceOf(market)
    if not marketplace or not Registry:hasEntity(order) then return false end
    if not marketplace:hasBid(order) and not marketplace:hasAsk(order) then return false end
    self:closeOrder(marketplace, order, Enums.OrderStatus.Cancelled)
    return true
end

---Cancels every open order of an issuer on the market.
---@param market Entity|MarketplaceComponent
---@param holder Entity
function MarketplaceSystem:cancelOrdersOf(market, holder)
    local marketplace = marketplaceOf(market)
    if not marketplace then return end
    for _, order in ipairs(self:getOrdersOf(marketplace, holder)) do
        self:closeOrder(marketplace, order, Enums.OrderStatus.Cancelled)
    end
end

---@param market Entity|MarketplaceComponent
---@param holder Entity
---@return Entity[] orders The issuer's open bids and asks
function MarketplaceSystem:getOrdersOf(market, holder)
    local marketplace = marketplaceOf(market)
    local orders = {}
    if not marketplace then return orders end
    for _, set in ipairs({ marketplace:getBids(), marketplace:getAsks() }) do
        for _, order in ipairs(set) do
            if order:get(Economy.Ownership):getOwner() == holder.id then orders[#orders + 1] = order end
        end
    end
    return orders
end

---The credits an issuer's open bids on the market hold: its own money, out of its account until the
---bids fill or come back.
---@param market Entity|MarketplaceComponent
---@param holder Entity
---@return integer credits
function MarketplaceSystem:creditsHeldBy(market, holder)
    local marketplace = marketplaceOf(market)
    local held = 0
    if not marketplace then return held end
    for _, order in ipairs(marketplace:getBids()) do
        if order:get(Economy.Ownership):getOwner() == holder.id then
            held = held + order:get(Economy.Quantity):getQuantity() * order:get(Economy.Price):getPrice()
        end
    end
    return held
end

---The goods of an item an issuer's open asks on the market hold: its own, out of its inventory until
---the asks fill or come back.
---@param market Entity|MarketplaceComponent
---@param holder Entity
---@param itemId integer
---@return integer quantity
function MarketplaceSystem:goodsHeldBy(market, holder, itemId)
    local marketplace = marketplaceOf(market)
    local held = 0
    if not marketplace then return held end
    for _, order in ipairs(marketplace:getAsks()) do
        if order:get(Economy.Ownership):getOwner() == holder.id and order:get(Economy.Item):getItem() == itemId then
            held = held + order:get(Economy.Quantity):getQuantity()
        end
    end
    return held
end

---Buys at once from the market's asks: up to `quantity` units at `maxPrice` or less, as far as the
---buyer can pay and has room. The trades are settled before it returns.
---@param market Entity|MarketplaceComponent
---@param buyer Entity
---@param itemId integer
---@param quantity integer
---@param maxPrice integer
---@return integer bought, number spent
function MarketplaceSystem:buyNow(market, buyer, itemId, quantity, maxPrice)
    local inventory = inventoryOf(buyer)
    if not inventory then return 0, 0 end
    maxPrice = math.floor(maxPrice)
    local affordable = maxPrice > 0 and math.floor(AccountManager:balanceOf(buyer) / maxPrice) or 0
    quantity = math.min(quantity, affordable, InventoryManager:getRoomFor(inventory, itemId))
    if quantity <= 0 then return 0, 0 end

    local before, balance = InventoryManager:count(inventory, itemId), AccountManager:balanceOf(buyer)
    local order = self:postBid(market, buyer, itemId, quantity, maxPrice)
    if not order then return 0, 0 end
    self:matchItem(marketplaceOf(market), itemId)
    if Registry:hasEntity(order) then self:cancelOrder(market, order) end
    return InventoryManager:count(inventory, itemId) - before, balance - AccountManager:balanceOf(buyer)
end

---Sells at once to the market's bids: up to `quantity` units at `minPrice` or more. The trades are
---settled before it returns.
---@param market Entity|MarketplaceComponent
---@param seller Entity
---@param itemId integer
---@param quantity integer
---@param minPrice integer
---@return integer sold, number earned
function MarketplaceSystem:sellNow(market, seller, itemId, quantity, minPrice)
    local inventory = inventoryOf(seller)
    if not inventory then return 0, 0 end
    quantity = math.min(quantity, InventoryManager:count(inventory, itemId))
    if quantity <= 0 then return 0, 0 end

    local before, balance = InventoryManager:count(inventory, itemId), AccountManager:balanceOf(seller)
    local order = self:postAsk(market, seller, itemId, quantity, minPrice)
    if not order then return 0, 0 end
    self:matchItem(marketplaceOf(market), itemId)
    if Registry:hasEntity(order) then self:cancelOrder(market, order) end
    return before - InventoryManager:count(inventory, itemId), AccountManager:balanceOf(seller) - balance
end

-- Matching ---------------------------------------------------------------------------------------

---@param marketplace MarketplaceComponent
function MarketplaceSystem:updateMarketplace(marketplace)
    self:removeEndedOrders(marketplace)
    local itemTypes = self:getActiveItemTypes(marketplace:getBids(), marketplace:getAsks())
    for itemType in pairs(itemTypes) do
        self:matchItem(marketplace, itemType)
    end
    local bids, asks = marketplace:getBids(), marketplace:getAsks()
    self:updateMarketConditionTags(marketplace, bids, asks)
    self:updateMarketPrices(marketplace, bids, asks)
    marketplace:setLastUpdated(self.time)
end

---Closes the orders that have expired or whose issuer is gone.
---@param marketplace MarketplaceComponent
function MarketplaceSystem:removeEndedOrders(marketplace)
    for _, set in ipairs({ marketplace:getBids(), marketplace:getAsks() }) do
        for _, order in ipairs(set) do
            if not Registry:hasEntity(order) then
                marketplace:removeBid(order)
                marketplace:removeAsk(order)
            elseif not holderOf(order) then
                self:closeOrder(marketplace, order, Enums.OrderStatus.Invalid)
            elseif order:get(Economy.Expiry):isExpired(self.time) then
                self:closeOrder(marketplace, order, Enums.OrderStatus.Expired)
            end
        end
    end
end

---@param orders Entity[]
---@param itemType integer
---@param descending boolean
---@return Entity[] sorted The orders of the item by price, the older first at one price
local function sortedOrders(orders, itemType, descending)
    local sorted = {}
    for _, order in ipairs(orders) do
        if Registry:hasEntity(order) and order:get(Economy.Item):getItem() == itemType then
            sorted[#sorted + 1] = order
        end
    end
    table.sort(sorted, function(a, b)
        local priceA, priceB = a:get(Economy.Price):getPrice(), b:get(Economy.Price):getPrice()
        if priceA ~= priceB then
            if descending then return priceA > priceB end
            return priceA < priceB
        end
        return a.id < b.id
    end)
    return sorted
end

---@param bids Entity[]
function MarketplaceSystem:sortBids(bids, itemType)
    return sortedOrders(bids, itemType, true)
end

---@param asks Entity[]
function MarketplaceSystem:sortAsks(asks, itemType)
    return sortedOrders(asks, itemType, false)
end

---Matches the bids and asks of one item, best prices first, and settles each trade. A bid and an
---ask of the same issuer don't trade, nor do contraband orders here.
---@param marketplace MarketplaceComponent
---@param itemType integer
function MarketplaceSystem:matchItem(marketplace, itemType)
    local bids = self:sortBids(marketplace:getBids(), itemType)
    local asks = self:sortAsks(marketplace:getAsks(), itemType)
    self.matched = { quantity = 0, value = 0 }

    for _, bid in ipairs(bids) do
        if bid:get(CoreComponents.Tag):hasTag("Contraband") then goto continueBid end
        local buyer = holderOf(bid)
        local buyerInventory = inventoryOf(buyer)
        if not buyerInventory then goto continueBid end

        for _, ask in ipairs(asks) do
            if not Registry:hasEntity(ask) or ask:get(CoreComponents.Tag):hasTag("Contraband") then goto continueAsk end
            local bidQuantity = bid:get(Economy.Quantity):getQuantity()
            local askQuantity = ask:get(Economy.Quantity):getQuantity()
            local bidPrice, askPrice = bid:get(Economy.Price):getPrice(), ask:get(Economy.Price):getPrice()
            -- The asks are sorted: none after this one is cheap enough.
            if bidPrice < askPrice then break end
            local seller = holderOf(ask)
            if not seller or seller.id == buyer.id then goto continueAsk end

            local quantity = math.min(bidQuantity, askQuantity, InventoryManager:getRoomFor(buyerInventory, itemType))
            if quantity <= 0 then break end
            self:settle(marketplace, bid, ask, buyer, seller, buyerInventory, itemType, quantity,
                self:calculateTradePrice(bidPrice, askPrice))

            if ask:get(Economy.Quantity):getQuantity() <= 0 then
                self:closeOrder(marketplace, ask, Enums.OrderStatus.Filled)
            end
            if bid:get(Economy.Quantity):getQuantity() <= 0 then
                self:closeOrder(marketplace, bid, Enums.OrderStatus.Filled)
                break
            end

            ::continueAsk::
        end
        ::continueBid::
    end
    if self.matched.quantity > 0 then
        self:updateAfterTrades(marketplace, itemType, self.matched.value / self.matched.quantity)
    end
end

---Settles one trade out of the two orders' escrow.
---@param marketplace MarketplaceComponent
---@param bid Entity
---@param ask Entity
---@param buyer Entity
---@param seller Entity
---@param buyerInventory InventoryComponent
---@param itemType integer
---@param quantity integer
---@param price integer
function MarketplaceSystem:settle(marketplace, bid, ask, buyer, seller, buyerInventory, itemType, quantity, price)
    InventoryManager:add(buyerInventory, itemType, quantity)
    AccountManager:deposit(AccountManager:accountOf(seller), price * quantity)
    AccountManager:deposit(AccountManager:accountOf(buyer), (bid:get(Economy.Price):getPrice() - price) * quantity)

    for _, order in ipairs({ bid, ask }) do
        local quantityComponent = order:get(Economy.Quantity)
        quantityComponent:setQuantity(quantityComponent:getQuantity() - quantity)
        order:get(Economy.OrderStatus):setStatus(Enums.OrderStatus.PartiallyFilled)
    end

    self:recordTrade(marketplace, itemType, price, quantity)
end

---@param bidPrice integer
---@param askPrice integer
function MarketplaceSystem:calculateTradePrice(bidPrice, askPrice)
    return math.floor((bidPrice + askPrice) / 2)
end

---@param itemType integer
---@return number
function MarketplaceSystem:getStartPrice(itemType)
    local definition = Items:getDefinition(itemType)
    return definition and definition.startEquilibriumPrice or self.defaultStartPrice
end

---Records a trade in the market's history, and starts a pull timeout on a large movement away from
---the shifting equilibrium.
---@param marketplace MarketplaceComponent
---@param itemType integer
---@param price integer
---@param quantity integer
function MarketplaceSystem:recordTrade(marketplace, itemType, price, quantity)
    marketplace:recordTrade(itemType, { price = price, quantity = quantity, time = self.time }, self.priceMemory)
    self.matched.quantity = self.matched.quantity + quantity
    self.matched.value = self.matched.value + price * quantity

    local shiftingEq = marketplace:getShiftingEquilibrium(itemType) or self:getStartPrice(itemType)
    local deviation = math.abs(price - shiftingEq) / shiftingEq
    local timeout = marketplace:getPullTimeout(itemType)
    if deviation >= self.largeMovementThreshold and (not timeout or self.time >= timeout.cooldownUntil) then
        marketplace:setPullTimeout(itemType, {
            startedAt = self.time,
            cooldownUntil = self.time + self.pullTimeout + self.timeoutCooldown,
        })
    end
end

---After a match that traded: the market price is the trades' average price, and the shifting
---equilibrium moves a step toward the history's average.
---@param marketplace MarketplaceComponent
---@param itemType integer
---@param averagePrice number
function MarketplaceSystem:updateAfterTrades(marketplace, itemType, averagePrice)
    marketplace:setMarketPrice(itemType, math.floor(averagePrice))
    local shiftingEq = marketplace:getShiftingEquilibrium(itemType) or self:getStartPrice(itemType)
    local historicalAvg = self:getHistoricalAverage(marketplace, itemType) or shiftingEq
    marketplace:setShiftingEquilibrium(itemType, self.shiftAlpha * historicalAvg + (1 - self.shiftAlpha) * shiftingEq)
end

-- Prices -----------------------------------------------------------------------------------------

---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:updateMarketPrices(marketplace, bids, asks)
    for itemType in pairs(self:getActiveItemTypes(bids, asks)) do
        local supplyDemand = self:calculateSupplyDemand(bids, asks, itemType)
        marketplace:setMarketPrice(itemType, self:calculateEmergentPrice(marketplace, itemType, bids, asks, supplyDemand))
    end
end

---@param bids Entity[]
---@param asks Entity[]
---@return table<integer, boolean>
function MarketplaceSystem:getActiveItemTypes(bids, asks)
    local itemTypes = {}
    for _, set in ipairs({ bids, asks }) do
        for _, order in ipairs(set) do
            if Registry:hasEntity(order) then itemTypes[order:get(Economy.Item):getItem()] = true end
        end
    end
    return itemTypes
end

---@param orders Entity[]
---@param itemType integer
---@return integer quantity
local function totalQuantity(orders, itemType)
    local total = 0
    for _, order in ipairs(orders) do
        if Registry:hasEntity(order) and order:get(Economy.Item):getItem() == itemType then
            total = total + order:get(Economy.Quantity):getQuantity()
        end
    end
    return total
end

---@param bids Entity[]
---@param asks Entity[]
---@param itemType integer
---@return number ratio Demand over supply; 2 without supply, 0.5 without demand
function MarketplaceSystem:calculateSupplyDemand(bids, asks, itemType)
    local totalDemand = totalQuantity(bids, itemType)
    local totalSupply = totalQuantity(asks, itemType)
    if totalSupply == 0 then return 2.0 end
    if totalDemand == 0 then return 0.5 end
    return totalDemand / totalSupply
end

---@param marketplace MarketplaceComponent
---@param itemType integer
---@return number pullFactor 0 during a pull timeout, recovering during its cooldown, else 1
function MarketplaceSystem:getPullFactor(marketplace, itemType)
    local timeout = marketplace:getPullTimeout(itemType)
    if not timeout then return 1.0 end
    local sinceMovement = self.time - timeout.startedAt
    if sinceMovement < self.pullTimeout then return 0.0 end
    if sinceMovement < self.pullTimeout + self.timeoutCooldown then
        return 1.0 - math.exp(-self.pullRecoveryRate * (sinceMovement - self.pullTimeout))
    end
    return 1.0
end

---The item's price from the market's book, history and equilibria: a base price (the history once
---it is long enough, else the best prices), moved by the ratio of demand to supply, pulled toward
---the start and the shifting equilibrium, clamped to a band around the history, with noise that
---lessens as the market matures.
---@param marketplace MarketplaceComponent
---@param itemType integer
---@param bids Entity[]
---@param asks Entity[]
---@param supplyDemandRatio number
---@return integer price
function MarketplaceSystem:calculateEmergentPrice(marketplace, itemType, bids, asks, supplyDemandRatio)
    local startEq = self:getStartPrice(itemType)
    local shiftingEq = marketplace:getShiftingEquilibrium(itemType)
    if not shiftingEq then
        shiftingEq = startEq
        marketplace:setShiftingEquilibrium(itemType, shiftingEq)
    end

    local bestBid = self:getBestBidPrice(bids, itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplace, itemType)
    local tradeCount = self:getTradeCount(marketplace, itemType)

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

    local sdRatio = math.max(0.2, math.min(5.0, supplyDemandRatio))
    local demandFactor
    if sdRatio > 1.0 then
        demandFactor = 1.0 + math.log(sdRatio) * 0.2
    else
        demandFactor = 1.0 / (1.0 + math.log(1 / sdRatio) * 0.2)
    end
    local targetPrice = basePrice * demandFactor

    local pullFactor = self:getPullFactor(marketplace, itemType)
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
        local maxChange = (sdRatio > 2.0 or sdRatio < 0.5) and 0.25 or 0.1
        local logHist = math.log(historicalPrice + 1)
        local logTarget = math.log(targetPrice + 1)
        logTarget = math.max(logHist - math.log(1 + maxChange), math.min(logHist + math.log(1 + maxChange), logTarget))
        targetPrice = math.exp(logTarget) - 1
    end

    local maturity = math.min(1.0, tradeCount / self.priceDiscoveryPeriod)
    local vol = self.volatility * (1.0 - maturity * 0.5)
    targetPrice = targetPrice * math.exp(self.rng:getUniformRange(-vol, vol))
    return math.max(self.minPrice, math.floor(targetPrice))
end

---@param bids Entity[]
---@param itemType integer
---@return integer|nil
function MarketplaceSystem:getBestBidPrice(bids, itemType)
    local sorted = self:sortBids(bids, itemType)
    return sorted[1] and sorted[1]:get(Economy.Price):getPrice()
end

---@param asks Entity[]
---@param itemType integer
---@return integer|nil
function MarketplaceSystem:getBestAskPrice(asks, itemType)
    local sorted = self:sortAsks(asks, itemType)
    return sorted[1] and sorted[1]:get(Economy.Price):getPrice()
end

---@param marketplace MarketplaceComponent
---@param itemType integer
---@return integer
function MarketplaceSystem:getTradeCount(marketplace, itemType)
    return #marketplace:getHistory(itemType)
end

---The history's average price, weighting recent and large trades more.
---@param marketplace MarketplaceComponent
---@param itemType integer
---@return number|nil
function MarketplaceSystem:getHistoricalAverage(marketplace, itemType)
    local history = marketplace:getHistory(itemType)
    if #history == 0 then return nil end

    local totalPrice = 0
    local totalWeight = 0
    for i, trade in ipairs(history) do
        local weight = (i / #history) * math.log(trade.quantity + 1)
        totalPrice = totalPrice + trade.price * weight
        totalWeight = totalWeight + weight
    end
    return totalWeight > 0 and (totalPrice / totalWeight) or nil
end

---The best bid and ask of an item, with their quantities, and the whole demand and supply. The
---orders of `exclude` (an issuer looking at the market) are left out.
---@param market Entity|MarketplaceComponent
---@param itemType integer
---@param exclude Entity|nil
---@return MarketQuote
function MarketplaceSystem:quote(market, itemType, exclude)
    local marketplace = marketplaceOf(market)
    local quote = { bidQuantity = 0, askQuantity = 0, demand = 0, supply = 0 }
    if not marketplace then return quote end
    local function scan(orders, best, quantity, total, better)
        for _, order in ipairs(orders) do
            if Registry:hasEntity(order) and order:get(Economy.Item):getItem() == itemType
                and not (exclude and order:get(Economy.Ownership):getOwner() == exclude.id) then
                local price = order:get(Economy.Price):getPrice()
                local q = order:get(Economy.Quantity):getQuantity()
                quote[total] = quote[total] + q
                if not quote[best] or better(price, quote[best]) then
                    quote[best], quote[quantity] = price, q
                elseif price == quote[best] then
                    quote[quantity] = quote[quantity] + q
                end
            end
        end
    end
    scan(marketplace:getBids(), "bestBid", "bidQuantity", "demand", function(a, b) return a > b end)
    scan(marketplace:getAsks(), "bestAsk", "askQuantity", "supply", function(a, b) return a < b end)
    return quote
end

---A bid price that should trade soon: a little under the market price, moved by the conditions.
---@param market Entity|MarketplaceComponent
---@param itemType integer
---@return integer
function MarketplaceSystem:getSuggestedBidPrice(market, itemType)
    local marketplace = marketplaceOf(market)
    local bids, asks = marketplace:getBids(), marketplace:getAsks()
    local emergentPrice = marketplace:getMarketPrice(itemType)
    local bestAsk = self:getBestAskPrice(asks, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplace, itemType)
    local basePrice
    if emergentPrice then
        basePrice = math.floor(emergentPrice * 0.98)
    elseif bestAsk then
        basePrice = math.floor(bestAsk * 0.98)
    elseif historicalPrice then
        basePrice = math.floor(historicalPrice)
    else
        basePrice = math.floor(self:getStartPrice(itemType) * 0.95)
    end
    local conditions = self:analyzeMarketConditions(marketplace, itemType, bids, asks)
    return math.max(self.minPrice, self:applyMarketModifiers(conditions, basePrice))
end

---An ask price that should trade soon: a little over the market price, moved by the conditions.
---@param market Entity|MarketplaceComponent
---@param itemType integer
---@return integer
function MarketplaceSystem:getSuggestedAskPrice(market, itemType)
    local marketplace = marketplaceOf(market)
    local bids, asks = marketplace:getBids(), marketplace:getAsks()
    local emergentPrice = marketplace:getMarketPrice(itemType)
    local bestBid = self:getBestBidPrice(bids, itemType)
    local historicalPrice = self:getHistoricalAverage(marketplace, itemType)
    local basePrice
    if emergentPrice then
        basePrice = math.ceil(emergentPrice * 1.02)
    elseif bestBid then
        basePrice = math.ceil(bestBid * 1.02)
    elseif historicalPrice then
        basePrice = math.ceil(historicalPrice)
    else
        basePrice = math.ceil(self:getStartPrice(itemType) * 1.05)
    end
    local conditions = self:analyzeMarketConditions(marketplace, itemType, bids, asks)
    return math.max(self.minPrice, self:applyMarketModifiers(conditions, basePrice))
end

---@param market Entity|MarketplaceComponent
---@param itemType integer
---@return table<integer, integer> bidDepth, table<integer, integer> askDepth Price -> quantity
function MarketplaceSystem:getMarketDepth(market, itemType)
    local marketplace = marketplaceOf(market)
    local function depth(orders)
        local out = {}
        for _, order in ipairs(orders) do
            if Registry:hasEntity(order) and order:get(Economy.Item):getItem() == itemType then
                local price = order:get(Economy.Price):getPrice()
                out[price] = (out[price] or 0) + order:get(Economy.Quantity):getQuantity()
            end
        end
        return out
    end
    return depth(marketplace:getBids()), depth(marketplace:getAsks())
end

-- Conditions -------------------------------------------------------------------------------------

---@param marketplace MarketplaceComponent
---@param bids Entity[]
---@param asks Entity[]
function MarketplaceSystem:updateMarketConditionTags(marketplace, bids, asks)
    for itemType in pairs(self:getActiveItemTypes(bids, asks)) do
        local conditions = self:analyzeMarketConditions(marketplace, itemType, bids, asks)
        marketplace:setConditions(itemType, conditions)
        for _, set in ipairs({ bids, asks }) do
            for _, order in ipairs(set) do
                if Registry:hasEntity(order) and order:get(Economy.Item):getItem() == itemType then
                    self:applyConditionTags(order, conditions)
                end
            end
        end
    end
end

---@param marketplace MarketplaceComponent
---@param itemType integer
---@param bids Entity[]
---@param asks Entity[]
---@return MarketConditions
function MarketplaceSystem:analyzeMarketConditions(marketplace, itemType, bids, asks)
    local conditions = {
        highDemand = false,
        lowSupply = false,
        volatile = false,
        lowDemand = false,
        highSupply = false
    }
    local totalDemand = totalQuantity(bids, itemType)
    local totalSupply = totalQuantity(asks, itemType)
    if totalSupply == 0 then
        if totalDemand > 0 then
            conditions.highDemand = true
            conditions.lowSupply = true
        end
    elseif totalDemand == 0 then
        conditions.lowDemand = true
        conditions.highSupply = true
    else
        local ratio = totalDemand / totalSupply
        if ratio > 1.5 then conditions.highDemand = true elseif ratio < 0.75 then conditions.lowDemand = true end
        if ratio > 2.0 then conditions.lowSupply = true elseif ratio < 1 / 1.5 then conditions.highSupply = true end
    end

    local history = marketplace:getHistory(itemType)
    if #history >= 5 then
        local avgPrice = self:getHistoricalAverage(marketplace, itemType)
        if avgPrice and avgPrice > 0 and self:calculatePriceVariance(history) / avgPrice > 0.15 then
            conditions.volatile = true
        end
    end
    return conditions
end

---@param history MarketTrade[]
---@return number stddev
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
        varianceSum = varianceSum + diff * diff
    end
    return math.sqrt(varianceSum / #history)
end

---@param order Entity
---@param conditions MarketConditions
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

---@param conditions MarketConditions
---@param price integer
---@return integer
function MarketplaceSystem:applyMarketModifiers(conditions, price)
    local modifier = 0
    if conditions.highDemand then modifier = modifier + 0.08 end
    if conditions.lowSupply then modifier = modifier + 0.05 end
    if conditions.volatile then modifier = modifier + 0.03 end
    if conditions.lowDemand then modifier = modifier - 0.05 end
    if conditions.highSupply then modifier = modifier - 0.03 end
    modifier = math.max(-0.15, math.min(0.15, modifier))
    local newPrice = math.exp(math.log(price + 1) + modifier) - 1
    return math.max(self.minPrice, math.floor(newPrice))
end

return MarketplaceSystem()
