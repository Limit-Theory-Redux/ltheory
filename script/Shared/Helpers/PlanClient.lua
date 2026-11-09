-- The client of the system planner's worker: sends a request (bodies, endpoints, ship, optional
-- travel-drive block) and hands back the plan when it has come. One worker serves the whole Lua
-- state; it starts with the first request.
-- Design: doc/script/system-planner.md
local SystemPlanner = require("Shared.Helpers.SystemPlanner")

local WORKER_NAME   = "SystemPlanner"
local WORKER_SCRIPT = "script/Shared/Helpers/SystemPlannerWorker.lua"
local WORKER_COUNT  = 1

---A point of a plan: a world point when `frame` is nil, else an offset, in world axes, from the
---centre of body `frame` of the request (1-based).
---@class PlanWaypoint
---@field frame integer|nil Index of the body the offset is from
---@field offset Position
---@field radius number The corner's arc radius, game units; zero for a sharp corner and for the goal
---@field speed number The speed the corner is passed at, game units per second; zero to stop there
---@field time number Seconds from the snapshot to the waypoint, at the speeds of the plan
---@field drive boolean Whether the leg that ends at the waypoint is flown under the travel drive

---@class Plan
---@field status integer|nil A `SystemPlanner.Status` value; nil when the worker failed
---@field error string|nil The worker's error message
---@field waypoints PlanWaypoint[] The path, the goal last
---@field times number[] Seconds from the snapshot to each waypoint
---@field kernelMs number Milliseconds the kernel took in the worker

---What a request carries: the worker's task. `bodies` is `count * BODY_STRIDE` numbers, `start` and
---`goal` are 3 numbers, `goalFrame` is a 0-based body index or `SYSTEM_FRAME`, and `drive` is
---`DRIVE_STRIDE` numbers or nil.
---@class PlanRequest
---@field bodies number[]
---@field count integer
---@field start number[]
---@field goalFrame integer
---@field goal number[]
---@field speed number
---@field accel number
---@field drag number
---@field depth integer
---@field capacity integer
---@field drive number[]|nil

---@class PlanClient
local PlanClient = {}

local workerId = nil
---Replies that have come and not been taken, by task id.
---@type table<integer, Plan>
local results = {}
---Tasks nobody waits for any more, whose replies are dropped when they come.
---@type table<integer, true>
local discarded = {}

---Starts the worker once. The engine stops it at shutdown: a worker stopped from Lua stays
---registered, so it can neither be started again nor be stopped at shutdown without an error.
local function worker()
    if not workerId then
        workerId = TaskQueue:startWorker(WORKER_NAME, WORKER_SCRIPT, WORKER_COUNT)
        assert(workerId, "PlanClient: the planner worker did not start")
    end
    return workerId
end

---Sends a request. Not blocking.
---@param task PlanRequest
---@return integer taskId To `take` the plan with, or to `discard` it
function PlanClient.request(task)
    local taskId = TaskQueue:sendTask(worker(), task)
    assert(taskId, "PlanClient: the planning request was not sent")
    return taskId
end

---@param flat number[] `frame, x, y, z, t, radius, speed, drive` per waypoint, as `SystemPlanner.Out` lays them out
---@return PlanWaypoint[] waypoints
---@return number[] times
local function unpackWaypoints(flat)
    local stride, out, waypoints, times = SystemPlanner.OUT_STRIDE, SystemPlanner.Out, {}, {}
    for i = 1, #flat, stride do
        local frame = flat[i + out.Frame]
        table.insert(waypoints, {
            frame = frame ~= SystemPlanner.SYSTEM_FRAME and frame + 1 or nil,
            offset = Position(flat[i + out.X], flat[i + out.Y], flat[i + out.Z]),
            radius = flat[i + out.Radius],
            speed = flat[i + out.Speed],
            time = flat[i + out.T],
            drive = flat[i + out.Drive] ~= 0,
        })
        table.insert(times, flat[i + out.T])
    end
    return waypoints, times
end

---Collects the replies that have come since the last call. Not blocking. Call it before `take`.
function PlanClient.poll()
    if not workerId then return end
    while true do
        local taskId, value, err = TaskQueue:nextTaskResult(workerId)
        if not taskId then return end
        if discarded[taskId] then
            discarded[taskId] = nil
        elseif err then
            results[taskId] = { error = err, waypoints = {}, times = {}, kernelMs = 0 }
        else
            local waypoints, times = unpackWaypoints(value.waypoints)
            results[taskId] = { status = value.status, waypoints = waypoints, times = times, kernelMs = value.kernelMs }
        end
    end
end

---The plan of a request, once it has come and been collected by `poll`.
---@param taskId integer
---@return Plan|nil plan Nil while the reply has not come
function PlanClient.take(taskId)
    local plan = results[taskId]
    results[taskId] = nil
    return plan
end

---Drops a request nobody waits for: its reply is thrown away, whether it has come or not.
---@param taskId integer
function PlanClient.discard(taskId)
    if results[taskId] then
        results[taskId] = nil
    else
        discarded[taskId] = true
    end
end

return PlanClient
