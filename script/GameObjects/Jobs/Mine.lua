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
  local duration = self:getTravelTime(e, self.src, self.dst) -- TODO : + miningTime (from miningTime() function)
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
  local basePayout = Mine:getBasePayout(e, self.src, self.dst)
  if basePayout > 0 then
    payout = Mine:getAdjustedPayout(e, self.src, self.dst, basePayout)
  end

--local capacity = e:getInventoryFree()
--local item = self.src:getYield().item
--local pstr1 = "Mine PAYOUT-ADJU [%s (%s)]: capacity = %d, item = %s, src = %s, dest = %s, "
--local pstr2 = "base payout = %f, adjusted payout = %f"
--local pstr  = pstr1 .. pstr2
--printf(pstr,
--e:getName(), e:getOwner():getName(), capacity, item:getName(), self.src:getName(), self.dst:getName(),
--basePayout, payout)

  return payout
end

function Mine:getBasePayout (e, src, dst)
  -- Calculate the base payout for a Mine job
  -- TODO: Adjust maximum count by current units of Yield available
  local basePayout = 0
  local item = src:getYield().item
  local itemBidVol = dst:getTrader():getBidVolume(item)
  if itemBidVol and itemBidVol > 0 then
    -- Mine only as many units as the destination has bids for
    --    or as many as we can carry
    --    or as many as are still minable
    local capacity = e:getInventoryFree()
    local count = math.min(itemBidVol, math.floor(capacity / item:getMass()))
    basePayout = dst:getTrader():getSellToPrice(item, count)

--local pstr1 = "Mine PAYOUT-BASE [%s (%s)]: capacity = %d, item = %s, count = %d, src = %s, dest = %s, "
--local pstr2 = "base payout = %f"
--local pstr  = pstr1 .. pstr2
--printf(pstr,
--e:getName(), e:getOwner():getName(), capacity, item:getName(), count, src:getName(), dst:getName(),
--basePayout)

  end

  return math.floor(basePayout)
end

function Mine:getAdjustedPayout (e, src, dst, basePayout)
  -- Modify the value of the expected payout by the estimated yield (not enabled until Yield becomes variable)
  --   divided by travel time to reach the yield source plus travel time from the source to the destination
  local adjPayout = 0

--  local yieldSize = self.src:getYield().size
  local pickupTravelTime = self:getShipTravelTime(e, dst)
  local transportTravelTime = self:getTravelTime(e, src, dst)
  local payoutMod = 10000 / ((pickupTravelTime    / Config.econ.pickupDistWeightMine) +
                             (transportTravelTime / Config.econ.pickupDistWeightTran))
--  local payoutMod = math.min(10000, yieldSize) / (pickupTravelTime + transportTravelTime)

  adjPayout = math.max(1, math.floor(basePayout * payoutMod))

  return math.floor(adjPayout)
end

function Mine:getShipTravelTime (e, dst)
  -- Return the travel time between the ship and a non-ship target depending on ship's top speed
  return e:getDistance(dst) / e:getTopSpeed()
end

function Mine:getTravelTime (e, src, dst)
  -- Return the two-way travel time between two non-ship targets depending on ship's top speed
  return 2.0 * src:getDistance(dst) / e:getTopSpeed()
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
if count == 0 or itemBidVol == 0 then
printf("*** MINE FAIL *** [e:%s (%s)] %d x %s from %s (travel: %d) -> %s (travel: %d), %d bid (dt = %f)",
e:getName(), e:getOwner():getName(), count, item:getName(),
self.src:getName(), self:getShipTravelTime(e, self.dst), self.dst:getName(), self:getTravelTime(e, self.src, self.dst),
itemBidVol, dt)
end
      if itemBidVol and itemBidVol > 0 then
        count = math.min(itemBidVol, count)
      end
      if count == 0 then
        -- Can't do this Mine job! End this job (owning player should seek a new sale for existing inventory)
        e:popAction()
        e.jobState = nil
      else
        local profit = self.dst:getTrader():getSellToPrice(item, count)
printf("[MINE] [e:%s (%s)] %d x %s from %s (travel: %d) -> %s (travel: %d), %d bid, expect %d profit (dt = %f)",
e:getName(), e:getOwner():getName(), count, item:getName(),
self.src:getName(), self:getShipTravelTime(e, self.dst), self.dst:getName(), self:getTravelTime(e, self.src, self.dst),
itemBidVol, profit, dt)
        e:pushAction(Actions.MoveTo(self.src, 100))
      end
    elseif e.jobState == 2 then
      local miningTimePerItem = 5
      e:pushAction(Actions.MineAt(self.src, self.dst, miningTimePerItem)) -- TODO: create a miningTime() function
    elseif e.jobState == 3 then
      local item = self.src:getYield().item
      if e:getItemCount(item) == 0 then
printf("*** NO SALE *** %s was unable to mine any units of %s for Trader %s, ending MINE action",
e:getName(), item:getName(), self.dst:getName())
        e.jobState = 6 -- end MINE action
      else
        e:pushAction(Actions.DockAt(self.dst))
      end
    elseif e.jobState == 4 then
      local item = self.src:getYield().item
printf("%s offers to sell %d units of %s to Trader %s",
e:getName(), e:getItemCount(item), item:getName(), self.dst:getName())
if e:getItemCount(item) == 0 then
printf("***** WHAT SALE??? *****")
end
      while e:getItemCount(item) > 0 and self.dst:getTrader():buy(e, item) do end
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
