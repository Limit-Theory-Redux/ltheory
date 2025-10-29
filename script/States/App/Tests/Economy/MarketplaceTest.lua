local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")

local EconomyComponents = require("Modules.Economy.Components")
local EconomyEntities = require("Modules.Economy.Entities")
local ConstructsEntities = require("Modules.Constructs.Entities")
local CoreEntities = require("Modules.Core.Entities")
local CoreComponents = require("Modules.Core.Components")

require("Shared.Definitions.ItemDefs")
local Items = require("Shared.Registries.Items")
local Tags = require("Shared.Registries.Tags")

local MarketplaceTest = require('States.Application')
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")

-- ANSI color codes
local GREEN = "\27[32m"
local RED = "\27[31m"
local GRAY = "\27[37m"
local WHITE = "\27[97m"
local RESET = "\27[0m"

local SELECTED_TEST_SCENARIO = "GoldSupplyDrop"

-- TODO: BREAK THIS TEST UP INTO MANAGERS, HELPERS, ETC

---@diagnostic disable-next-line
function MarketplaceTest:onInit()
    Log.Info("[MarketplaceTest] Setting up comprehensive marketplace test")

    -- Choose scenario
    self.selectedScenario = SELECTED_TEST_SCENARIO

    -- Tag setup first
    Tags:new("TradeType", { "BidOrder", "AskOrder", "Contract", "Auction", "Brokered" })
    Tags:new("Goods", { "HighDemand", "LowDemand", "HighSupply", "LowSupply", "Volatile", "Luxury", "Consumable", "RawMaterial" })
    Tags:new("Legality", { "Legal", "Restricted", "Contraband" })
    Tags:new("Context", { "OrgTrade", "MissionTrade", "EventTrade" })

    -- Create marketplace
    local trader = CoreEntities.Player("Station Trader", true)
    local station = ConstructsEntities.SpaceStation(1)
    local marketplace = station:get(EconomyComponents.Marketplace)
    marketplace:setTrader(trader)

    -- Create multiple traders
    self.traders = {}
    self.ships = {}

    for i = 1, 5 do
        local player = CoreEntities.Player("Trader_" .. i, true)
        local ship = ConstructsEntities.Spaceship(i)
        Registry:attachEntity(ship, player)

        -- Give each trader credits and inventory
        local inv = ship:get(EconomyComponents.Inventory)
        inv:addItem(EconomyEntities.Item(Items.Virtual.Credit, 5e6))
        self.gold = 1e8
        self.infowafers = 1e8
        self.iron = 1e8
        inv:addItem(EconomyEntities.Item(Items.RefinedMaterials.Gold, self.gold))
        inv:addItem(EconomyEntities.Item(Items.Data.InfoWafer, self.infowafers))
        inv:addItem(EconomyEntities.Item(Items.RawMaterials.IronOre, self.iron))

        table.insert(self.traders, player)
        table.insert(self.ships, ship)
    end

    -- Store marketplace for updates
    self.marketplace = marketplace
    self.marketplaceId = tostring(marketplace)

    -- Test configuration
    self.orderExpiryTime = 600
    self.testDuration = 30
    self.orderInterval = 0.01
    self.supplyRestockInterval = 500
    self.lastOrderTime = TimeStamp.Now()
    self.lastRestockTime = TimeStamp.Now()
    self.startTime = TimeStamp.Now()
    self.orderCounter = 0
    self.testComplete = false
    self.printedReport = false

    -- Available items for trading
    self.tradableItems = {
        Items.RefinedMaterials.Gold,
        Items.Data.InfoWafer,
        Items.RawMaterials.IronOre
    }

    -- Statistics tracking
    self.stats = {
        ordersCreated = 0,
        tradesExecuted = 0,
        totalVolume = 0,
        pricesByItem = {
            [Items.RefinedMaterials.Gold.id] = Items:getDefinition(Items.RefinedMaterials.Gold.id).startEquilibriumPrice,
            [Items.Data.InfoWafer.id] = Items:getDefinition(Items.Data.InfoWafer.id).startEquilibriumPrice,
            [Items.RawMaterials.IronOre.id] = Items:getDefinition(Items.RawMaterials.IronOre.id).startEquilibriumPrice
        },
        priceHistory = {},
        supplyDemandHistory = {},
        tagHistory = {},
        timeoutHistory = {} -- Track pull timeouts per item
    }

    -- Time tracking for snapshots
    self.lastSnapshotTime = TimeStamp.Now()
    self.snapshotInterval = 30.0

    -- Run scenario if selected
    if self.selectedScenario ~= nil then
        self:runScenario(self.selectedScenario)
    end

    -- Create initial orders
    self:createRandomOrders(10)

    Log.Info("[MarketplaceTest] Test initialized with %d traders", #self.traders)
    Log.Info("[MarketplaceTest] Will run for %d seconds, creating orders every %d seconds",
        self.testDuration, self.orderInterval)
end

-- Scenario definitions
MarketplaceTest.Scenarios = {}

MarketplaceTest.Scenarios.GoldSupplyDrop = function(self)
    Log.Info("[Scenario] Gold supply drop triggered")
    for _, ship in ipairs(self.ships) do
        local inv = ship:get(EconomyComponents.Inventory)
        local goldStack = inv:getItemsOfType(Items.RefinedMaterials.Gold)
        for goldItem in Iterator(goldStack) do
            local qty = goldItem:get(EconomyComponents.Quantity):getQuantity()
            goldItem:get(EconomyComponents.Quantity):setQuantity(math.floor(qty * 0.5))
            if goldItem:get(EconomyComponents.Quantity):getQuantity() <= 0 then
                inv:removeItem(goldItem)
            end
        end
    end
    local startPrice = Items:getDefinition(Items.RefinedMaterials.Gold.id).startEquilibriumPrice
    local highPrice = math.ceil(startPrice * 1.5)
    for i = 1, 3 do
        local trader = self.traders[i]
        local order = EconomyEntities.Order(trader, Items.RefinedMaterials.Gold, 5, highPrice, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag("AskOrder")
        tagComp:addTag("LowSupply")
        self.marketplace:addAsk(order)
    end
end

MarketplaceTest.Scenarios.InfoWaferFlood = function(self)
    Log.Info("[Scenario] InfoWafer oversupply triggered")
    for _, ship in ipairs(self.ships) do
        local inv = ship:get(EconomyComponents.Inventory)
        inv:addItem(EconomyEntities.Item(Items.Data.InfoWafer, 2000))
    end
    local startPrice = Items:getDefinition(Items.Data.InfoWafer.id).startEquilibriumPrice
    local lowPrice = math.floor(startPrice * 0.2)
    for i = 1, 5 do
        local trader = self.traders[i]
        local order = EconomyEntities.Order(trader, Items.Data.InfoWafer, 200, lowPrice, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag("AskOrder")
        tagComp:addTag("HighSupply")
        self.marketplace:addAsk(order)
    end
end

MarketplaceTest.Scenarios.IronOreVolatility = function(self)
    Log.Info("[Scenario] IronOre volatility spike triggered")
    local startPrice = Items:getDefinition(Items.RawMaterials.IronOre.id).startEquilibriumPrice
    for i = 1, 8 do
        local trader = self.traders[(i % #self.traders) + 1]
        local qty = math.random(100, 300)
        local priceVariation = math.random(50, 200) / 100
        local price = math.floor(startPrice * priceVariation)
        local orderType = (i % 2 == 0) and "AskOrder" or "BidOrder"
        local order = EconomyEntities.Order(trader, Items.RawMaterials.IronOre, qty, price, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag(orderType)
        tagComp:addTag("Volatile")
        if orderType == "BidOrder" then
            self.marketplace:addBid(order)
        else
            self.marketplace:addAsk(order)
        end
    end
end

MarketplaceTest.Scenarios.PriceShockPullTest = function(self)
    Log.Info("[Scenario] Price shock pull test triggered")
    local item = Items.RefinedMaterials.Gold
    local startPrice = Items:getDefinition(item.id).startEquilibriumPrice
    for _, ship in ipairs(self.ships) do
        local inv = ship:get(EconomyComponents.Inventory)
        local goldStack = inv:getItemsOfType(item)
        for goldItem in Iterator(goldStack) do
            local qty = goldItem:get(EconomyComponents.Quantity):getQuantity()
            goldItem:get(EconomyComponents.Quantity):setQuantity(math.floor(qty * 0.5))
            if goldItem:get(EconomyComponents.Quantity):getQuantity() <= 0 then
                inv:removeItem(goldItem)
            end
        end
    end
    local shockPrice = math.ceil(startPrice * 1.3)
    for i = 1, 5 do
        local trader = self.traders[i]
        local order = EconomyEntities.Order(trader, item, 50, shockPrice, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag("BidOrder")
        tagComp:addTag("HighDemand")
        tagComp:addTag("LowSupply")
        self.marketplace:addBid(order)
    end
    local askPrice = math.ceil(startPrice * 1.4)
    for i = 1, 2 do
        local trader = self.traders[(i % #self.traders) + 1]
        local order = EconomyEntities.Order(trader, item, 10, askPrice, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag("AskOrder")
        tagComp:addTag("LowSupply")
        self.marketplace:addAsk(order)
    end
end

MarketplaceTest.Scenarios.ControlledPriceShockTest = function(self)
    Log.Info("[Scenario] Controlled price shock test triggered")
    local item = Items.RefinedMaterials.Gold
    local startPrice = Items:getDefinition(item.id).startEquilibriumPrice -- 200
    -- Moderate supply reduction (30%)
    for _, ship in ipairs(self.ships) do
        local inv = ship:get(EconomyComponents.Inventory)
        local goldStack = inv:getItemsOfType(item)
        for goldItem in Iterator(goldStack) do
            local qty = goldItem:get(EconomyComponents.Quantity):getQuantity()
            goldItem:get(EconomyComponents.Quantity):setQuantity(math.floor(qty * 0.7))
            if goldItem:get(EconomyComponents.Quantity):getQuantity() <= 0 then
                inv:removeItem(goldItem)
            end
        end
    end
    -- Moderate initial price shock
    local shockPrice = math.ceil(startPrice * 1.2) -- 240
    for i = 1, 5 do
        local trader = self.traders[i]
        local order = EconomyEntities.Order(trader, item, 50, shockPrice, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag("BidOrder")
        tagComp:addTag("HighDemand")
        tagComp:addTag("LowSupply")
        self.marketplace:addBid(order)
    end
    local askPrice = math.ceil(startPrice * 1.3) -- 260
    for i = 1, 2 do
        local trader = self.traders[(i % #self.traders) + 1]
        local order = EconomyEntities.Order(trader, item, 10, askPrice, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag("AskOrder")
        tagComp:addTag("LowSupply")
        self.marketplace:addAsk(order)
    end
end

function MarketplaceTest:runScenario(scenarioName)
    local scenario = self.Scenarios[scenarioName]
    if not scenario then
        Log.Warn("[MarketplaceTest] Scenario '%s' not found", scenarioName)
        return
    end
    Log.Info("[MarketplaceTest] Running scenario: %s", scenarioName)
    scenario(self)
end

function MarketplaceTest:onTransaction(event)
    self.stats.tradesExecuted = self.stats.tradesExecuted + 1
    self.stats.totalVolume = self.stats.totalVolume + (event.quantity * event.price)
    local itemName = Items:getDefinition(event.itemType).name
    if not self.stats.pricesByItem[itemName] then
        self.stats.pricesByItem[itemName] = {}
    end
    table.insert(self.stats.pricesByItem[itemName], event.price)
    if not self.stats.priceHistory[itemName] then
        self.stats.priceHistory[itemName] = {}
    end
    table.insert(self.stats.priceHistory[itemName], {
        price = event.price,
        quantity = event.quantity,
        timestamp = math.abs(TimeStamp.Now():getDifference(self.startTime))
    })
    -- Check for pull timeout on trade
    local marketplaceId = self.marketplaceId
    local timeoutData = MarketplaceSystem.pullTimeoutData[marketplaceId] and
        MarketplaceSystem.pullTimeoutData[marketplaceId][event.itemType]
    if timeoutData and timeoutData.timestamp then
        local secondsRemaining = MarketplaceSystem.pullTimeout - TimeStamp.Now():getDifference(timeoutData.timestamp)
        if secondsRemaining > 0 then
            if not self.stats.timeoutHistory[itemName] then
                self.stats.timeoutHistory[itemName] = {}
            end
            table.insert(self.stats.timeoutHistory[itemName], {
                timestamp = math.abs(TimeStamp.Now():getDifference(self.startTime)),
                duration = MarketplaceSystem.pullTimeout
            })
        end
    end
    Log.Info("[Trade #%d] %s: %d x %s @ %d credits (buyer: %s, seller: %s)",
        self.stats.tradesExecuted,
        itemName,
        event.quantity,
        itemName,
        event.price,
        Entity(event.buyer),
        Entity(event.seller))
end

function MarketplaceTest:createRandomOrders(count)
    self.stats.ordersCreated = self.stats.ordersCreated + count
    for _ = 1, count do
        local trader = self.traders[math.random(#self.traders)]
        local item = self.tradableItems[math.random(#self.tradableItems)]
        local isBid = math.random() > 0.5
        local suggestedPrice
        if isBid then
            suggestedPrice = MarketplaceSystem:getSuggestedBidPrice(self.marketplace, item.id)
        else
            suggestedPrice = MarketplaceSystem:getSuggestedAskPrice(self.marketplace, item.id)
        end
        local qty = math.random(10, 100)
        local isGold = item == Items.RefinedMaterials.Gold
        local isInfoWafer = item == Items.Data.InfoWafer
        local isIronOre = item == Items.RawMaterials.IronOre
        if self.selectedScenario == "GoldSupplyDrop" and isGold then
            if isBid then
                qty = math.random(5, 15)
            else
                if math.random() < 0.2 then
                    qty = math.random(1, 3)
                else
                    goto continue
                end
            end
        elseif self.selectedScenario == "InfoWaferFlood" and isInfoWafer then
            if isBid then
                qty = math.random(5, 20)
            else
                qty = math.random(50, 150)
            end
        elseif self.selectedScenario == "IronOreVolatility" and isIronOre then
            qty = math.random(10, 200)
        elseif (self.selectedScenario == "PriceShockPullTest" or self.selectedScenario == "ControlledPriceShockTest") and isGold then
            if isBid then
                qty = math.random(20, 50)
            else
                qty = math.random(5, 15)
            end
        end
        local priceVariation = math.random(95, 105) / 100
        local price = math.max(1, math.floor(suggestedPrice * priceVariation))
        local order = EconomyEntities.Order(trader, item, qty, price, TimeStamp.GetFuture(self.orderExpiryTime))
        local tagComp = order:get(CoreComponents.Tag)
        tagComp:addTag(isBid and "BidOrder" or "AskOrder")
        if self.selectedScenario == "GoldSupplyDrop" and isGold then
            tagComp:addTag(isBid and "HighDemand" or "LowSupply")
        elseif self.selectedScenario == "InfoWaferFlood" and isInfoWafer then
            tagComp:addTag(isBid and "LowDemand" or "HighSupply")
        elseif self.selectedScenario == "IronOreVolatility" and isIronOre then
            tagComp:addTag("Volatile")
        elseif (self.selectedScenario == "PriceShockPullTest" or self.selectedScenario == "ControlledPriceShockTest") and isGold then
            tagComp:addTag(isBid and "HighDemand" or "LowSupply")
        end
        if isBid then
            self.marketplace:addBid(order)
        else
            self.marketplace:addAsk(order)
        end
        ::continue::
    end
end

---@param e EventData
function MarketplaceTest:onPreRender(e)
    if self.testComplete then
        if not self.printedReport then
            self.printedReport = true
            Log.Info("\27[92m[MarketplaceTest] Test complete. Generating final report...\27[0m")
            self:printFinalReport()
        end
        self:quit()
        return
    end
    local now = TimeStamp.Now()
    local elapsed = self.startTime:getElapsed()
    self.lastTimePrint = self.lastTimePrint or now
    if self.lastTimePrint:getDifference(now) >= 10 then
        self.lastTimePrint = now
        Log.Debug("\27[92m[MarketplaceTest] Elapsed time: %.1fs\27[0m", elapsed)
        Log.Debug("\27[92m[MarketplaceTest] Time left: %.1fs\27[0m", math.max(0, self.testDuration - elapsed))
    end
    if elapsed >= self.testDuration then
        self.testComplete = true
        return
    end
    if (self.selectedScenario == "GoldSupplyDrop" or self.selectedScenario == "PriceShockPullTest" or self.selectedScenario == "ControlledPriceShockTest") and self.lastRestockTime:getDifference(now) >= self.supplyRestockInterval then
        for _, ship in ipairs(self.ships) do
            local inv = ship:get(EconomyComponents.Inventory)
            inv:addItem(EconomyEntities.Item(Items.RefinedMaterials.Gold, math.random(10, 20)))
        end
        Log.Info("[MarketplaceTest] Restocked gold supply for traders")
        self.lastRestockTime = now
    end
    if self.lastSnapshotTime:getDifference(now) >= self.snapshotInterval then
        self:takeMarketSnapshot(elapsed)
        self.lastSnapshotTime = now
    end
    local timeSinceLastOrder = self.lastOrderTime:getDifference(now)
    if timeSinceLastOrder >= self.orderInterval and not self.testComplete then
        self:createRandomOrders(math.random(2, 5))
        self.lastOrderTime = now
    end
    local elapsedInt = math.floor(elapsed)
    if elapsedInt > 0 and elapsedInt % 30 == 0 and not self.lastReportTime then
        self:printInterimReport(elapsed)
        self.lastReportTime = elapsedInt
    elseif elapsedInt % 30 ~= 0 then
        self.lastReportTime = nil
    end
end

function MarketplaceTest:takeMarketSnapshot(timestamp)
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local conditions = self:analyzeItemMarket(item.id)
        if not self.stats.supplyDemandHistory[itemName] then
            self.stats.supplyDemandHistory[itemName] = {}
        end
        table.insert(self.stats.supplyDemandHistory[itemName], {
            timestamp = timestamp,
            supply = conditions.totalSupply,
            demand = conditions.totalDemand,
            bidCount = conditions.bidCount,
            askCount = conditions.askCount
        })
        if not self.stats.tagHistory[itemName] then
            self.stats.tagHistory[itemName] = {}
        end
        table.insert(self.stats.tagHistory[itemName], {
            timestamp = timestamp,
            tags = table.copy(conditions.tags)
        })
    end
end

function MarketplaceTest:printInterimReport(elapsed)
    Log.Info("========== INTERIM REPORT (t=%ds) ==========", math.floor(elapsed))
    Log.Info("Orders Created: %d", self.stats.ordersCreated)
    Log.Info("Trades Executed: %d", self.stats.tradesExecuted)
    Log.Info("Total Volume: %d credits", self.stats.totalVolume)
    local bids = self.marketplace:getBids()
    local asks = self.marketplace:getAsks()
    local bidCount = 0
    local askCount = 0
    for _ in Iterator(bids) do bidCount = bidCount + 1 end
    for _ in Iterator(asks) do askCount = askCount + 1 end
    Log.Info("Current Order Book: %d bids, %d asks", bidCount, askCount)
    Log.Info("--- Market State by Item ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local conditions = self:analyzeItemMarket(item.id)
        local priceStats = self:getPriceStats(itemName, elapsed)
        Log.Info("  %s:", itemName)
        Log.Info("    Order Book: %d bids, %d asks", conditions.bidCount, conditions.askCount)
        Log.Info("    Supply/Demand: %d / %d (ratio: %.2f)",
            conditions.totalSupply,
            conditions.totalDemand,
            conditions.totalDemand > 0 and conditions.totalDemand / math.max(conditions.totalSupply, 1) or 0)
        if priceStats then
            Log.Info("    Price: Current=%.2f, Avg=%.2f, Min=%d, Max=%d, Trend=%s",
                priceStats.current or 0,
                priceStats.average,
                priceStats.min,
                priceStats.max,
                priceStats.trend)
        end
        if #conditions.tags > 0 then
            Log.Info("    Active Tags: %s", table.concat(conditions.tags, ", "))
        end
        local shiftingEq = MarketplaceSystem.shiftingEquilibrium[self.marketplaceId] and
            MarketplaceSystem.shiftingEquilibrium[self.marketplaceId][item.id] or
            Items:getDefinition(item.id).startEquilibriumPrice
        Log.Info("    Equilibria: Start=%d, Shifting=%.2f",
            Items:getDefinition(item.id).startEquilibriumPrice, shiftingEq)
        local timeoutData = MarketplaceSystem.pullTimeoutData[self.marketplaceId] and
            MarketplaceSystem.pullTimeoutData[self.marketplaceId][item.id]
        if timeoutData and timeoutData.timestamp then
            local secondsRemaining = MarketplaceSystem.pullTimeout - TimeStamp.Now():getDifference(timeoutData.timestamp)
            if secondsRemaining > 0 then
                Log.Info("    Pull Timeout: Active (%.1fs remaining)", secondsRemaining)
            else
                local cooldownRemaining = timeoutData.cooldownUntil and
                    timeoutData.cooldownUntil:getDifference(TimeStamp.Now()) or 0
                if cooldownRemaining > 0 then
                    Log.Info("    Pull Timeout: In cooldown (%.1fs remaining)", cooldownRemaining)
                else
                    Log.Info("    Pull Timeout: Inactive")
                end
            end
        else
            Log.Info("    Pull Timeout: Inactive")
        end
    end
    Log.Info("=============================================")
end

function MarketplaceTest:analyzeItemMarket(itemId)
    local bids = self.marketplace:getBids()
    local asks = self.marketplace:getAsks()
    local bidCount = 0
    local askCount = 0
    local totalDemand = 0
    local totalSupply = 0
    local tags = {}
    for bid in Iterator(bids) do
        if bid:get(EconomyComponents.ItemType):getItemType() == itemId then
            bidCount = bidCount + 1
            totalDemand = totalDemand + bid:get(EconomyComponents.Quantity):getQuantity()
            local tagComp = bid:get(CoreComponents.Tag)
            if tagComp:hasTag("HighDemand") and not table.contains(tags, "HighDemand") then
                table.insert(tags, "HighDemand")
            end
            if tagComp:hasTag("LowSupply") and not table.contains(tags, "LowSupply") then
                table.insert(tags, "LowSupply")
            end
            if tagComp:hasTag("Volatile") and not table.contains(tags, "Volatile") then
                table.insert(tags, "Volatile")
            end
            if tagComp:hasTag("LowDemand") and not table.contains(tags, "LowDemand") then
                table.insert(tags, "LowDemand")
            end
            if tagComp:hasTag("HighSupply") and not table.contains(tags, "HighSupply") then
                table.insert(tags, "HighSupply")
            end
        end
    end
    for ask in Iterator(asks) do
        if ask:get(EconomyComponents.ItemType):getItemType() == itemId then
            askCount = askCount + 1
            totalSupply = totalSupply + ask:get(EconomyComponents.Quantity):getQuantity()
            local tagComp = ask:get(CoreComponents.Tag)
            if tagComp:hasTag("HighDemand") and not table.contains(tags, "HighDemand") then
                table.insert(tags, "HighDemand")
            end
            if tagComp:hasTag("LowSupply") and not table.contains(tags, "LowSupply") then
                table.insert(tags, "LowSupply")
            end
            if tagComp:hasTag("Volatile") and not table.contains(tags, "Volatile") then
                table.insert(tags, "Volatile")
            end
            if tagComp:hasTag("LowDemand") and not table.contains(tags, "LowDemand") then
                table.insert(tags, "LowDemand")
            end
            if tagComp:hasTag("HighSupply") and not table.contains(tags, "HighSupply") then
                table.insert(tags, "HighSupply")
            end
        end
    end
    return {
        bidCount = bidCount,
        askCount = askCount,
        totalDemand = totalDemand,
        totalSupply = totalSupply,
        tags = tags
    }
end

function MarketplaceTest:getPriceStats(itemName, currentTime)
    local history = self.stats.priceHistory[itemName]
    if not history or #history == 0 then
        return nil
    end
    local sum = 0
    local min = history[1].price
    local max = history[1].price
    local recentPrices = {}
    for i, trade in ipairs(history) do
        sum = sum + trade.price
        min = math.min(min, trade.price)
        max = math.max(max, trade.price)
        if i > #history - 5 then
            table.insert(recentPrices, trade.price)
        end
    end
    local average = sum / #history
    local trend = "stable"
    if #history >= 4 then
        local firstHalf = 0
        local secondHalf = 0
        local halfPoint = math.floor(#history / 2)
        for i = 1, halfPoint do
            firstHalf = firstHalf + history[i].price
        end
        for i = halfPoint + 1, #history do
            secondHalf = secondHalf + history[i].price
        end
        firstHalf = firstHalf / halfPoint
        secondHalf = secondHalf / (#history - halfPoint)
        local change = (secondHalf - firstHalf) / firstHalf
        if change > 0.1 then
            trend = "rising"
        elseif change < -0.1 then
            trend = "falling"
        end
    end
    return {
        current = recentPrices[#recentPrices],
        average = average,
        min = min,
        max = max,
        trend = trend
    }
end

function MarketplaceTest:printFinalReport()
    Log.Info("========== FINAL MARKETPLACE REPORT ==========")
    Log.Info("Test Duration: %d seconds", self.testDuration)
    Log.Info("Total Orders Created: %d", self.stats.ordersCreated)
    Log.Info("Total Trades Executed: %d", self.stats.tradesExecuted)
    Log.Info("Total Trading Volume: %d credits", self.stats.totalVolume)
    if self.stats.tradesExecuted > 0 then
        Log.Info("Average Trade Size: %d credits",
            self.stats.totalVolume / self.stats.tradesExecuted)
    end
    Log.Info("--- PRICE DEVELOPMENT OVER TIME ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        self:printPriceDevelopment(itemName)
    end
    Log.Info("--- PRICE ANALYSIS ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local history = self.stats.priceHistory[itemName]
        if history and #history > 0 then
            Log.Info("%s: %d trades", itemName, #history)
            local sum = 0
            local min = history[1].price
            local max = history[1].price
            for _, trade in ipairs(history) do
                sum = sum + trade.price
                min = math.min(min, trade.price)
                max = math.max(max, trade.price)
            end
            local avg = sum / #history
            local variance = 0
            for _, trade in ipairs(history) do
                variance = variance + (trade.price - avg) ^ 2
            end
            local stdDev = math.sqrt(variance / #history)
            local cv = (stdDev / avg) * 100
            Log.Info("  Overall: Avg=%.2f, Min=%d, Max=%d, StdDev=%.2f, CV=%.1f%%",
                avg, min, max, stdDev, cv)
            Log.Info("  Timeline:")
            local samplePoints = math.min(5, #history)
            for i = 1, samplePoints do
                local idx = math.floor((#history / samplePoints) * i)
                if idx > 0 and idx <= #history then
                    local trade = history[idx]
                    Log.Info("    t=%.1fs: %d credits (qty=%d)",
                        trade.timestamp, trade.price, trade.quantity)
                end
            end
            if cv > 20 then
                Log.Info("  Assessment: HIGHLY VOLATILE market")
            elseif cv > 10 then
                Log.Info("  Assessment: Moderately volatile market")
            else
                Log.Info("  Assessment: Stable market")
            end
            local shiftingEq = MarketplaceSystem.shiftingEquilibrium[self.marketplaceId] and
                MarketplaceSystem.shiftingEquilibrium[self.marketplaceId][item.id] or
                Items:getDefinition(item.id).startEquilibriumPrice
            Log.Info("  Final Equilibria: Start=%d, Shifting=%.2f",
                Items:getDefinition(item.id).startEquilibriumPrice, shiftingEq)
            local deviationStart = ((avg - Items:getDefinition(item.id).startEquilibriumPrice) / Items:getDefinition(item.id).startEquilibriumPrice) *
                100
            local deviationShift = ((avg - shiftingEq) / shiftingEq) * 100
            Log.Info("  Deviation: From Start=%.1f%%, From Shifting=%.1f%%",
                deviationStart, deviationShift)
            local timeoutHistory = self.stats.timeoutHistory[itemName] or {}
            local timeoutCount = #timeoutHistory
            local totalTimeoutDuration = 0
            for _, timeout in ipairs(timeoutHistory) do
                totalTimeoutDuration = totalTimeoutDuration + timeout.duration
            end
            Log.Info("  Pull Timeouts: %d occurrences, Total Duration=%.1fs (%.1f%% of test)",
                timeoutCount, totalTimeoutDuration, (totalTimeoutDuration / self.testDuration) * 100)
        else
            Log.Info("%s: No trades", itemName)
        end
    end
    Log.Info("--- SUPPLY/DEMAND EVOLUTION ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local sdHistory = self.stats.supplyDemandHistory[itemName]
        if sdHistory and #sdHistory > 0 then
            Log.Info("%s:", itemName)
            local snapshots = { 1, math.floor(#sdHistory / 2), #sdHistory }
            for _, idx in ipairs(snapshots) do
                if sdHistory[idx] then
                    local snap = sdHistory[idx]
                    local ratio = snap.demand > 0 and snap.demand / math.max(snap.supply, 1) or 0
                    Log.Info("  t=%.1fs: Supply=%d, Demand=%d (ratio=%.2f) | Orders: %d bids, %d asks",
                        snap.timestamp, snap.supply, snap.demand, ratio,
                        snap.bidCount, snap.askCount)
                end
            end
        end
    end
    Log.Info("--- MARKET CONDITION TAGS OVER TIME ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local tagHist = self.stats.tagHistory[itemName]
        if tagHist and #tagHist > 0 then
            Log.Info("%s:", itemName)
            local lastTags = {}
            for _, snapshot in ipairs(tagHist) do
                local currentTags = table.toSet(snapshot.tags)
                local previousTags = table.toSet(lastTags)
                local newTags = {}
                for tag, _ in pairs(currentTags) do
                    if not previousTags[tag] then
                        table.insert(newTags, tag)
                    end
                end
                local removedTags = {}
                for tag, _ in pairs(previousTags) do
                    if not currentTags[tag] then
                        table.insert(removedTags, tag)
                    end
                end
                if #newTags > 0 then
                    Log.Info("  t=%.1fs: +%s", snapshot.timestamp, table.concat(newTags, ", "))
                end
                if #removedTags > 0 then
                    Log.Info("  t=%.1fs: -%s", snapshot.timestamp, table.concat(removedTags, ", "))
                end
                lastTags = snapshot.tags
            end
            if #lastTags > 0 then
                Log.Info("  Final: %s", table.concat(lastTags, ", "))
            else
                Log.Info("  Final: No condition tags")
            end
        end
    end
    Log.Info("--- FINAL ORDER BOOK ---")
    local bids = self.marketplace:getBids()
    local asks = self.marketplace:getAsks()
    local bidCount = 0
    local askCount = 0
    for _ in Iterator(bids) do bidCount = bidCount + 1 end
    for _ in Iterator(asks) do askCount = askCount + 1 end
    Log.Info("Remaining Orders: %d bids, %d asks", bidCount, askCount)
    Log.Info("==============================================")
    Log.Info("[MarketplaceTest] Test complete!")
end

function MarketplaceTest:printPriceDevelopment(itemName)
    local history = self.stats.priceHistory[itemName]
    if not history or #history == 0 then
        Log.Info("%s: No trade history", itemName)
        return
    end
    Log.Info("%s: %d trades", itemName, #history)
    local prices = {}
    local timestamps = {}
    local minPrice = math.huge
    local maxPrice = -math.huge
    for _, trade in ipairs(history) do
        table.insert(prices, trade.price)
        table.insert(timestamps, trade.timestamp)
        minPrice = math.min(minPrice, trade.price)
        maxPrice = math.max(maxPrice, trade.price)
    end
    local avgPrice = 0
    for _, price in ipairs(prices) do
        avgPrice = avgPrice + price
    end
    avgPrice = avgPrice / #prices
    Log.Info("  Stats: Min=%d, Max=%d, Avg=%.2f, Range=%d (%.1f%%)",
        minPrice, maxPrice, avgPrice, maxPrice - minPrice,
        ((maxPrice - minPrice) / avgPrice) * 100)
    Log.Info("  Timeline:")
    Log.Info("    Time(s) | Price | Qty | Change | Trend")
    Log.Info("    --------|-------|-----|--------|------")
    local lastPrice = nil
    for i, trade in ipairs(history) do
        local change = ""
        local trend = ""
        if lastPrice then
            local diff = trade.price - lastPrice
            local pct = (diff / lastPrice) * 100
            if diff > 0 then
                change = string.format("+%d (+%.1f%%)", diff, pct)
                trend = "↑"
            elseif diff < 0 then
                change = string.format("%d (%.1f%%)", diff, pct)
                trend = "↓"
            else
                change = "±0"
                trend = "→"
            end
        else
            change = "---"
            trend = "•"
        end
        Log.Info("    %7.1f | %5d | %3d | %12s | %s",
            trade.timestamp, trade.price, trade.quantity, change, trend)
        lastPrice = trade.price
    end
    Log.Info("%s (%ss timeline):", itemName, self.testDuration)
    self:printASCIIChart(history, minPrice, maxPrice)
    self:printCandlestickChart(history, minPrice, maxPrice)
    local trend = self:analyzePriceTrend(prices)
    Log.Info("        Trend Analysis: %s \n", trend)
end

function MarketplaceTest:printASCIIChart(history, minPrice, maxPrice)
    local chartWidth = 170
    local chartHeight = 16
    local priceRange = maxPrice - minPrice
    if priceRange == 0 then
        Log.Info("    Price remained constant at %d", minPrice)
        return
    end
    local minTime = math.huge
    local maxTime = -math.huge
    for _, trade in ipairs(history) do
        minTime = math.min(minTime, trade.timestamp)
        maxTime = math.max(maxTime, trade.timestamp)
    end
    local timeRange = maxTime - minTime
    if timeRange <= 0 then
        Log.Info("    All trades at same timestamp")
        return
    end
    local columns = {}
    for col = 1, chartWidth do
        columns[col] = {}
    end
    for _, trade in ipairs(history) do
        local normalizedTime = (trade.timestamp - minTime) / timeRange
        local col = math.floor(normalizedTime * (chartWidth - 1)) + 1
        col = math.max(1, math.min(chartWidth, col))
        table.insert(columns[col], trade.price)
    end
    local plotData = {}
    for col = 1, chartWidth do
        if #columns[col] > 0 then
            plotData[col] = columns[col]
        else
            plotData[col] = nil
        end
    end
    for row = chartHeight, 1, -1 do
        local line = "    "
        local priceLevel = minPrice + (priceRange * (row - 1) / (chartHeight - 1))
        line = line .. string.format("%5.0f |", priceLevel)
        for col = 1, chartWidth do
            local prices = plotData[col]
            if prices then
                local rowNormalized = (row - 1) / (chartHeight - 1)
                local hasPoint = false
                for _, price in ipairs(prices) do
                    local normalizedPrice = (price - minPrice) / priceRange
                    if math.abs(normalizedPrice - rowNormalized) < (1.0 / chartHeight) then
                        hasPoint = true
                        break
                    end
                end
                if hasPoint then
                    line = line .. "●"
                elseif prices[1] and ((prices[1] - minPrice) / priceRange) >= rowNormalized then
                    line = line .. "│"
                else
                    line = line .. " "
                end
            else
                line = line .. " "
            end
        end
        Log.Info(line)
    end
    local xAxis = "          +"
    for i = 1, chartWidth do
        xAxis = xAxis .. "-"
    end
    Log.Info(xAxis)
    Log.Info("           %.1fs%s%.1fs",
        minTime,
        string.rep(" ", chartWidth - 15),
        maxTime)
    local totalTrades = #history
    local columnsWithData = 0
    local maxTradesPerColumn = 0
    for col = 1, chartWidth do
        if #columns[col] > 0 then
            columnsWithData = columnsWithData + 1
            maxTradesPerColumn = math.max(maxTradesPerColumn, #columns[col])
        end
    end
    Log.Info("           Data: %d trades across %d time buckets (max %d trades/bucket)",
        totalTrades, columnsWithData, maxTradesPerColumn)
end

function MarketplaceTest:printCandlestickChart(history, minPrice, maxPrice)
    local chartWidth = 170
    local chartHeight = 16
    local priceRange = maxPrice - minPrice
    if priceRange == 0 then
        Log.Info("    Price remained constant at %d", minPrice)
        return
    end
    local minTime = math.huge
    local maxTime = -math.huge
    for _, trade in ipairs(history) do
        minTime = math.min(minTime, trade.timestamp)
        maxTime = math.max(maxTime, trade.timestamp)
    end
    local timeRange = maxTime - minTime
    if timeRange <= 0 then return end
    local buckets = {}
    for col = 1, chartWidth do
        buckets[col] = { prices = {}, first = nil, last = nil }
    end
    for _, trade in ipairs(history) do
        local normalizedTime = (trade.timestamp - minTime) / timeRange
        local col = math.floor(normalizedTime * (chartWidth - 1)) + 1
        col = math.max(1, math.min(chartWidth, col))
        table.insert(buckets[col].prices, { price = trade.price, timestamp = trade.timestamp })
        if not buckets[col].first then
            buckets[col].first = trade.price
        end
        buckets[col].last = trade.price
    end
    local candlesticks = {}
    for col = 1, chartWidth do
        if #buckets[col].prices > 0 then
            table.sort(buckets[col].prices, function(a, b) return a.timestamp < b.timestamp end)
            local prices = {}
            for _, entry in ipairs(buckets[col].prices) do
                table.insert(prices, entry.price)
            end
            table.sort(prices)
            candlesticks[col] = {
                open = buckets[col].first,
                high = prices[#prices],
                low = prices[1],
                close = buckets[col].last,
                count = #prices
            }
        end
    end
    for row = chartHeight, 1, -1 do
        local line = "    "
        local priceLevel = minPrice + (priceRange * (row - 1) / (chartHeight - 1))
        line = line .. string.format("%5.0f |", priceLevel)
        for col = 1, chartWidth do
            local candle = candlesticks[col]
            if candle then
                local normHigh = (candle.high - minPrice) / priceRange
                local normLow = (candle.low - minPrice) / priceRange
                local normOpen = (candle.open - minPrice) / priceRange
                local normClose = (candle.close - minPrice) / priceRange
                local rowNorm = (row - 1) / (chartHeight - 1)
                local bodyTop = math.max(normOpen, normClose)
                local bodyBottom = math.min(normOpen, normClose)
                if rowNorm <= normHigh and rowNorm >= bodyTop then
                    line = line .. WHITE .. "│" .. RESET
                elseif rowNorm <= bodyTop and rowNorm >= bodyBottom then
                    if candle.close > candle.open then
                        line = line .. GREEN .. "▓" .. RESET
                    elseif candle.close < candle.open then
                        line = line .. RED .. "░" .. RESET
                    else
                        line = line .. GRAY .. "─" .. RESET
                    end
                elseif rowNorm <= bodyBottom and rowNorm >= normLow then
                    line = line .. WHITE .. "│" .. RESET
                else
                    line = line .. " "
                end
            else
                line = line .. " "
            end
        end
        Log.Info(line)
    end
    local xAxis = "          +"
    for i = 1, chartWidth do
        xAxis = xAxis .. "-"
    end
    Log.Info(xAxis)
    Log.Info("%.1fs%s%.1fs", minTime, string.rep(" ", chartWidth - 15), maxTime)
    Log.Info("        Legend: " ..
        GREEN .. "▓" .. RESET .. "=Price Up, " .. RED .. "░" .. RESET .. "=Price Down, " .. WHITE .. "│" .. RESET .. "=High/Low, ─=No Change")
end

function MarketplaceTest:analyzePriceTrend(prices)
    if #prices < 3 then
        return "Insufficient data"
    end
    local windowSize = math.max(3, math.floor(#prices / 5))
    local movingAvgs = {}
    for i = windowSize, #prices do
        local sum = 0
        for j = i - windowSize + 1, i do
            sum = sum + prices[j]
        end
        table.insert(movingAvgs, sum / windowSize)
    end
    local thirdSize = math.floor(#movingAvgs / 3)
    if thirdSize < 1 then
        return "Stable (insufficient variation)"
    end
    local firstThirdAvg = 0
    for i = 1, thirdSize do
        firstThirdAvg = firstThirdAvg + movingAvgs[i]
    end
    firstThirdAvg = firstThirdAvg / thirdSize
    local lastThirdAvg = 0
    for i = #movingAvgs - thirdSize + 1, #movingAvgs do
        lastThirdAvg = lastThirdAvg + movingAvgs[i]
    end
    lastThirdAvg = lastThirdAvg / thirdSize
    local change = ((lastThirdAvg - firstThirdAvg) / firstThirdAvg) * 100
    if change > 10 then
        return string.format("Strong Uptrend (+%.1f%% overall)", change)
    elseif change > 3 then
        return string.format("Mild Uptrend (+%.1f%% overall)", change)
    elseif change < -10 then
        return string.format("Strong Downtrend (%.1f%% overall)", change)
    elseif change < -3 then
        return string.format("Mild Downtrend (%.1f%% overall)", change)
    else
        return string.format("Stable (%.1f%% variation)", change)
    end
end

return MarketplaceTest
