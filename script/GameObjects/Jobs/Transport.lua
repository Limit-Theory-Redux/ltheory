--[[ TODO ----------------------------------------------------------------------
  - AI should use pathfinding
    - Pathfinding should be implemented as a query that goes through the current
      system and uses a lazy NavCache component
    - NavCache must be invalidated upon construction of new transporation
      infrastructure
  - getTravelTime should use route:getTravelTime, not Euclidean
  - Should be possible to do bidirectional trade routes, although reasoning
    about such routes is much more difficult than unidirectional routes
    - Need to use stochastic sampling here, going to have to abandon exhaustive
      search for sure
----------------------------------------------------------------------------]]
                                                                               --

local Flow = require('Systems.Economy.Flow')
local Job = require('GameObjects.Job')

local Transport = subclass(Job, function(self, src, dst, item)
  self.src = src
  self.dst = dst
  self.item = item
  self.jcount = 0
end)

function Transport:clone()
  return Transport(self.src, self.dst, self.item)
end

function Transport:getFlows(e)
  local capacity = e:getInventoryFree()
  local duration = self:getTravelTime(e)
  local count = math.floor(capacity / self.item:getMass())
  return {
    Flow(self.item, -count / duration, self.src),
    Flow(self.item, count / duration, self.dst)
  }
end

function Transport:getName()
  return format('Transport %d x %s from %s to %s',
    self.jcount,
    self.item:getName(),
    self.src:getName(),
    self.dst:getName())
end

function Transport:getPayout(e)
  self.jcount = 0
  local payout = 0
  -- Only stations that are dockable and not destroyed have traders that can offer payouts
  if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) and
      self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
    local capacity = e:getInventoryFree()
    local maxCount = math.floor(capacity / self.item:getMass())

    if maxCount > 0 then
      -- Calculate trade count and profit of available bids
      local srcTrader = self.src:getTrader()
      local dstTrader = self.dst:getTrader()
      local itemAskVol = srcTrader:getAskVolume(self.item)
      local itemBidVol = dstTrader:getBidVolume(self.item)
      if itemAskVol and itemAskVol > 0 and itemBidVol and itemBidVol > 0 then
        local count, profit = srcTrader:computeTrade(self.item, maxCount, dstTrader, nil)

        if count > 0 then
          -- Modify the value of the expected payout by the estimated yield divided by travel time to get there
          local pickupTravelTime = self:getShipTravelTime(e, self.dst)
          local transportTravelTime = self:getTravelTime(e)
          local payoutMod = 10000 / ((pickupTravelTime / Config.econ.pickupDistWeightMine) +
            (transportTravelTime / Config.econ.pickupDistWeightTran))
          payout = math.max(1, math.floor(profit * payoutMod))
          self.jcount = count
        end
      end
    end
  end

  --printf("Transport check: Asset %s (%d free) taking %d (max %d) units of item %s from %s to %s, raw profit = %d, payout = %d",
  --    e:getName(), capacity, count, maxCount, self.item:getName(), self.src:getName(), self.dst:getName(), profit, payout)

  return payout
end

function Transport:getShipTravelTime(e, dst)
  -- Return the travel time between the ship and a non-ship target depending on ship's top speed
  return e:getDistance(dst) / e:getTopSpeed()
end

function Transport:getTravelTime(e)
  return 2.0 * self.src:getDistance(self.dst) / e:getTopSpeed()
end

function Transport:onUpdateActive(e, dt)
  if not GameState.paused then
    Profiler.Begin('Actions.Transport.onUpdateActive')
    if not e.jobState then e.jobState = Enums.JobStateTransport.None end
    e.jobState = e.jobState + 1

    if e.jobState == Enums.JobStateTransport.DockingAtSrc then
      local capacity = e:getInventoryFree()
      local capCount = math.floor(capacity / self.item:getMass())
      local count, profit = self.src:getTrader():computeTrade(self.item, capCount, self.dst:getTrader(), e)
      printf("[TRANSPORT 1] %s to move %d x %s from %s -> %s, expect %d profit (oldCount = %d)",
        e:getName(), count, self.item:getName(), self.src:getName(), self.dst:getName(), profit, self.jcount)
      self.jcount = count
      e.count = count
      if count > 0 then
        if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) then
          e:pushAction(Actions.DockAt(self.src))
        else
          -- Source station no longer exists, so terminate this entire job
          printf("[TRANSPORT 1] *** Source station %s no longer exists for %s DockAt; terminating transport job",
            self.src:getName(), e:getName())
          e:popAction()
          e.jobState = nil
        end
      else
        printf("[TRANSPORT OFFER FAIL ***] No trade of 0 %s from %s -> %s", self.item:getName(), self.src:getName(),
          self.dst:getName())
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateTransport.BuyingItems then
      if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) then
        printf("[TRANSPORT 2] %s offers to buy %d units of %s from Trader %s", e:getName(), e.count, self.item:getName(),
          self.src:getName())
        local bought = 0
        for i = 1, e.count do
          if self.src:getTrader():sell(e, self.item) then
            bought = bought + 1
          end
        end
        if bought == 0 then
          printf("[TRANSPORT 2 BUY FAIL ***] %s bought 0 %s from %s!", e:getName(), self.item:getName(),
            self.src:getName())
          e:popAction()
          e.jobState = nil
        else
          printf("[TRANSPORT 2] %s bought %d units of %s from Trader %s", e:getName(), bought, self.item:getName(),
            self.src:getName())
        end
      else
        -- Source station no longer exists, so terminate this entire job
        printf("[TRANSPORT 2] *** Source station %s no longer exists for %s item purchase; terminating transport job",
          self.src:getName(), e:getName())
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateTransport.UndockingFromSrc then
      if e:isShipDocked() then
        printf("[TRANSPORT 3] %s undocking from Trader %s", e:getName(), self.src:getName())
        e:pushAction(Actions.Undock())
      end
    elseif e.jobState == Enums.JobStateTransport.DockingAtDst then
      printf("[TRANSPORT 4] %s to move to %s", e:getName(), self.dst:getName())
      if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
        e:pushAction(Actions.DockAt(self.dst))
      else
        -- Destination station no longer exists, so terminate this entire job
        printf("[TRANSPORT 4] *** Destination station %s no longer exists for %s DockAt; terminating transport job",
          self.dst:getName(), e:getName())
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateTransport.SellingItems then
      if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
        printf("[TRANSPORT 5] %s offers to sell %d units of %s to Trader %s", e:getName(), e.count, self.item:getName(),
          self.dst:getName())
        local sold = 0
        while self.dst:getTrader():buy(e, self.item) do
          sold = sold + 1
        end
        printf("[TRANSPORT 5] %s sold %d units of %s to Trader %s", e:getName(), sold, self.item:getName(),
          self.dst:getName())
      else
        -- Destination station no longer exists, so terminate this entire job
        printf("[TRANSPORT 5] *** Destination station %s no longer exists for %s item sale; terminating transport job",
          self.dst:getName(), e:getName())
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateTransport.UndockingFromDst then
      if e:isShipDocked() then
        e:pushAction(Actions.Undock())
      end
    elseif e.jobState == Enums.JobStateTransport.JobFinished then
      e:popAction()
      e.jobState = nil
    end
    Profiler.End()
  end
end

return Transport
