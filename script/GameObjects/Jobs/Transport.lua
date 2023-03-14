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
----------------------------------------------------------------------------]]--

local Flow = require('Systems.Economy.Flow')
local Job = require('GameObjects.Job')

local Transport = subclass(Job, function (self, src, dst, item)
  self.src = src
  self.dst = dst
  self.item = item
end)

function Transport:clone ()
  return Transport(self.src, self.dst, self.item)
end

function Transport:getFlows (e)
  local capacity = e:getInventoryFree()
  local duration = self:getTravelTime(e)
  local count = math.floor(capacity / self.item:getMass())
  return {
    Flow(self.item, -count / duration, self.src),
    Flow(self.item,  count / duration, self.dst)
  }
end

function Transport:getName ()
  return format('Transport %s from %s to %s',
    self.item:getName(),
    self.src:getName(),
    self.dst:getName())
end

function Transport:getPayout (e)
  local payout = 0
  local capacity = e:getInventoryFree()
  local maxCount = math.floor(capacity / self.item:getMass())
  local count, profit = self.src:getTrader():computeTrade(self.item, maxCount, self.dst:getTrader())

  if count > 0 then
    -- Modify the value of the expected payout by the estimated yield divided by travel time to get there
    local pickupTravelTime = self:getShipTravelTime(e)
    local transportTravelTime = self:getTravelTime(e)
    local payoutMod = 10000 / ((pickupTravelTime / Config.econ.pickupDistWeightTran) + transportTravelTime)
    payout = math.max(1, math.floor(profit * payoutMod))
  else
    payout = 0
  end

--printf("Transport check: Asset %s (%d free) taking %d (max %d) units of item %s from %s to %s, raw profit = %d, payout = %d",
--    e:getName(), capacity, count, maxCount, self.item:getName(), self.src:getName(), self.dst:getName(), profit, payout)

  return payout
end

function Transport:getShipTravelTime (e)
  -- Return the travel time between the ship and a non-ship target depending on ship's top speed
  return e:getDistance(self.dst) / e:getTopSpeed()
end

function Transport:getTravelTime (e)
  return 2.0 * self.src:getDistance(self.dst) / e:getTopSpeed()
end

function Transport:onUpdateActive (e, dt)
  if not e.jobState then e.jobState = 0 end
  e.jobState = e.jobState + 1

  if e.jobState == 1 then
    local capacity = e:getInventoryFree()
    local maxCount = math.floor(capacity / self.item:getMass())
    local count, profit = self.src:getTrader():computeTrade(self.item, maxCount, self.dst:getTrader())
printf("[TRADE] %d x %s from %s -> %s, expect %d profit", count, self.item:getName(), self.src:getName(), self.dst:getName(), profit)
    e.tradeCount = count
    e:pushAction(Actions.DockAt(self.src))
  elseif e.jobState == 2 then
printf("%s offers to buy %d units of %s from Trader %s", e:getName(), e.tradeCount, self.item:getName(), self.src:getName())
    for i = 1, e.tradeCount do self.src:getTrader():sell(e, self.item) end
  elseif e.jobState == 3 then
    e:pushAction(Actions.Undock())
  elseif e.jobState == 4 then
    e:pushAction(Actions.DockAt(self.dst))
  elseif e.jobState == 5 then
printf("%s offers to sell %d units of %s to Trader %s", e:getName(), e.tradeCount, self.item:getName(), self.dst:getName())
    while self.dst:getTrader():buy(e, self.item) do end
  elseif e.jobState == 6 then
    e:pushAction(Actions.Undock())
  elseif e.jobState == 7 then
    e:popAction()
    e.jobState = nil
  end
end

return Transport
