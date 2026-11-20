-- The rule that tells a ship is stalled: it has not come `NavConfig.blockedProgress()` closer to its
-- goal for `NavConfig.blockedWindowS` seconds. One watch per flight; the caller gives the clock.
-- Design: doc/script/nav/navigation.md

local NavConfig = require("Config.Navigation.NavConfig")

---@class ProgressWatch
---@field best gameunit The least distance to the goal seen since the last progress
---@field since number The time of the last progress, seconds on the caller's clock
---@field stalled boolean Whether the stall has been reported already
local ProgressWatch = {}

---@param time number Seconds on the caller's clock when the flight begins
---@return ProgressWatch
function ProgressWatch.new(time)
    return { best = math.huge, since = time, stalled = false }
end

---Starts the watch over, as for a new goal.
---@param watch ProgressWatch
---@param time number
function ProgressWatch.reset(watch, time)
    watch.best = math.huge
    watch.since = time
    watch.stalled = false
end

---Takes the distance to the goal at `time`. Progress, a distance `NavConfig.blockedProgress()` below
---the best so far, restarts the window and re-arms the report.
---@param watch ProgressWatch
---@param distance gameunit
---@param time number
---@return boolean stalled True once per stall: when the window has passed with no progress
function ProgressWatch.update(watch, distance, time)
    if distance <= watch.best - NavConfig.blockedProgress() then
        watch.best = distance
        watch.since = time
        watch.stalled = false
        return false
    end
    if watch.stalled or time - watch.since < NavConfig.blockedWindowS then return false end
    watch.stalled = true
    return true
end

return ProgressWatch
