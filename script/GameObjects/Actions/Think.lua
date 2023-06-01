local Action = require('GameObjects.Action')
local Player = require('GameObjects.Entities.Player')

local Think = subclass(Action, function (self)
  self.timer = 0
  self.rng = RNG.FromTime()
end)

function Think:clone ()
  return Think()
end

function Think:getName ()
  return 'Think'
end

local function applyFlows (flows, mult)
  for _, flow in ipairs(flows) do
    flow.location:modFlow(flow.item, mult * flow.rate)
  end
end

--function Think:manageAsset (asset)
--  local root = asset:getRoot()
--  local bestPressure = asset.job and asset.job:getPressure(asset) or math.huge
--  local bestJob = asset.job
--  for i = 1, Config.econ.jobIterations do
--    -- TODO : KnowsAbout check
--    local job = self.rng:choose(root:getEconomy().jobs)
--    if not job then break end
--
--    local pressure = job:getPressure(asset)
--    if pressure < bestPressure then
--      bestPressure = pressure
--      bestJob = job
----printf("[asset:%s] pressure = %s, job = %s", asset:getName(), pressure, job:getName(asset))
--    end
--  end
--
--  if bestJob then
--    if asset.jobFlows then
--      applyFlows(asset.jobFlows, -1)
--      asset.jobFlows = nil
--    end
--
--    asset.job = bestJob
--    asset.jobFlows = bestJob:getFlows(asset)
--    applyFlows(asset.jobFlows, 1)
--
--    asset:pushAction(bestJob)
--  end
--end

  -- Use payout, not flow
function Think:manageAsset (asset)
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
  for _ = 1, math.min(Config.econ.jobIterations, #root:getEconomy().jobs * 2) do
    -- TODO : KnowsAbout check (information economy + AI load reduction)
    local jobType = self.rng:choose(root:getEconomy().jobs)
    local job

    if jobType then
      for _ = 1, math.min(Config.econ.jobIterations, #jobType * 2) do
        job = self.rng:choose(jobType)
        if not job then break end

        if job:getType() == Enums.Jobs.Mining then -- temp preventing all ships to mine at the same asteroid
          if job.workers and #job.workers >= job.maxWorkers then -- should do checks here if ship is allowed to mine here
            goto skipJob
          end
        end

        local payout = job:getPayout(asset)
        if payout > bestPayout then
          if job.jcount > 0 then
            bestPayout = payout
            bestJob = job
          else
            printf  ("THINK ***: %s tried to pick job '%s' with payout = %d but jcount = 0!",
            asset:  getName(), job:getName(), payout)
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

    if asset.job.workers and #asset.job.workers < asset.job.maxWorkers then -- temporary allow only one worker, this should depend on the job later (e.g. asteroid suze --> max workers)
      asset.job:addWorker(asset)
    end

    if asset.job.workers and not asset.job:isWorker(asset) then
      return
    end

    if (asset.job.dst:hasDockable() and asset.job.dst:isDockable() and not asset.job.dst:isDestroyed()) and
        (not string.find(asset.job:getName(), "Transport") or
        (asset.job.src:hasDockable() and asset.job.src:isDockable() and not asset.job.src:isDestroyed())) then
      -- Place offer for the best job's bids to reserve them
      -- Note that this also sets the job's count of items to be moved
      asset.job.jcount = asset.job.dst:getTrader():addBidOffer(asset)
      asset.job.bids = asset.job.jcount

      -- Push job to asset's Action queue
      printf("THINK: pushing job %s '%s' to %s with jcount = %d, bids = %d, bestPayout = %d",
      asset.job, asset.job:getName(), asset:getName(), asset.job.jcount, asset.job.bids, bestPayout)
      asset:pushAction(bestJob)
      jobAssigned = true

      -- Make some updates based on the type of job being assigned to this asset
      if string.match(asset.job:getName(), "Transport") then
        -- Job is a Trade job
        -- Reserve best job's asks
        asset.job.src:getTrader():addAskOffer(asset)
        asset:setSubType(Config:getObjectTypeByName("ship_subtypes", "Trader"))
      elseif string.find(asset.job:getName(asset), "Mine") then
        -- Job is a Mine job
        asset:setSubType(Config:getObjectTypeByName("ship_subtypes", "Miner"))
      end

      -- Wake up asset if it was sleeping and make sure it undocks
      local station = asset:isShipDocked()
      if station then
        printf("THINK +++ 1: Asset %s (owner %s) wakes up at Station %s with job %s, jcount = %d, bids = %d",
        asset:getName(), asset:getOwner():getName(), station:getName(), asset.job:getName(asset), asset.job.jcount, asset.job.bids)
        --for i, v in ipairs(asset.actions) do
        --  printf("  Actions %d : %s", i, v:getName(asset))
        --end
        asset:pushAction(Actions.Undock())
        printf("THINK +++ 2: Asset %s (owner %s) wakes up at Station %s with job %s, jcount = %d, bids = %d",
        asset:getName(), asset:getOwner():getName(), station:getName(), asset.job:getName(asset), asset.job.jcount, asset.job.bids)
        --for i, v in ipairs(asset.actions) do
        --  printf("  Actions %d : %s", i, v:getName(asset))
        --end
      end
    else
      -- TODO: canceling old job, so release any asks or bids held by this ship with a source or destination trader
      printf("THINK: canceling job '%s' for asset %s", asset.job:getName(asset), asset:getName())
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
      -- don´t dock at hostile stations
      while stations[i] and stations[i].stationRef:getOwner() ~= asset:getOwner() do
        i = i + 1
      end

      if stations[i] then
        local station = stations[i].stationRef
        printf("THINK ---: Asset %s (owner %s) with capacity %d has no more jobs available; docking at Station %s",
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

function Think:onUpdateActive (e, dt)
  if not GameState.paused then
    Profiler.Begin('Action.Think')
    do -- manage assets
      -- TODO: route planning for efficiency (but avoid TSP!)
      for asset in e:iterAssets() do
        if asset:getRoot():hasEconomy() and asset:isIdle() and asset:getType() ~= Config:getObjectTypeByName("object_types", "Station") then
          -- add randomized "thinking time" (so player 1 doesn´t get all jobs)
          if not self.thinking then
            self.thinking = true
            self.thinkTimer = 0
            self.thinkTime = self.rng:getUniformRange(0.1, 10)
            --printf("%s Thinking.. - Time: %d", e:getName(), self.thinkTime)
          end

          if self.thinking then
            self.thinkTimer = self.thinkTimer + dt

            if self.thinkTimer >= self.thinkTime then
              --printf("%s Thinking Done", e:getName(), self.thinkTime)
              self:manageAsset(asset)
              self.thinking = false
            end
          end
        end
      end
    end

    -- Increment elapsed time in seconds (a float value) since game start
    -- Note that self.timer does not appear to reset!
    -- TODO: Correct the self.timer tests below to trigger on their _intervals_,
    --       not on elapsed time (which never resets)
    self.timer = self.timer + dt
--printf("THINK [%s]: dt = %f, self.timer = %f", e:getName(), dt, self.timer)

    do -- TODO: capital expenditure AI
      if self.timer > 30 then
        --
      end
    end

    do -- TODO: fleet management AI
      if self.timer > 60 then
        --
      end
    end

    do -- TODO: strategic goal-planning AI
      if self.timer > 300 then
        --
      end
    end
    Profiler.End()
  end
end

return Think
