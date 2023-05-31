local Entity = require('GameObjects.Entity')
local Jobs = requireAll('GameObjects.Jobs')
local Mine = require('GameObjects.Jobs.Mine')

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
  if not GameState.paused then
    self.timer = self.timer + dt
    --    Profiler.Begin('Economy.Update')
    Profiler.Begin('Economy.Update.tableclear')
    table.clear(self.factories)
    table.clear(self.flows)
    table.clear(self.markets)
    table.clear(self.traders)
    table.clear(self.yields)
    Profiler.End()

    Profiler.Begin('Economy.Update.POI')
    do -- Cache points-of-interest
      for _, e in self.parent:iterChildren() do
        if e:hasFactory() then insert(self.factories, e) end
        if e:hasFlows() then insert(self.flows, e) end
        if e:hasMarket() then insert(self.markets, e) end
        if e:hasTrader() then insert(self.traders, e) end
        if e:hasBlackMarket() then insert(self.blackMarkets, e) end
        if e:hasBlackMarketTrader() then insert(self.blackMarketTraders, e) end
        if e:hasYield() and e:getYieldSize() > 0 then insert(self.yields, e) end
      end
    end
    Profiler.End()

    if self.timer >= self.nextUpdates[1] then
      -- Cache profitable mining jobs
      Profiler.Begin('Economy.Mining.Mining')
      if self.jobs[Enums.Jobs.Mining] then
        table.clear(self.jobs[Enums.Jobs.Mining])
      else
        self.jobs[Enums.Jobs.Mining] = {}
      end

      -- TODO: This section is an enormous CPU hog due to the number of station - asteroid combinations
      local allJobCount = 0
      local realJobCount = 0
      do -- Cache mining jobs
        for _, src in ipairs(self.yields) do
          local yieldSize = src:getYield().size
          local item = src:getYield().item

          if yieldSize > 0 and Config:getObjectInfo("object_types", src:getType()) ~= 'Planet' and src:getClaimsCount() < 1 then --TODO: not hardcode this but integrate with factions & claims system
            for _, dst in ipairs(self.markets) do
              -- Create a Mine job only if the destination trader has a bid for the source item
              if dst:hasDockable() and dst:isDockable() and not dst:isDestroyed() and src:getClaimsCount() < 1 then
                allJobCount = allJobCount + 1
                local itemBidVol = dst:getTrader():getBidVolume(item)
                if itemBidVol > 0 then
                  --printf("ECONOMY: src = %s, dst = %s, item = %s, itemBidVol = %d",
                  --    src:getName(), dst:getName(), item:getName(), itemBidVol)
                  realJobCount = realJobCount + 1
                  insert(self.jobs[Enums.Jobs.Mining], Jobs.Mine(src, dst, item))
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
    --printf("ECONOMY: Mine job test: allJobCount = %d, realJobCount = %d", allJobCount, realJobCount)

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
      if self.jobs[Enums.Jobs.Transport] then
        table.clear(self.jobs[Enums.Jobs.Transport])
      else
        self.jobs[Enums.Jobs.Transport] = {}
      end

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
                        insert(self.jobs[Enums.Jobs.Transport], Jobs.Transport(src, dst, item))
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

    if self.timer >= self.nextUpdates[3] then
      -- Cache profitable trade jobs
      Profiler.Begin('Economy.Update.Marauding')

      if self.blackMarketJobs[Enums.BlackMarketJobs.Marauding] then
        table.clear(self.blackMarketJobs[Enums.BlackMarketJobs.Marauding])
      else
        self.blackMarketJobs[Enums.BlackMarketJobs.Marauding] = {}
      end

      local allJobCount = 0
      local realJobCount = 0
      for i, src in ipairs(self.blackMarketTraders) do
        if src:hasDockable() and src:isDockable() and not src:isDestroyed() then
          for item, data in pairs(src:getBlackMarketTrader().elems) do
            allJobCount = allJobCount + 1
            local itemBidVol = src:getBlackMarketTrader():getBidVolume(item)

            if itemBidVol > 0 then
              realJobCount = realJobCount + 1
              insert(self.blackMarketJobs[Enums.BlackMarketJobs.Marauding], Jobs.Marauding(src, src:getRoot())) --TODO: should also be able to extent to other systems. (once impl)
            end
          end
        end
      end
      -- get next update
      self.nextUpdates[3] = self.timer + updateRates[3] + rng:getUniformRange(0, maxUpdateRateDeviation)
      Profiler.End()
    end
    --printf("ECONOMY: Trade job test: allJobCount = %d, realJobCount = %d", allJobCount, realJobCount)

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
  ctx:text("%d markets", #self.markets)
  ctx:text("Jobs")
  ctx:indent()
  for jobType, job in ipairs(self.jobs) do
    ctx:text("%s: %d", Enums.JobNames[jobType], #self.jobs[jobType])
  end
  ctx:undent()
  ctx:text("Black Market")
  ctx:indent()
  for jobType, job in ipairs(self.blackMarketJobs) do
    ctx:text("%s: %d", Enums.BlackMarketJobNames[jobType], #self.blackMarketJobs[jobType])
  end
  ctx:undent()
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
