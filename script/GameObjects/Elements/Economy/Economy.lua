local Entity = require('GameObjects.Entity')
local Jobs = requireAll('GameObjects.Jobs')
local Mine = require('GameObjects.Jobs.Mine')
local Item = require('Systems.Economy.Item')

--------------------------------------------------------------------------------

-- TEMP
local rng = RNG.FromTime()
local maxUpdateRateDeviation = 0.2
local updateRates = {
    [1] = 2, -- Mining
    [2] = 2, -- Transport
    [3] = 2  -- Marauding
}

local Economy = class(function(self, parent)
    self.parent = parent
    self.factories = {}
    self.flows = {}
    self.goods = {}
    self.jobs = {}
    self.markets = {}
    self.traders = {}
    self.blackMarketTraders = {}
    self.blackMarkets = {}
    self.blackMarketJobs = {}
    self.yields = {}

    self.nextUpdates = {
        [1] = 0,  -- Mining
        [2] = 0,  -- Transport
        [3] = 0   -- Marauding
    }

    self.timer = 0
end)

-- TODO : Economy cache should be updated infrequently and the update should be
--        spread over many frames.
-- NOTE : In particular, the evaluation of Mining jobs becomes very expensive as
--        asteroid count (Yield) and station/planet count (Market) increase.
function Economy:update(dt)
    self.timer = self.timer + dt
    if not GameState.paused then
        --    Profiler.Begin('Economy.Update')

        Profiler.Begin('Economy.Update.tableclear')
        table.clear(self.factories)
        table.clear(self.flows)
        table.clear(self.markets)
        table.clear(self.jobs)
        table.clear(self.traders)
        table.clear(self.yields)
--        table.clear(self.yields)
        Profiler.End()

        Profiler.Begin('Economy.Update.POI')
        do -- Cache points-of-interest
            for _, e in self.parent:iterChildren() do
                if e:hasFactory() and not e:isDestroyed() and e.zone then insert(self.factories, e) end
                if e:hasFlows() and not e:isDestroyed() then insert(self.flows, e) end
                if e:hasMarket() and not e:isDestroyed() then insert(self.markets, e) end
                if e:hasTrader() and not e:isDestroyed() then insert(self.traders, e) end
--                if e:hasYield() and e:getYieldSize() > 0 then insert(self.yields, e) end
            end
        end
        Profiler.End()

        if self.timer >= self.nextUpdates[1] then
            -- Cache profitable mining jobs
            Profiler.Begin('Economy.Update.Mining')
            local jobCount = 0
            do -- Cache mining jobs
                -- Iterate through all factories at functional space stations in this star system
                for _, station in ipairs(self.factories) do
                    local factory = station:getFactory()
                    local prodLines = factory:getProds()
                    -- Iterate through all production lines in the factory
                    for _, prodLine in ipairs(prodLines) do
                        -- Iterate through all Inputs for this production line
                        for _, input in prodLine.type:iterInputs() do
                            local item = input.item
                            if item:hasItem(Item.T2, item) then -- is Input item a minable resource?
                                local itemBidVol = station:getTrader():getBidVolume(item)
                                -- Does the Trader at the same station as this Factory have any bids for the minable Item?
                                if itemBidVol > 0 then
                                    -- Get no more than [considerCount] of the asteroids in the zone of this station
                                    --     whose current Yield is at least the number of the trader's bids for this Input Item
                                    local considerCount = 50
                                    local baseYield = itemBidVol * 3

                                    table.clear(self.yields)
                                    for i, asteroid in station.zone:iterChildren() do
                                        if asteroid:hasYield() and asteroid:getYieldSize() > baseYield then
                                            insert(self.yields, asteroid)
                                        end
                                        if i > considerCount then break end
                                    end
--printf    ("ECONOMY: Mine job test: station = %s, prod = %s, item = %s, #asteroids = %d",
--    st    ation:getName(), prodLine.type.name, item:getName(), #self.yields)

                                    for i, asteroid in ipairs(self.yields) do
                                        --printf("ECONOMY: src = %s, dst = %s, item = %s, itemBidVol = %d",
                                        --    src:getName(), dst:getName(), item:getName(), itemBidVol)
                                        jobCount = jobCount + 1
                                        insert(self.jobs, Jobs.Mine(asteroid, station, item))
                                        if i == considerCount then break end
                                    end
                                end
                            end
                        end
                    end
                end
            end
            -- get next update
            self.nextUpdates[1] = self.timer + updateRates[1] + rng:getUniformRange(0, maxUpdateRateDeviation)
            Profiler.End()
        end
--        printf("ECONOMY: Mine job test: jobCount = %d", jobCount)

--        -- Cache profitable mining jobs    -- INACTIVE (old style mining job generator)
--        Profiler.Begin('Economy.Update.Mining')
--        -- TODO: This section is an enormous CPU hog due to the number of station - asteroid combinations
--        local allJobCount = 0
--        local realJobCount = 0
--        do -- Cache mining jobs
--            for _, src in ipairs(self.yields) do
--                local item = src:getYield().item
--                for _, dst in ipairs(self.markets) do
--                    -- Create a Mine job only if the destination trader has a bid for the source item
--                    if dst:hasDockable() and dst:isDockable() and not dst:isDestroyed() then
--                        allJobCount = allJobCount + 1
--                        local itemBidVol = dst:getTrader():getBidVolume(item)
--                        if itemBidVol > 0 then
--                            --printf("ECONOMY: src = %s, dst = %s, item = %s, itemBidVol = %d",
--                            --    src:getName(), dst:getName(), item:getName(), itemBidVol)
--                            realJobCount = realJobCount + 1
--                            insert(self.jobs, Jobs.Mine(src, dst, item))
--                        end
--                    end
--                end
--            end
--        end
--        Profiler.End()
--        --printf("ECONOMY: Mine job test: allJobCount = %d, realJobCount = %d", allJobCount, realJobCount)

        --    if false then  -- INACTIVE (Josh code - preserve this for when we switch back to Flow model)
        --      do -- Cache trade jobs from positive to negative flow
        --        for _, src in ipairs(self.markets) do
        --          for item, srcFlow in pairs(src:getFlows()) do
        --            if srcFlow > 0 then
        --              for _, dst in ipairs(self.markets) do
        --                if dst:getFlow(item) < 0 then
        --                  insert(self.jobs, Jobs.Transport(src, dst, item))
        --                end
        --              end
        --            end
        --          end
        --        end
        --      end
        --    end

        if self.timer >= self.nextUpdates[2] then
            -- Cache profitable trade jobs
            Profiler.Begin('Economy.Update.Transport')
            local allJobCount = 0
            local realJobCount = 0
            for _, src in ipairs(self.traders) do
                if src:hasDockable() and src:isDockable() and not src:isDestroyed() then
                    for item, data in pairs(src:getTrader().elems) do
                        if src:getTrader():getAskVolume(item) > 0 then
                            local buyPrice = src:getTrader():getBuyFromPrice(item, 1)
                            --printf("Buy? item %s from %s, buyPrice = %d", item:getName(), src:getName(), buyPrice)
                            if buyPrice > 0 then
                                for _, dst in ipairs(self.traders) do
                                    if dst:hasDockable() and dst:isDockable() and not dst:isDestroyed() then
                                        if src ~= dst then
                                            allJobCount = allJobCount + 1
                                            local sellPrice = dst:getTrader():getSellToPrice(item, 1)
                                            --printf("Transport test: item %s from %s @ buyPrice = %d to %s @ sellPrice = %d",
                                            --    item:getName(), src:getName(), buyPrice, dst:getName(), sellPrice)
                                            if buyPrice < sellPrice then
                                                --printf("Transport job insert: item %s from %s @ buyPrice = %d to %s @ sellPrice = %d",
                                                --    item:getName(), src:getName(), buyPrice, dst:getName(), sellPrice)
                                                realJobCount = realJobCount + 1
                                                insert(self.jobs, Jobs.Transport(src, dst, item))
                                            end
                                        end
                                    end
                                end
                            end
                        end
                    end
                end
            end
            -- get next update
            self.nextUpdates[2] = self.timer + updateRates[2] + rng:getUniformRange(0, maxUpdateRateDeviation)
            Profiler.End()
        end
        --printf("ECONOMY: Trade job test: allJobCount = %d, realJobCount = %d", allJobCount, realJobCount)

        if self.timer >= self.nextUpdates[3] then
            -- Cache profitable trade jobs
            Profiler.Begin('Economy.Update.Marauding')
            local allJobCount = 0
            local realJobCount = 0
            for _, src in ipairs(self.blackMarketTraders) do
                if src:hasDockable() and src:isDockable() and not src:isDestroyed() then
                    for item, data in pairs(src:getBlackMarketTrader().elems) do
                        if src:getBlackMarketTrader():getAskVolume(item) > 0 then
                            local buyPrice = src:getBlackMarketTrader():getBuyFromPrice(item, 1)
                            --printf("Buy? item %s from %s, buyPrice = %d", item:getName(), src:getName(), buyPrice)
                            if buyPrice > 0 then
                                allJobCount = allJobCount + 1
                                --printf("Marauding job insert: item %s from %s @ buyPrice = %d to %s @ sellPrice = %d",
                                --    item:getName(), src:getName(), buyPrice, dst:getName(), sellPrice)
                                realJobCount = realJobCount + 1
                                insert(self.jobs, Jobs.Marauding(src, src:getRoot())) --TODO: should also be able to extent to other systems. 
                            end
                        end
                    end
                end
            end
            -- get next update
            self.nextUpdates[3] = self.timer + updateRates[3] + rng:getUniformRange(0, maxUpdateRateDeviation)
            Profiler.End()
        end

        Profiler.Begin('Economy.Update.Flows')
        do -- Compute net flow of entire economy
            -- Clear current flow
            for k, v in pairs(self.parent.flows) do self.parent.flows[k] = 0 end

            -- Sum all flows
            for _, e in ipairs(self.flows) do
                for k, v in pairs(e.flows) do
                    self.parent.flows[k] = (self.parent.flows[k] or 0) + v
                end
            end
        end
        Profiler.End()

        do -- Compute commodity metrics
            self.goods = {}
        end

        --    Profiler.End()
    end
end

function Economy:debug(ctx)
    ctx:text("Economy")
    ctx:indent()
    ctx:text("%d jobs", #self.jobs)
    ctx:text("%d markets", #self.markets)
    for item, data in pairs(self.goods) do
        ctx:text("%s", item:getName())
        ctx:indent()
        ctx:text("BUYING  : min = %.2f, max = %.2f", data.buyMin, data.buyMax)
        ctx:text("SELLING : min = %.2f, max = %.2f", data.sellMin, data.sellMax)
        ctx:undent()
    end
    ctx:undent()
end

--------------------------------------------------------------------------------

function Entity:addEconomy()
    assert(not self.economy)
    self.economy = Economy(self)
    self:register(Event.Debug, Entity.debugEconomy)
    self:register(Event.Update, Entity.updateEconomy)
end

function Entity:debugEconomy(state)
    self.economy:debug(state.context)
end

function Entity:getEconomy()
    assert(self.economy)
    return self.economy
end

function Entity:hasEconomy()
    return self.economy ~= nil
end

function Entity:updateEconomy(state)
    self.economy:update(state.dt)
end

--------------------------------------------------------------------------------
