local Job = require('GameObjects.Job')
local Flow = require('Systems.Economy.Flow')
local Actions = requireAll('GameObjects.Actions')

local Mine = subclass(Job, function (self, src, dst, item)
  self.src = src
  self.dst = dst
  self.item = item
  self.jcount = 0
  self.bids = 0
end)

function Mine:clone ()
  return Mine(self.src, self.dst, self.item, self.jcount)
end

function Mine:getFlows (e)
  local item = self.item
  local mass = item:getMass()
  local capacity = e:mgrInventoryGetFreeMax(mass)
  local duration = self:getTravelTime(e, self.src, self.dst) -- TODO : + miningTime (from miningTime() function)
  local rate = floor(capacity / mass) / duration
  return { Flow(item, rate, self.dst) }
end

function Mine:getName (actor)
  if self.jcount == 0 then
    -- This "bids" hack exists because something -- in between Think() waking up a sleeping ship to add
    --     the Undock() action to its queue, and the Undock() function being performed -- is causing
    --     the value of self.jcount for some jobs to be set to 0, and after several days I still can't find why.
    self.jcount = self.bids
  end

  local mineName = format("Mine %d x %s (mass = %s) at %s, drop off at %s (distance = %d)",
    self.jcount,
    self.item:getName(),
    self.item:getMass(),
    self.src:getName(),
    self.dst:getName(),
    self.src:getDistance(self.dst))
--if self.jcount == 0 then
--printf("1 MINE:getName - jcount = 0, bids = %d!!!: %s", self.bids, mineName)
--printf("1 MINE:getName - jcount = 0!!!: %s", self:bonk()) -- this is intended to crash the game for debugging purposes ***
--end

  return mineName
end

function Mine:getPayout (e)
  self.jcount = 0
  local payout = 0
  -- Only stations that are dockable and not destroyed have traders that can offer payouts
  if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
    local bcount, basePayout = Mine:getBasePayout(e, self.src, self.dst)
    if bcount > 0 and basePayout > 0 then
      payout = Mine:getAdjustedPayout(e, self.src, self.dst, basePayout)
      self.jcount = bcount
    else
      self.jcount = 0
    end
  end

--local item = self.item
--local capacity = e:mgrInventoryGetFreeMax(item:getMass())
--local pstr1 = "Mine PAYOUT-ADJU [%s (%s)]: count = %d, item = %s, src = %s, dest = %s, "
--local pstr2 = "base payout = %d, adjusted payout = %d"
--local pstr  = pstr1 .. pstr2
--printf(pstr,
--e:getName(), e:getOwner():getName(), self.jcount, item:getName(), self.src:getName(), self.dst:getName(),
--basePayout, payout)

  return payout
end

function Mine:getBasePayout (e, src, dst)
  -- Calculate the base payout for a Mine job
  -- TODO: Adjust maximum count by current units of Yield available
  local basePayout = 1
  local baseCount = 0
  local item = src:getYield().item
  self.item = item
  local mass = item:getMass()
  local itemBidVol = dst:getTrader():getBidVolume(item)
  if itemBidVol and itemBidVol > 0 then
    -- Mine only as many units as the destination has bids for
    --    or as many as we can carry
    --    or as many as are still minable
    local capacity = e:mgrInventoryGetFreeMax(mass)
    baseCount = math.min(itemBidVol, floor(capacity / mass))
    basePayout = dst:getTrader():getSellToPrice(item, baseCount)

--local pstr1 = "Mine PAYOUT-BASE [%s (%s)]: baseCount = %d, item = %s, src = %s, dest = %s, "
--local pstr2 = "base payout = %d"
--local pstr  = pstr1 .. pstr2
--printf(pstr,
--e:getName(), e:getOwner():getName(), baseCount, item:getName(), src:getName(), dst:getName(),
--basePayout)

  end

  return baseCount, floor(basePayout)
end

function Mine:getAdjustedPayout (e, src, dst, basePayout)
  -- Modify the value of the expected payout by the estimated yield
  --   divided by travel time to reach the yield source plus travel time from the source to the destination
  local adjPayout = 0

  local yieldSize = 1000
  if src:hasYield() then
    yieldSize = min(1000, src:getYieldSize() * 10)
  end
  local pickupTravelTime = self:getShipTravelTime(e, dst)
  local transportTravelTime = self:getTravelTime(e, src, dst)
  local payoutMod = yieldSize / ((pickupTravelTime    / Config.econ.pickupDistWeightMine) +
                                 (transportTravelTime / Config.econ.pickupDistWeightTran))

  adjPayout = math.max(1, floor(basePayout * payoutMod))

  return floor(adjPayout)
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
  if not GameState.paused then
    Profiler.Begin('Actions.Mine.onUpdateActive')
    if not e.jobState then e.jobState = Enums.JobStateMine.None end
    e.jobState = e.jobState + 1

    if e.jobState == Enums.JobStateMine.MovingToAsteroid then
      local item = self.item
      local mass = item:getMass()
      local capacity = e:mgrInventoryGetFreeMax(mass)
      local ccount = floor(capacity / mass)
      local itemBidVol = self.dst:getTrader():getBidVolumeForAsset(item, e)

if ccount == 0 or itemBidVol == 0 then
printf("*** MINE 1 FAIL *** [e:%s (%s)] %d x %s from %s (travel: %d) -> %s (travel: %d), %d bid (dt = %f)",
e:getName(), e:getOwner():getName(), ccount, item:getName(),
self.src:getName(), self:getShipTravelTime(e, self.dst), self.dst:getName(), self:getTravelTime(e, self.src, self.dst),
itemBidVol, dt)
end

      local mcount = math.min(itemBidVol, ccount)
      if mcount == 0 then
        -- Can't do this Mine job! End this job (owning player should seek a new sale for existing inventory)
printf("[MINE 1 FAIL] *** %s: itemBidVol = %d, ccount = %d; terminating mining job", e:getName(), itemBidVol, ccount)
        e:popAction()
        e.jobState = nil
      else
        self.jcount = mcount
--printf("MINE 1: jcount = %d", self.jcount)

        local profit = self.dst:getTrader():getSellToPriceForAsset(item, self.jcount, e)
printf("[MINE 1] [e:%s (%s)] %d x %s from %s (travel: %d) -> %s (travel: %d), %d bid, expect %d profit (dt = %f)",
e:getName(), e:getOwner():getName(), self.jcount, item:getName(),
self.src:getName(), self:getShipTravelTime(e, self.dst), self.dst:getName(), self:getTravelTime(e, self.src, self.dst),
itemBidVol, profit, dt)
        e:pushAction(Actions.MoveTo(self.src, 150)) -- TODO: convert static arrival range to dynamic based on target scale
      end
    elseif e.jobState == Enums.JobStateMine.MiningAsteroid then
      local miningTimePerItem = 5 -- TODO: create a miningTime() function based on item's rarity
      e:pushAction(Actions.MineAt(self.src, self.dst, miningTimePerItem))
    elseif e.jobState == Enums.JobStateMine.DockingAtDst then
      if e:mgrInventoryGetItemCount(self.item) == 0 then
printf("[MINE 3] *** NO SALE *** %s was unable to mine any units of %s for Trader %s, ending MINE action",
e:getName(), self.item:getName(), self.dst:getName())
        e:popAction()
        e.jobState = nil
      else
        if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
          e:pushAction(Actions.DockAt(self.dst))
        else
          -- Destination station no longer exists, so terminate this entire job
printf("[MINE 3] *** Destination station %s no longer exists for %s DockAt; terminating mining job", self.dst:getName(), e:getName())
          e:popAction()
          e.jobState = nil
        end
      end
    elseif e.jobState == Enums.JobStateMine.SellingItems then
      if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
        local item = self.item
--printf("[MINE 4] %s offers to sell %d units of %s to Trader %s",
--e:getName(), e:mgrInventoryGetItemCount(item), item:getName(), self.dst:getName())
        local sold = 0
        while e:mgrInventoryGetItemCount(item) > 0 and self.dst:getTrader():buy(e, item) do
          sold = sold + 1
        end
printf("[MINE 4] %s sold %d units of %s to Trader %s",
e:getName(), sold, item:getName(), self.dst:getName())
      else
        -- Destination station no longer exists, so terminate this entire job
printf("[MINE 4] *** Destination station %s no longer exists for %s item sale; terminating mining job", self.dst:getName(), e:getName())
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateMine.UndockingFromDst then
      if e:isShipDocked() then
printf("[MINE 5] %s pushing action Undock", e:getName())
        e:pushAction(Actions.Undock())
      end
    elseif e.jobState == Enums.JobStateMine.JobFinished then
      -- TODO : This is just a quick hack to force AI to re-evaluate job
      --        decisions. In reality, AI should 'pre-empt' the job, which
      --        should otherwise loop indefinitely by default
      if self.jcount <= 0 then
        e:popAction()
        e.jobState = nil
      else
        -- Repeat until job is done
        e.jobState = Enums.JobStateMine.None
      end
    end
    Profiler.End()
  end
end

return Mine
