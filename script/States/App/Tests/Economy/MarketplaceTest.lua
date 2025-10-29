local Registry = require("Core.ECS.Registry")
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

---@diagnostic disable-next-line
function MarketplaceTest:onInit()
    Log.Info("[MarketplaceTest] Setting up comprehensive marketplace test")

    -- Tag setup first
    Tags:new("TradeType", { "BidOrder", "AskOrder", "Contract", "Auction", "Brokered" })
    Tags:new("Goods", { "HighDemand", "LowSupply", "Volatile", "Luxury", "Consumable", "RawMaterial" })
    Tags:new("Legality", { "Legal", "Restricted", "Contraband" })
    Tags:new("Context", { "OrgTrade", "MissionTrade", "EventTrade" })

    -- Create marketplace
    local STATION_ID = 214523059
    local trader = CoreEntities.Player("Station Trader", true)
    local station = ConstructsEntities.SpaceStation(STATION_ID)
    local marketplace = station:get(EconomyComponents.Marketplace)
    marketplace:setTrader(trader)

    -- Create multiple traders
    self.traders = {}
    self.ships = {}

    for i = 1, 5 do
        local player = CoreEntities.Player("Trader_" .. i, true)
        local ship = ConstructsEntities.Spaceship(100000000 + i)
        Registry:attachEntity(ship, player)

        -- Give each trader MORE credits and inventory
        local inv = ship:get(EconomyComponents.Inventory)
        inv:addItem(EconomyEntities.Item(Items.Virtual.Credit, 5e6)) -- More credits

        -- Give ALL traders ALL items so they can fulfill asks
        inv:addItem(EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000))
        inv:addItem(EconomyEntities.Item(Items.Data.InfoWafer, 10000))
        inv:addItem(EconomyEntities.Item(Items.RawMaterials.IronOre, 50000))

        table.insert(self.traders, player)
        table.insert(self.ships, ship)
    end

    -- Store marketplace for updates
    self.marketplace = marketplace
    self.marketplaceId = tostring(marketplace)

    -- Test configuration
    self.testDuration = 1800 -- seconds
    self.orderInterval = 1   -- create new orders every second
    self.lastOrderTime = TimeStamp.Now()
    self.startTime = TimeStamp.Now()
    self.orderCounter = 0
    self.testComplete = false

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
        pricesByItem = {},
        priceHistory = {},        -- Track prices over time with timestamps
        supplyDemandHistory = {}, -- Track supply/demand ratios over time
        tagHistory = {}           -- Track when tags appear/disappear
    }

    -- Time tracking for snapshots
    self.lastSnapshotTime = TimeStamp.Now()
    self.snapshotInterval = 5 -- Take snapshot every 5 seconds

    --* add transaction events?
    -- EventBus:subscribe(Event.MarketTransaction, self, self.onTransaction)

    Log.Info("[MarketplaceTest] Test initialized with %d traders", #self.traders)
    Log.Info("[MarketplaceTest] Will run for %d seconds, creating orders every %d seconds",
        self.testDuration, self.orderInterval)

    -- Create initial orders
    self:createRandomOrders(10)
end

--todo: add these events to MarketplaceSystem
function MarketplaceTest:onTransaction(event)
    self.stats.tradesExecuted = self.stats.tradesExecuted + 1
    self.stats.totalVolume = self.stats.totalVolume + (event.quantity * event.price)

    local itemName = Items:getDefinition(event.itemType).name
    if not self.stats.pricesByItem[itemName] then
        self.stats.pricesByItem[itemName] = {}
    end
    table.insert(self.stats.pricesByItem[itemName], event.price)

    -- Record with timestamp
    if not self.stats.priceHistory[itemName] then
        self.stats.priceHistory[itemName] = {}
    end
    table.insert(self.stats.priceHistory[itemName], {
        price = event.price,
        quantity = event.quantity,
        timestamp = math.abs(TimeStamp.Now():getDifference(self.startTime))
    })

    Log.Info("[Trade #%d] %s: %d x %s @ %d credits (buyer: %s, seller: %s)",
        self.stats.tradesExecuted,
        itemName,
        event.quantity,
        itemName,
        event.price,
        tostring(event.buyer),
        tostring(event.seller))
end

function MarketplaceTest:createRandomOrders(count)
    for i = 1, count do
        local trader = self.traders[math.random(1, #self.traders)]
        local item = self.tradableItems[math.random(1, #self.tradableItems)]
        local isBid = math.random() > 0.5

        -- Get suggested price from marketplace
        local suggestedPrice
        if isBid then
            suggestedPrice = MarketplaceSystem:getSuggestedBidPrice(self.marketplace, item.id)
        else
            suggestedPrice = MarketplaceSystem:getSuggestedAskPrice(self.marketplace, item.id)
        end

        -- Add some randomness to prices
        local priceVariation = math.random(80, 120) / 100 -- 80% to 120%
        local price = math.max(1, math.floor(suggestedPrice * priceVariation))
        local quantity = math.random(10, 100)

        -- Create order
        local order = EconomyEntities.Order(trader, item, quantity, price)
        local tagComp = order:get(CoreComponents.Tag)

        -- Set trade type
        if isBid then
            tagComp:addTag("BidOrder")
            self.marketplace:addBid(order)
        else
            tagComp:addTag("AskOrder")
            self.marketplace:addAsk(order)
        end

        -- Set item characteristics (these would normally come from item definition)
        if item == Items.Data.InfoWafer then
            tagComp:addTag("Consumable")
        elseif item == Items.RefinedMaterials.Gold then
            tagComp:addTag("Luxury")
        elseif item == Items.RawMaterials.IronOre then
            tagComp:addTag("RawMaterial")
        end

        tagComp:addTag("Legal")
        tagComp:addTag("OrgTrade")

        self.stats.ordersCreated = self.stats.ordersCreated + 1
        self.orderCounter = self.orderCounter + 1

        Log.Debug("[Order #%d] Created %s for %s: %d x %s @ %d credits",
            self.orderCounter,
            isBid and "BID" or "ASK",
            tostring(trader),
            quantity,
            item.name,
            price)
    end
end

---@param data EventData
function MarketplaceTest:onPreRender(data)
    if self.testComplete then return end

    local now = TimeStamp.Now()
    local elapsed = self.startTime:getElapsed()
    self.lastTimePrint = self.lastTimePrint or now

    if self.lastTimePrint:getDifference(now) >= 10 then
        self.lastTimePrint = now
        Log.Debug("\27[92m[MarketplaceTest] Elapsed time: %.1fs\27[0m", elapsed)
        Log.Debug("\27[92m[MarketplaceTest] Time left: %.1fs\27[0m", math.max(0, self.testDuration - elapsed))
    end

    -- Check if test is complete
    if elapsed >= self.testDuration then
        self:printFinalReport()
        self.testComplete = true
        self:quit()
        return
    end

    -- Take market snapshot periodically
    if self.lastSnapshotTime:getDifference(now) >= self.snapshotInterval then
        self:takeMarketSnapshot(elapsed)
        self.lastSnapshotTime = now
    end

    -- Create new orders periodically
    local timeSinceLastOrder = self.lastOrderTime:getDifference(now)
    if timeSinceLastOrder >= self.orderInterval then
        self:createRandomOrders(math.random(2, 5))
        self.lastOrderTime = now
    end

    -- Print interim report every 10 seconds
    local elapsedInt = math.floor(elapsed)
    if elapsedInt > 0 and elapsedInt % 10 == 0 and not self.lastReportTime then
        self:printInterimReport(elapsed)
        self.lastReportTime = elapsedInt
    elseif elapsedInt % 10 ~= 0 then
        self.lastReportTime = nil
    end
end

---Take a snapshot of current market state
---@param timestamp number
function MarketplaceTest:takeMarketSnapshot(timestamp)
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local conditions = self:analyzeItemMarket(item.id)

        -- Record supply/demand
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

        -- Record active tags
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

    -- Print current order book
    local bids = self.marketplace:getBids()
    local asks = self.marketplace:getAsks()
    local bidCount = 0
    local askCount = 0

    for _ in Iterator(bids) do bidCount = bidCount + 1 end
    for _ in Iterator(asks) do askCount = askCount + 1 end

    Log.Info("Current Order Book: %d bids, %d asks", bidCount, askCount)

    -- Print market conditions and price trends
    Log.Info("\n--- Market State by Item ---")
    for _, item in ipairs(self.tradableItems) do
        local conditions = self:analyzeItemMarket(item.id)
        local priceStats = self:getPriceStats(item.name, elapsed)

        Log.Info("  %s:", item.name)
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

            -- Check for condition tags
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
        end
    end

    for ask in Iterator(asks) do
        if ask:get(EconomyComponents.ItemType):getItemType() == itemId then
            askCount = askCount + 1
            totalSupply = totalSupply + ask:get(EconomyComponents.Quantity):getQuantity()
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

--Get price statistics for an item
---@param itemName string
---@param currentTime number
---@return table|nil
function MarketplaceTest:getPriceStats(itemName, currentTime)
    local history = self.stats.priceHistory[itemName]
    if not history or #history == 0 then
        return nil
    end

    -- Calculate stats
    local sum = 0
    local min = history[1].price
    local max = history[1].price
    local recentPrices = {} -- Last 5 trades

    for i, trade in ipairs(history) do
        sum = sum + trade.price
        min = math.min(min, trade.price)
        max = math.max(max, trade.price)

        if i > #history - 5 then
            table.insert(recentPrices, trade.price)
        end
    end

    local average = sum / #history

    -- Calculate trend (comparing first half to second half)
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

    -- PRICE DEVELOPMENT VISUALIZATION
    Log.Info("\n--- PRICE DEVELOPMENT OVER TIME ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        self:printPriceDevelopment(itemName)
    end

    -- Detailed price analysis per item
    Log.Info("\n--- PRICE ANALYSIS ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local history = self.stats.priceHistory[itemName]

        if history and #history > 0 then
            Log.Info("\n%s: %d trades", itemName, #history)

            -- Overall statistics
            local sum = 0
            local min = history[1].price
            local max = history[1].price

            for _, trade in ipairs(history) do
                sum = sum + trade.price
                min = math.min(min, trade.price)
                max = math.max(max, trade.price)
            end

            local avg = sum / #history

            -- Calculate variance
            local variance = 0
            for _, trade in ipairs(history) do
                variance = variance + (trade.price - avg) ^ 2
            end
            local stdDev = math.sqrt(variance / #history)
            local cv = (stdDev / avg) * 100 -- Coefficient of variation

            Log.Info("  Overall: Avg=%.2f, Min=%d, Max=%d, StdDev=%.2f, CV=%.1f%%",
                avg, min, max, stdDev, cv)

            -- Price timeline (show 5 sample points across the test)
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

            -- Volatility assessment
            if cv > 20 then
                Log.Info("  Assessment: HIGHLY VOLATILE market")
            elseif cv > 10 then
                Log.Info("  Assessment: Moderately volatile market")
            else
                Log.Info("  Assessment: Stable market")
            end
        else
            Log.Info("\n%s: No trades", itemName)
        end
    end

    -- Supply/Demand evolution
    Log.Info("\n--- SUPPLY/DEMAND EVOLUTION ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local sdHistory = self.stats.supplyDemandHistory[itemName]

        if sdHistory and #sdHistory > 0 then
            Log.Info("\n%s:", itemName)

            -- Show beginning, middle, end
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

    -- Tag history
    Log.Info("\n--- MARKET CONDITION TAGS OVER TIME ---")
    for _, item in ipairs(self.tradableItems) do
        local itemName = item.name
        local tagHist = self.stats.tagHistory[itemName]

        if tagHist and #tagHist > 0 then
            Log.Info("\n%s:", itemName)

            -- Show when tags appeared/disappeared
            local lastTags = {}
            for _, snapshot in ipairs(tagHist) do
                local currentTags = table.toSet(snapshot.tags)
                local previousTags = table.toSet(lastTags)

                -- Check for new tags
                local newTags = {}
                for tag, _ in pairs(currentTags) do
                    if not previousTags[tag] then
                        table.insert(newTags, tag)
                    end
                end

                -- Check for removed tags
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

            -- Final state
            if #lastTags > 0 then
                Log.Info("  Final: %s", table.concat(lastTags, ", "))
            else
                Log.Info("  Final: No condition tags")
            end
        end
    end

    -- Final order book state
    Log.Info("\n--- FINAL ORDER BOOK ---")
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

---Print price development with ASCII chart
---@param itemName string
function MarketplaceTest:printPriceDevelopment(itemName)
    local history = self.stats.priceHistory[itemName]

    if not history or #history == 0 then
        Log.Info("\n%s: No trade history", itemName)
        return
    end

    Log.Info("\n%s: %d trades", itemName, #history)

    -- Calculate statistics
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

    -- Print summary stats
    Log.Info("  Stats: Min=%d, Max=%d, Avg=%.2f, Range=%d (%.1f%%)",
        minPrice, maxPrice, avgPrice, maxPrice - minPrice,
        ((maxPrice - minPrice) / avgPrice) * 100)

    -- Print price timeline table
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

    -- ASCII Price Chart
    Log.Info("\n  1 (60s timeline):")
    self:printASCIIChart(history, minPrice, maxPrice)
    self:printCandlestickChart(history, minPrice, maxPrice)

    -- Price trend analysis
    local trend = self:analyzePriceTrend(prices)
    Log.Info("\n  Trend Analysis: %s", trend)
end

---Print ASCII chart of price over time
---@param history table[]
---@param minPrice number
---@param maxPrice number
function MarketplaceTest:printASCIIChart(history, minPrice, maxPrice)
    local chartWidth = 170 -- characters
    local chartHeight = 10 -- lines
    local priceRange = maxPrice - minPrice

    if priceRange == 0 then
        Log.Info("    Price remained constant at %d", minPrice)
        return
    end

    -- Find actual time range from trade history
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

    -- Map trades to columns and collect all prices per column
    local columns = {}
    for col = 1, chartWidth do
        columns[col] = {}
    end

    for _, trade in ipairs(history) do
        -- Map timestamp to column (1 to chartWidth)
        local normalizedTime = (trade.timestamp - minTime) / timeRange
        local col = math.floor(normalizedTime * (chartWidth - 1)) + 1
        col = math.max(1, math.min(chartWidth, col))

        table.insert(columns[col], trade.price)
    end

    -- Calculate median for each column (more robust than average)
    local plotData = {}
    for col = 1, chartWidth do
        if #columns[col] > 0 then
            -- Sort prices to get median
            table.sort(columns[col])
            local mid = math.ceil(#columns[col] / 2)
            plotData[col] = columns[col][mid]
        else
            plotData[col] = nil
        end
    end

    -- Draw chart from top to bottom
    for row = chartHeight, 1, -1 do
        local line = "    "
        -- row=1 should be minPrice, row=chartHeight should be maxPrice
        local priceLevel = minPrice + (priceRange * (row - 1) / (chartHeight - 1))

        -- Y-axis label
        line = line .. string.format("%5.0f |", priceLevel)

        -- Plot points
        for col = 1, chartWidth do
            local price = plotData[col]

            if price then
                -- Normalize price to 0-1 range
                local normalizedPrice = (price - minPrice) / priceRange
                -- Normalize row to 0-1 range
                local rowNormalized = (row - 1) / (chartHeight - 1)

                -- Check if this price should be plotted at this row
                if math.abs(normalizedPrice - rowNormalized) < (1.0 / chartHeight) then
                    line = line .. "●"
                elseif normalizedPrice >= rowNormalized then
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

    -- X-axis
    local xAxis = "          +"
    for i = 1, chartWidth do
        xAxis = xAxis .. "-"
    end
    Log.Info(xAxis)

    -- X-axis labels with actual time range
    local labelSpacing = chartWidth - 15
    Log.Info("           %.1fs%s%.1fs",
        minTime,
        string.rep(" ", labelSpacing),
        maxTime)

    -- Add statistics about data density
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

---Print ASCII candlestick chart (better for high-volume data)
---@param history table[]
---@param minPrice number
---@param maxPrice number
function MarketplaceTest:printCandlestickChart(history, minPrice, maxPrice)
    local chartWidth = 170 -- Wider buckets for candlesticks
    local chartHeight = 10
    local priceRange = maxPrice - minPrice

    if priceRange == 0 then
        Log.Info("    Price remained constant at %d", minPrice)
        return
    end

    -- Find time range
    local minTime = math.huge
    local maxTime = -math.huge

    for _, trade in ipairs(history) do
        minTime = math.min(minTime, trade.timestamp)
        maxTime = math.max(maxTime, trade.timestamp)
    end

    local timeRange = maxTime - minTime
    if timeRange <= 0 then return end

    -- Collect trades per time bucket
    local buckets = {}
    for col = 1, chartWidth do
        buckets[col] = { prices = {}, first = nil, last = nil }
    end

    for _, trade in ipairs(history) do
        local normalizedTime = (trade.timestamp - minTime) / timeRange
        local col = math.floor(normalizedTime * (chartWidth - 1)) + 1
        col = math.max(1, math.min(chartWidth, col))

        table.insert(buckets[col].prices, trade.price)
        if not buckets[col].first then
            buckets[col].first = trade.price
        end
        buckets[col].last = trade.price
    end

    -- Calculate OHLC (Open, High, Low, Close) for each bucket
    local candlesticks = {}
    for col = 1, chartWidth do
        if #buckets[col].prices > 0 then
            table.sort(buckets[col].prices)
            candlesticks[col] = {
                open = buckets[col].first,
                high = buckets[col].prices[#buckets[col].prices],
                low = buckets[col].prices[1],
                close = buckets[col].last,
                count = #buckets[col].prices
            }
        end
    end

    -- Draw candlestick chart
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
                    line = line .. "│" -- Upper wick
                elseif rowNorm <= bodyTop and rowNorm >= bodyBottom then
                    -- Body
                    if candle.close > candle.open then
                        line = line .. "▓" -- Bullish (green)
                    elseif candle.close < candle.open then
                        line = line .. "░" -- Bearish (red)
                    else
                        line = line .. "─" -- Neutral
                    end
                elseif rowNorm <= bodyBottom and rowNorm >= normLow then
                    line = line .. "│" -- Lower wick
                else
                    line = line .. " "
                end
            else
                line = line .. " "
            end
        end

        Log.Info(line)
    end

    -- X-axis
    local xAxis = "          +"
    for i = 1, chartWidth do
        xAxis = xAxis .. "-"
    end
    Log.Info(xAxis)

    Log.Info("           %.1fs%s%.1fs", minTime, string.rep(" ", chartWidth - 15), maxTime)
    Log.Info("           Legend: ▓=Price Up, ░=Price Down, │=High/Low, ─=No Change")
end

---Analyze overall price trend
---@param prices number[]
---@return string
function MarketplaceTest:analyzePriceTrend(prices)
    if #prices < 3 then
        return "Insufficient data"
    end

    -- Calculate moving average to smooth out noise
    local windowSize = math.max(3, math.floor(#prices / 5))
    local movingAvgs = {}

    for i = windowSize, #prices do
        local sum = 0
        for j = i - windowSize + 1, i do
            sum = sum + prices[j]
        end
        table.insert(movingAvgs, sum / windowSize)
    end

    -- Compare first third to last third
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
