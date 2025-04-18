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

local Flow = require('Legacy.Systems.Economy.Flow')
local Job = require('Legacy.GameObjects.Job')

local Transport = Subclass("Transport", Job, function(self, src, dst, item)
    self.src = src
    self.dst = dst
    self.item = item
    self.jcount = 0
    self.bids = 0
end)

function Transport:clone()
    return Transport(self.src, self.dst, self.item, self.jcount)
end

function Transport:cancelJob(e)
    e:popAction()
    e.jobState = nil
end

function Transport:getFlows(e)
    local mass = self.item:getMass()
    local capacity = e:mgrInventoryGetFreeMax(mass) -- NOTE: inventory units? or count of free slots for mass = x?
    local duration = self:getTravelTime(e)
    local count = floor(capacity / mass)
    return {
        Flow(self.item, -count / duration, self.src),
        Flow(self.item, count / duration, self.dst)
    }
end

function Transport:getType()
    return Enums.Jobs.Transport
end

function Transport:getName()
    if self.jcount == 0 then
        self.jcount = self.bids
    end
    return format('Transport %d x %s (mass = %s) from %s to %s',
        self.jcount,
        self.item:getName(),
        self.item:getMass(),
        self.src:getName(),
        self.dst:getName())
end

function Transport:getPayout(e)
    self.jcount = 0
    local payout = 0
    -- Only stations that are dockable and not destroyed have traders that can offer payouts
    if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) and
        self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
        local mass = self.item:getMass()
        local capacity = e:mgrInventoryGetFreeMax(mass)
        local maxCount = floor(capacity / mass)

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
                    payout = math.max(1, floor(profit * payoutMod))
                    self.jcount = count
                    --Log.Debug("2 TRANSPORT: jcount = %s", self.jcount)
                end
            end
        end
    end

    --Log.Debug("Transport check: Asset %s (%d free) taking %d (max %d) units of item %s from %s to %s, raw profit = %d, payout = %d",
    -- e:getName(), capacity, count, maxCount, self.item:getName(), self.src:getName(), self.dst:getName(), profit, payout)

    return payout
end

function Transport:getShipTravelTime(e, dst)
    -- Return the travel time between the ship and a non-ship target depending on ship's top speed
    return e:getDistance(dst) / e:getTopSpeed()
end

function Transport:getTravelTime(e)
    return 2.0 * self.src:getDistance(self.dst) / e:getTopSpeed()
end

function Transport:getThreatLevel()
    local zone = self.src:getZone()
    if zone then
        return zone.threatLevel
    else
        return 0
    end
end

function Transport:onUpdateActive(e, dt)
    if not GameState.paused then
        Profiler.Begin('Actions.Transport.onUpdateActive')
        if not e.jobState then e.jobState = Enums.JobStateTransport.None end
        e.jobState = e.jobState + 1

        if e.jobState == Enums.JobStateTransport.DockingAtSrc then
            local mass = self.item:getMass()
            local capacity = e:mgrInventoryGetFreeMax(mass)
            local capCount = floor(capacity / mass)
            local count, profit = self.src:getTrader():computeTrade(self.item, capCount, self.dst:getTrader(), e)
            Log.Debug("[TRANSPORT 1] %s to move %d x %s from %s -> %s, expect %d profit (oldCount = %d)",
                e:getName(), count, self.item:getName(), self.src:getName(), self.dst:getName(), profit, self.jcount)
            self.jcount = count -- only in case jcount is needed by Trader, which I think it doesn't anymore
            --Log.Debug("3 TRANSPORT: jcount = %s", self.jcount)
            e.count = count
            if count > 0 then
                if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) then
                    e:pushAction(Actions.DockAt(self.src))
                else
                    -- Source station no longer exists, so terminate this entire job
                    Log.Debug(
                        "[TRANSPORT 1] *** Source station %s no longer exists for %s DockAt; terminating transport job",
                        self.src:getName(), e:getName())
                    self:cancelJob(e)
                end
            else
                Log.Debug("[TRANSPORT OFFER FAIL ***] No trade of 0 %s from %s -> %s", self.item:getName(),
                    self.src:getName(), self.dst:getName())
                self:cancelJob(e)
            end
        elseif e.jobState == Enums.JobStateTransport.BuyingItems then
            if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) then
                Log.Debug("[TRANSPORT 2] %s offers to buy %d units of %s from Trader %s", e:getName(), e.count,
                    self.item:getName(), self.src:getName())
                local bought = 0
                for i = 1, e.count do
                    if self.src:getTrader():sell(e, self.item) then
                        bought = bought + 1
                    end
                end
                if bought == 0 then
                    Log.Debug("[TRANSPORT 2 BUY FAIL ***] %s bought 0 %s from %s!", e:getName(), self.item:getName(),
                        self.src:getName())
                    self:cancelJob(e)
                else
                    if bought == e.count then
                        Log.Debug("[TRANSPORT 2] %s bought all %d units of %s from Trader %s",
                            e:getName(), bought, self.item:getName(), self.src:getName())
                    else
                        Log.Debug("[TRANSPORT 2] *** %s bought %d units of %s (%d desired) from Trader %s",
                            e:getName(), bought, self.item:getName(), e.count, self.src:getName())
                    end
                end
            else
                -- Source station no longer exists, so terminate this entire job
                Log.Debug(
                    "[TRANSPORT 2] *** Source station %s no longer exists for %s item purchase; terminating transport job",
                    self.src:getName(), e:getName())
                self:cancelJob(e)
            end
        elseif e.jobState == Enums.JobStateTransport.UndockingFromSrc then
            if e:isShipDocked() then
                Log.Debug("[TRANSPORT 3] %s undocking from Trader %s", e:getName(), self.src:getName())
                e:pushAction(Actions.Undock())
            end
        elseif e.jobState == Enums.JobStateTransport.DockingAtDst then
            Log.Debug("[TRANSPORT 4] %s to move to %s", e:getName(), self.dst:getName())
            if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
                e:pushAction(Actions.DockAt(self.dst))
            else
                -- Destination station no longer exists, so terminate this entire job
                Log.Debug(
                    "[TRANSPORT 4] *** Destination station %s no longer exists for %s DockAt; terminating transport job",
                    self.dst:getName(), e:getName())
                self:cancelJob(e)
            end
        elseif e.jobState == Enums.JobStateTransport.SellingItems then
            if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
                local item = self.item
                Log.Debug("[TRANSPORT 5] %s offers to sell %d units of %s to Trader %s",
                    e:getName(), e.count, item:getName(), self.dst:getName())
                local sold = 0
                while e:mgrInventoryGetItemCount(item) > 0 and self.dst:getTrader():buy(e, item) do
                    sold = sold + 1
                end
                Log.Debug("[TRANSPORT 5] %s sold %d units of %s to Trader %s; %d units remaining in inventory",
                    e:getName(), sold, item:getName(), self.dst:getName(), e:mgrInventoryGetItemCount(item))
            else
                -- Destination station no longer exists, so terminate this entire job
                Log.Debug(
                    "[TRANSPORT 5] *** Destination station %s no longer exists for %s item sale; terminating transport job",
                    self.dst:getName(), e:getName())
                self:cancelJob(e)
            end
        elseif e.jobState == Enums.JobStateTransport.UndockingFromDst then
            if e:isShipDocked() then
                e:pushAction(Actions.Undock())
            end
        elseif e.jobState == Enums.JobStateTransport.JobFinished then
            -- TODO : This is just a quick hack to force AI to re-evaluate job
            -- decisions. In reality, AI should 'pre-empt' the job, which
            -- should otherwise loop indefinitely by default
            self:cancelJob(e)
        end
        Profiler.End()
    end
end

return Transport
