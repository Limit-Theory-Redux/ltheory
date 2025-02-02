local Entity = require('GameObjects.Entity')
local fn = require('GameObjects.Elements.Economy._Functions')

-- TEMP
local rng = RNG.FromTime()
local maxUpdateRateDeviation = 0.2
local updateRates = {
    [1] = 2, -- Mining
    [2] = 2, -- Transport
    [3] = 2, -- Marauding
    [4] = 2  -- Patrolling
}

local Economy = Class("Economy", function(self, parent)
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
        [1] = 0, -- Mining
        [2] = 0, -- Transport
        [3] = 0, -- Marauding
        [4] = 0, -- Patrolling
    }

    self.timer = 0
end)



function Economy:update(dt)
    self.timer = self.timer + dt
    if not GameState.paused then
        -- Profiler.Begin('Economy.Update')
        Profiler.Begin('Economy.Update.tableclear')
        table.clear(self.factories)
        table.clear(self.flows)
        table.clear(self.markets)
        table.clear(self.jobs)
        table.clear(self.traders)
        table.clear(self.blackMarkets)
        table.clear(self.blackMarketJobs)
        table.clear(self.blackMarketTraders)
        table.clear(self.yields)
        -- table.clear(self.yields)
        Profiler.End()

        Profiler.Begin('Economy.Update.POI')
        do -- Cache points-of-interest
            for _, e in self.parent:iterChildren() do
                if e:hasFactory() and not e:isDestroyed() and e.zone then insert(self.factories, e) end
                if e:hasFlows() and not e:isDestroyed() then insert(self.flows, e) end
                if e:hasMarket() and not e:isDestroyed() then insert(self.markets, e) end
                if e:hasTrader() and not e:isDestroyed() then insert(self.traders, e) end
                if e:hasBlackMarket() and not e:isDestroyed() then insert(self.blackMarkets, e) end
                if e:hasBlackMarketTrader() and not e:isDestroyed() then insert(self.blackMarketTraders, e) end
                if e:hasYield() and e:getYieldSize() > 0 then insert(self.yields, e) end
            end
        end
        Profiler.End()

        if self.timer >= self.nextUpdates[1] then
            -- Cache profitable mining jobs
            Profiler.Begin('Economy.Update.Mining')

            fn.cacheMiningJobs(self)
            -- get next update
            self.nextUpdates[1] = self.timer + updateRates[1] + rng:getUniformRange(0, maxUpdateRateDeviation)

            Profiler.End()
        end

        if self.timer >= self.nextUpdates[2] then
            -- Cache profitable trade jobs
            Profiler.Begin('Economy.Update.Transport')

            fn.cacheTransportJobs(self)
            -- get next update
            self.nextUpdates[2] = self.timer + updateRates[2] + rng:getUniformRange(0, maxUpdateRateDeviation)

            Profiler.End()
        end
        --printf("ECONOMY: Trade job test: allJobCount = %d, realJobCount = %d", allJobCount, realJobCount)

        if self.timer >= self.nextUpdates[3] then
            -- Cache profitable trade jobs
            Profiler.Begin('Economy.Update.Marauding')

            fn.cacheMaraudingJobs(self)
            -- get next update
            self.nextUpdates[3] = self.timer + updateRates[3] + rng:getUniformRange(0, maxUpdateRateDeviation)

            Profiler.End()
        end

        if self.timer >= self.nextUpdates[4] then
            -- Cache profitable trade jobs
            Profiler.Begin('Economy.Update.Patrolling')

            fn.cachePatrollingJobs(self)

            self.nextUpdates[4] = self.timer + updateRates[4] + rng:getUniformRange(0, maxUpdateRateDeviation)
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

        -- Profiler.End()
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
    self:register(OldEvent.Debug, Entity.debugEconomy)
    --self:register(OldEvent.Update, Entity.updateEconomy)
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

-- from script\Systems\Universe\UniverseEconomy.lua
function Entity:updateEconomy(dt)
    self.economy:update(dt)
end

--------------------------------------------------------------------------------
