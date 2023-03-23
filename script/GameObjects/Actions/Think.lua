local Action = require('GameObjects.Action')
local Player = require('GameObjects.Entities.Player')

local kJobIterations = 4000 -- how many randomly-chosen jobs the asset will consider before deciding

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
--  for i = 1, kJobIterations do
--    -- TODO : KnowsAbout check
--    local job = self.rng:choose(root:getEconomy().jobs)
--    if not job then break end
--
--    local pressure = job:getPressure(asset)
--    if pressure < bestPressure then
--      bestPressure = pressure
--      bestJob = job
----printf("[asset:%s] pressure = %s, job = %s", asset:getName(), pressure, job:getName())
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

if true then -- Use payout, not flow
  function Think:manageAsset (asset)
    local root = asset:getRoot()
    local bestPayout = 0
    local bestJob = nil

    -- Consider re-running last job
    if asset.job then
      local payout = asset.job:getPayout(asset)
      if payout > bestPayout then
        bestPayout = payout
        bestJob = asset.job
      end
    end

    -- Consider changing to a new job
    for i = 1, math.min(kJobIterations, #root:getEconomy().jobs * 2) do
      -- TODO : KnowsAbout check (information economy + AI load reduction)
      local job = self.rng:choose(root:getEconomy().jobs)
      if not job then break end

      local payout = job:getPayout(asset)
      if payout > bestPayout then
        if job.jcount > 0 then
          bestPayout = payout
          bestJob = job
        else
printf("THINK ***: tried to pick action: %s %s with payout = %d but jcount = 0!",
asset:getName(), job:getName(), payout)
        end
      end
    end

    -- Maybe assign a new or reassign an old job
    if bestJob and bestPayout > 0 then -- if asset has no capacity left, bestPayout should be 0
      asset.job = bestJob
printf("THINK: pushing action: %s %s with bestPayout = %d", asset:getName(), asset.job:getName(), bestPayout)
      asset:pushAction(bestJob)
      -- Place offer for the best job's bids to reserve them
      asset.job.jcount = asset.job.dst:getTrader():addBidOffer(asset)
      -- If buying, place offer for the best job's asks to reserve them (must be after addBidOffer() to use asset.job.jcount)
      if string.find(asset.job:getName(), "Transport") then -- ugly way to insure this is a job whose source is a Trader
        asset.job.src:getTrader():addAskOffer(asset)
      end

      -- Wake up asset if it was sleeping and make sure it undocks
      local station = asset:isShipDocked()
      if station then
printf("THINK +++: Asset '%s' (owner %s) wakes up at Station '%s'",
asset:getName(), asset:getOwner():getName(), station:getName())
        asset:pushAction(Actions.Undock())
      end
    end

    if asset:isIdle() and asset:isShipDocked() == nil then
      -- No more jobs available; send asset to nearest station to sleep
      -- TODO: Make sure this is only done at the AI player's direction for ECONOMIC ships (miners and transports)!
      local system = asset.parent

      local stations = system:getStationsByDistance(asset)
      if #stations > 0 and stations[1] ~= nil then
        local station = stations[1].stationRef

printf("THINK ---: Asset '%s' (owner %s), with capacity %d has no more jobs available; docking at Station '%s'",
asset:getName(), asset:getOwner():getName(), asset:getInventoryFree(), station:getName())
        asset:clearActions()
        asset:pushAction(Actions.DockAt(station))
      end
    end
  end
end

function Think:onUpdateActive (e, dt)
  if not Config.game.gamePaused then
    Profiler.Begin('Action.Think')
    do -- manage assets
      -- TODO: route planning for efficiency (but avoid TSP!)
      for asset in e:iterAssets() do
        if asset:getRoot():hasEconomy() and asset:isIdle() then
          self:manageAsset(asset)
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
