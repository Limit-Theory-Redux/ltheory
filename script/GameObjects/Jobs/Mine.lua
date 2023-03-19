local Job = require('GameObjects.Job')
local Flow = require('Systems.Economy.Flow')
local Actions = requireAll('GameObjects.Actions')

local Mine = subclass(Job, function (self, src, dst)
  self.src = src
  self.dst = dst
end)

function Mine:clone ()
  return Mine(self.src, self.dst)
end

function Mine:getFlows (e)
  local capacity = e:getInventoryFree()
  local duration = self:getTravelTime(e) -- TODO : + miningTime
  local item = self.src:getYield().item
  local rate = math.floor(capacity / item:getMass()) / duration
  return { Flow(item, rate, self.dst) }
end

function Mine:getName ()
  return format('Mine %s at %s, drop off at %s (distance = %d)',
    self.src:getYield().item:getName(),
    self.src:getName(),
    self.dst:getName(),
    self.src:getDistance(self.dst))
end

function Mine:getPayout (e)
  local payout = 0
  local capacity = e:getInventoryFree()
  local item = self.src:getYield().item
  local itemBidVol = self.dst:getTrader():getBidVolume(item)
  if itemBidVol and itemBidVol > 0 then
    -- Mine only as many units as the destination has bids for, or as much as we can carry
    local count = math.min(itemBidVol, math.floor(capacity / item:getMass()))
    local value = self.dst:getTrader():getSellToPrice(item, count)

    -- Modify the value of the expected payout by the estimated yield divided by travel time to get there
--    local yieldSize = self.src:getYield().size
    local pickupTravelTime = self:getShipTravelTime(e)
    local transportTravelTime = self:getTravelTime(e)
    local payoutMod = 10000 / ((pickupTravelTime / Config.econ.pickupDistWeightMine) + transportTravelTime)
--    local payoutMod = math.min(10000, yieldSize) / (pickupTravelTime + transportTravelTime)

    payout = math.max(1, math.floor(value * payoutMod))

--local pstr1 = "Mine [%s]: capacity = %d, item = %s, count = %d, src = %s, dest = %s, value = %d, "
--local pstr2 = "yieldsize = %d, pickupTravelTime = %d, transportTravelTime = %d, payoutmod = %f, payout = %s"
--local pstr  = pstr1 .. pstr2
--printf(pstr,
--e:getName(), capacity, item:getName(), count, self.src:getName(), self.dst:getName(), value,
--  yieldSize, transportTravelTime, pickupTravelTime, payoutMod, payout)
  end

  return payout
end

function Mine:getShipTravelTime (e)
  -- Return the travel time between the ship and a non-ship target depending on ship's top speed
  return e:getDistance(self.dst) / e:getTopSpeed()
end

function Mine:getTravelTime (e)
  -- Return the two-way travel time between two non-ship targets depending on ship's top speed
  return 2.0 * self.src:getDistance(self.dst) / e:getTopSpeed()
end

function Mine:onUpdateActive (e, dt)
  if not Config.game.gamePaused then
    if not e.jobState then e.jobState = 0 end
    e.jobState = e.jobState + 1

    if e.jobState == 1 then
      local capacity = e:getInventoryFree()
      local item = self.src:getYield().item
      local count = math.floor(capacity / item:getMass())
      local itemBidVol = self.dst:getTrader():getBidVolume(item)
      if itemBidVol and itemBidVol > 0 then
        count = math.min(itemBidVol, count)
      end
      local profit = self.dst:getTrader():getSellToPrice(item, count)
printf("[MINE] [e:%s] %d x %s from %s (travel: %d) -> %s (travel: %d), expect %d profit",
e:getName(), count, item:getName(),
self.src:getName(), self:getShipTravelTime(e), self.dst:getName(), self:getTravelTime(e), profit)
      e:pushAction(Actions.MoveTo(self.src, 100))
    elseif e.jobState == 2 then
      e:pushAction(Actions.MineAt(self.src, self.dst))
    elseif e.jobState == 3 then
      e:pushAction(Actions.DockAt(self.dst))
    elseif e.jobState == 4 then
      local item = self.src:getYield().item
printf("%s offers to sell %d units of %s to Trader %s",
e:getName(), e:getItemCount(item), item:getName(), self.dst:getName())
      while self.dst:getTrader():buy(e, item) do end
    elseif e.jobState == 5 then
      e:pushAction(Actions.Undock())
    elseif e.jobState == 6 then
      -- TODO : This is just a quick hack to force AI to re-evaluate job
      --        decisions. In reality, AI should 'pre-empt' the job, which
      --        should otherwise loop indefinitely by default
      e:popAction()
      e.jobState = nil
    end
  end
end

return Mine
