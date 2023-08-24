local Action = require('GameObjects.Action')
local Player = require('GameObjects.Entities.Player')

-- Temporary File until other systems are more fleshed out, to be merged into a general AI script that handles all the THINKING

local CriminalThink = subclass(Action, function (self)
    self.timer = 0
    self.rng = RNG.FromTime()
end)

function CriminalThink:clone()
    return CriminalThink()
end

function CriminalThink:getName()
    return 'CriminalThink'
end

local function applyFlows(flows, mult)
    for _, flow in ipairs(flows) do
        flow.location:modFlow(flow.item, mult * flow.rate)
    end
end

function CriminalThink:manageAsset(asset)
    local root = asset:getRoot()
    local bestPayout = 0
    local bestJob = nil
    local jobAssigned = false

    -- Consider re-running last job
    if asset.job then
        local payout = asset.job:getPayout(asset)
        if payout > bestPayout then
            bestPayout = payout
            bestJob = asset.job
        end
    end

    -- Consider changing to a new job
    for _ = 1, math.min(Config.econ.jobIterations, #root:getEconomy().blackMarketJobs * 2) do
        -- TODO : KnowsAbout check (information economy + AI load reduction)
        local jobType = self.rng:choose(root:getEconomy().blackMarketJobs)
        local job

        if jobType then
            for _ = 1, math.min(Config.econ.jobIterations, #jobType * 2) do
                job = self.rng:choose(jobType)

                if not job then break end

                --if job:getType() == Enums.Jobs.Mining then -- temp preventing all ships to mine at the same asteroid
                --  if job.workers and #job.workers >= job.maxWorkers then -- should do checks here if ship is allowed to mine here
                --    goto skipJob
                --  end
                --end

                local payout = job:getPayout(asset)
                if payout > bestPayout then
                    if job.jcount > 0 then
                        bestPayout = payout
                        bestJob = job
                    else
                        printf("CRIMINAL THINK ***: %s tried to pick job '%s' with payout = %d but jcount = 0!",
                            asset:getName(), job:getName(), payout)
                    end
                end
            end
        end
        ::skipJob::
    end

    -- Maybe assign a new or reassign an old job
    if bestJob and bestPayout > 0 then -- if asset has no capacity left, bestPayout should be 0
        -- Don't assign an old job if it can no longer be completed because a required station was destroyed
        asset.job = bestJob

        if (asset.job.base:hasDockable() and asset.job.base:isDockable() and not asset.job.base:isDestroyed()) and
            (string.find(asset.job:getName(), "Marauding") and asset.job.system) then
            do
                printf("CRIMINAL THINK: pushing job '%s' to %s with bestPayout = %d", asset.job:getName(),
                    asset:getName(), bestPayout)
                asset:pushAction(bestJob)
                jobAssigned = true

                -- Place offer for the best job's bids to reserve them
                asset.job.jcount = asset.job.base:getBlackMarketTrader():addBidOffer(asset)
                asset:setSubType(Config:getObjectTypeByName("ship_subtypes", "Marauder"))

                -- Wake up asset if it was sleeping and make sure it undocks
                local station = asset:isShipDocked()
                if station then
                    printf("CRIMINAL THINK +++: Asset %s (owner %s) wakes up at Station %s",
                        asset:getName(), asset:getOwner():getName(), station:getName())
                    asset:pushAction(Actions.Undock())
                end
            end
        else
            -- TODO: canceling old job, so release any asks or bids held by this ship with a source or destination trader
            printf("CRIMINAL THINK: canceling job '%s' for asset %s", asset.job:getName(), asset:getName())
            asset.job = nil
        end
    end

    if asset:isIdle() and asset:isShipDocked() == nil then
        -- No more jobs available; send asset to nearest station to sleep
        -- TODO: Make sure this is only done at the AI player's direction for ECONOMIC ships (miners and transports)!
        local system = asset.parent

        local stations = system:getStationsByDistance(asset)

        if #stations > 0 then
            local i = 1
            -- only dock at station with black market
            while stations[i] and not stations[i].stationRef:hasBlackMarket() do
                i = i + 1
            end

            if stations[i] then
                local station = stations[i].stationRef
                printf(
                    "CRIMINAL THINK ---: Asset %s (owner %s) with capacity %d has no more jobs available; docking at Station %s",
                    asset:getName(), asset:getOwner():getName(), asset:mgrInventoryGetFreeTotal(), station:getName())
                asset:clearActions()
                asset:pushAction(Actions.DockAt(station))
            else
                -- do nothing
                asset:clearActions()
            end
        end
    end

    if not jobAssigned then
        -- Asset has no job, so revert to default role and store in ship subtype
        asset:setSubType(Config:getObjectTypeByName("ship_subtypes", "Fighter"))
    end
end

function CriminalThink:onUpdateActive(e, dt)
    if not GameState.paused then
        Profiler.Begin('Action.CriminalThink')

        do -- manage assets
            -- TODO: route planning for efficiency (but avoid TSP!)
            for asset in e:iterAssets() do
                if asset:getRoot():hasEconomy() and asset:isIdle() and asset:getType() ~= Config:getObjectTypeByName("object_types", "Station") then
                    self:manageAsset(asset)
                end
            end
        end

        -- Increment elapsed time in seconds (a float value) since game start
        -- Note that self.timer does not appear to reset!
        -- TODO: Correct the self.timer tests below to trigger on their _intervals_,
        --       not on elapsed time (which never resets)
        self.timer = self.timer + dt
        --printf("CriminalThink [%s]: dt = %f, self.timer = %f", e:getName(), dt, self.timer)
    end
    Profiler.End()
end

return CriminalThink
