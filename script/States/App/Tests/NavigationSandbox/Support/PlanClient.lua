-- The sandbox's side of the system planner: starts the worker that runs the kernel, sends it a
-- snapshot of the scenario's obstacles as bodies on analytic orbits, and collects the plans that
-- come back, with the time each took from send to receipt.
-- Design: doc/script/system-planner.md
local SystemPlanner = require("Shared.Helpers.SystemPlanner")

local WORKER_NAME   = "NavigationSandboxPlanner"
local WORKER_SCRIPT = "script/States/App/Tests/NavigationSandbox/Support/PlannerWorker.lua"
local WORKER_COUNT  = 1
-- How many times a leg may be split; the plan holds at most one waypoint per leg, and the goal.
local DEPTH_LIMIT   = 5
local CAPACITY      = 2 ^ DEPTH_LIMIT
local TWO_PI        = 2 * math.pi

local Body          = SystemPlanner.Body
local NO_PARENT     = -1

---@class SandboxPlan
---@field status integer|nil A `SystemPlanner.Status` value; nil when the worker failed
---@field error string|nil The worker's error message
---@field waypoints Position[] The path, the goal last
---@field times number[] Seconds from the snapshot to each waypoint at the planning speed
---@field kernelMs number Milliseconds the kernel took in the worker

---@class SandboxPlanReply
---@field ship SandboxShip
---@field plan SandboxPlan
---@field latencyMs number Milliseconds from send to receipt, as seen when the result was polled
---@field simTime number Simulated seconds when the request was sent
---@field probe boolean Whether it answers the latency probe: the ship's plan is not to be taken from it

---@class PlanClient
---@field workerId integer
---@field pending table<integer, { ship: SandboxShip, sentAt: InstantTime, simTime: number, task: table, probe: boolean }>
---@field stale table<integer, true> Tasks of an ended scenario, whose replies are discarded
---@field probed boolean Whether the warm-latency probe has been sent
local PlanClient = {}
PlanClient.__index = PlanClient

---Starts the worker. The engine stops it at shutdown: a worker stopped from Lua stays registered,
---so it can neither be started again nor be stopped at shutdown without an error.
---@return PlanClient
function PlanClient.start()
    local workerId = TaskQueue:startWorker(WORKER_NAME, WORKER_SCRIPT, WORKER_COUNT)
    assert(workerId, "PlanClient: the planner worker did not start")
    return setmetatable({ workerId = workerId, pending = {}, stale = {}, probed = false }, PlanClient)
end

---Drops the requests of a scenario that has ended: their replies, if they still come, are discarded.
function PlanClient:reset()
    for taskId in pairs(self.pending) do self.stale[taskId] = true end
    self.pending = {}
    self.probed = false
end

---Appends obstacle `index` as a body to `values`: where it is, and the orbit it follows, at `simTime`.
---@param values number[]
---@param obstacle SandboxObstacle
---@param simTime number
local function appendBody(values, obstacle, simTime)
    local circle = obstacle.circle
    local row = {}
    row[Body.Parent + 1] = circle and circle.around and circle.around - 1 or NO_PARENT
    local home = circle and circle.centre or obstacle.home
    if home then
        row[Body.X + 1], row[Body.Y + 1], row[Body.Z + 1] = home.x, home.y, home.z
    else
        row[Body.X + 1], row[Body.Y + 1], row[Body.Z + 1] = 0, 0, 0
    end
    local angularSpeed = circle and TWO_PI / circle.periodS or 0
    row[Body.OrbitRadius + 1] = circle and circle.radius or 0
    row[Body.Phase + 1] = angularSpeed * simTime
    row[Body.AngularSpeed + 1] = angularSpeed
    row[Body.Inclination + 1] = 0
    row[Body.Radius + 1] = obstacle.radius
    row[Body.SafeRadius + 1] = obstacle.safeRadius
    for i = 1, SystemPlanner.BODY_STRIDE do values[#values + 1] = row[i] end
end

---Sends a request to plan the ship's path to `goal`, from where it is now.
---@param ship SandboxShip
---@param goal Position
---@param speed number The planning speed, game units per second
---@param obstacles SandboxObstacle[] Parents come before their moons
---@param simTime number Simulated seconds since the scenario started
function PlanClient:request(ship, goal, speed, obstacles, simTime)
    local bodies = {}
    for _, obstacle in ipairs(obstacles) do appendBody(bodies, obstacle, simTime) end
    local at = ship.rb:getPos()
    self:send(ship, simTime, {
        bodies = bodies,
        count = #obstacles,
        start = { at.x, at.y, at.z },
        goal = { goal.x, goal.y, goal.z },
        speed = speed,
        depth = DEPTH_LIMIT,
        capacity = CAPACITY,
    }, false)
end

---@param ship SandboxShip
---@param simTime number
---@param task table The worker's task
---@param probe boolean Whether the reply is only a latency measurement
function PlanClient:send(ship, simTime, task, probe)
    local taskId = TaskQueue:sendTask(self.workerId, task)
    assert(taskId, "PlanClient: the planning request was not sent")
    self.pending[taskId] = { ship = ship, sentAt = InstantTime.Now(), simTime = simTime, task = task, probe = probe }
end

---@param flat number[] `x, y, z, t` per waypoint
---@return Position[] waypoints
---@return number[] times
local function unpackWaypoints(flat)
    local stride, waypoints, times = SystemPlanner.OUT_STRIDE, {}, {}
    for i = 1, #flat, stride do
        table.insert(waypoints, Position(flat[i], flat[i + 1], flat[i + 2]))
        table.insert(times, flat[i + stride - 1])
    end
    return waypoints, times
end

---The plans that have come back since the last call. Not blocking. The first plan to come back
---is followed by one more request with the same task, a probe: by then the worker has started,
---so its reply measures the latency of a worker that is running.
---@return SandboxPlanReply[]
function PlanClient:poll()
    local replies = {}
    while true do
        local taskId, value, err = TaskQueue:nextTaskResult(self.workerId)
        if not taskId then return replies end
        local request = self.pending[taskId]
        self.pending[taskId] = nil
        if request then
            self:collect(replies, request, value, err)
        else
            assert(self.stale[taskId], "PlanClient: a result for an unknown task")
            self.stale[taskId] = nil
        end
    end
end

---@param replies SandboxPlanReply[]
---@param request table The pending entry of the task
---@param value table|nil The worker's result
---@param err string|nil
function PlanClient:collect(replies, request, value, err)
    local plan
    if err then
        plan = { error = err, waypoints = {}, times = {}, kernelMs = 0 }
    else
        local waypoints, times = unpackWaypoints(value.waypoints)
        plan = { status = value.status, waypoints = waypoints, times = times, kernelMs = value.kernelMs }
    end
    table.insert(replies, {
        ship = request.ship,
        plan = plan,
        latencyMs = InstantTime.Now():durationSince(request.sentAt),
        simTime = request.simTime,
        probe = request.probe,
    })
    if not self.probed and not request.probe then
        self.probed = true
        self:send(request.ship, request.simTime, request.task, true)
    end
end

return PlanClient
