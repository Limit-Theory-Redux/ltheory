-- The sandbox's side of the system planner: sends the shared planner client a snapshot of the
-- scenario's obstacles as bodies on analytic orbits and of its cost zones weighted by the ship's
-- profile, and collects the plans that come back, with the time each took from send to receipt.
-- Design: doc/script/nav/system-planner.md

local SystemPlanner        = require("Shared.Helpers.SystemPlanner")
local PlannerClient        = require("Shared.Helpers.PlanClient")
local CostZones            = require("Shared.Helpers.CostZones")
local ConstructComponents  = require("Modules.Constructs.Components")
local NavigationComponents = require("Modules.Navigation.Components")

-- How many times a leg may be split; the plan holds at most one waypoint per leg, and the goal.
local DEPTH_LIMIT   = 5
local CAPACITY      = 2 ^ DEPTH_LIMIT
local TWO_PI        = 2 * math.pi

local Body          = SystemPlanner.Body
local NO_PARENT     = -1

---@class SandboxPlanReply
---@field ship SandboxShip
---@field plan Plan
---@field latencyMs number Milliseconds from send to receipt, as seen when the result was polled
---@field simTime number Simulated seconds when the request was sent
---@field probe boolean Whether it answers the latency probe: the ship's plan is not to be taken from it

---@class PlanClient
---@field pending table<integer, { ship: SandboxShip, sentAt: InstantTime, simTime: number, task: PlanRequest, probe: boolean }>
---@field probed boolean Whether the warm-latency probe has been sent
local PlanClient = {}
PlanClient.__index = PlanClient

---Creates the client. The worker itself belongs to the shared client and starts with the first request.
---@return PlanClient
function PlanClient.start()
    return setmetatable({ pending = {}, probed = false }, PlanClient)
end

---Drops the requests of a scenario that has ended: their replies, if they still come, are discarded.
function PlanClient:reset()
    for taskId in pairs(self.pending) do PlannerClient.discard(taskId) end
    self.pending = {}
    self.probed = false
end

---Drops the ship's plan requests that are out: their replies, if they still come, are discarded.
---The latency probe is not the ship's plan, and stays.
---@param ship SandboxShip
function PlanClient:discardFor(ship)
    for taskId, request in pairs(self.pending) do
        if request.ship == ship and not request.probe then
            PlannerClient.discard(taskId)
            self.pending[taskId] = nil
        end
    end
end

---The sandbox's obstacles are not entities, so no zone rides on a body.
local function noBody() return nil end

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
    -- The sandbox plans without the travel drive: no body has a zone.
    row[Body.ZoneRadius + 1], row[Body.ZoneCap + 1] = 0, 0
    for i = 1, SystemPlanner.BODY_STRIDE do values[#values + 1] = row[i] end
end

---Sends a request to plan the ship's path to `goal`, from where it is now, with the scenario's cost
---zones weighted by the ship's cost profile (none when it has none).
---@param ship SandboxShip
---@param goal Position A world point, or an offset from obstacle `goalFrame` when that is given
---@param goalFrame integer|nil Index of the obstacle the goal rides on
---@param speed number The planning speed, game units per second
---@param obstacles SandboxObstacle[] Parents come before their moons
---@param simTime number Simulated seconds since the scenario started
---@param arrivalRadius number How close to the goal the ship counts as arrived, game units
function PlanClient:request(ship, goal, goalFrame, speed, obstacles, simTime, arrivalRadius)
    local bodies = {}
    for _, obstacle in ipairs(obstacles) do appendBody(bodies, obstacle, simTime) end
    local at = ship.rb:getPos()
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)
    local zones, zoneCount = CostZones.collect(NavigationComponents.CostProfile.ProfileOf(ship.entity), noBody)
    self:send(ship, simTime, {
        bodies = bodies,
        count = #obstacles,
        start = { at.x, at.y, at.z },
        goalFrame = goalFrame and goalFrame - 1 or SystemPlanner.SYSTEM_FRAME,
        goal = { goal.x, goal.y, goal.z },
        speed = speed,
        accel = profile.thrustForward / profile.mass,
        drag = profile.linearDrag,
        depth = DEPTH_LIMIT,
        capacity = CAPACITY,
        arrivalRadius = arrivalRadius,
        zones = zones,
        zoneCount = zoneCount,
    }, false)
end

---@param ship SandboxShip
---@param simTime number
---@param task PlanRequest The worker's task
---@param probe boolean Whether the reply is only a latency measurement
function PlanClient:send(ship, simTime, task, probe)
    local taskId = PlannerClient.request(task)
    self.pending[taskId] = { ship = ship, sentAt = InstantTime.Now(), simTime = simTime, task = task, probe = probe }
end

---The plans that have come back since the last call. Not blocking. The first plan to come back
---is followed by one more request with the same task, a probe: by then the worker has started,
---so its reply measures the latency of a worker that is running.
---@return SandboxPlanReply[]
function PlanClient:poll()
    PlannerClient.poll()
    local ids = {}
    for taskId in pairs(self.pending) do ids[#ids + 1] = taskId end
    table.sort(ids)
    local replies = {}
    for _, taskId in ipairs(ids) do
        local plan = PlannerClient.take(taskId)
        if plan then
            local request = self.pending[taskId]
            self.pending[taskId] = nil
            self:collect(replies, request, plan)
        end
    end
    return replies
end

---@param replies SandboxPlanReply[]
---@param request table The pending entry of the task
---@param plan Plan
function PlanClient:collect(replies, request, plan)
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
