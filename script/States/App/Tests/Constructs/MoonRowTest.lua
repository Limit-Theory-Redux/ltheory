-- Checks that a ship flown by the player's autopilot from one end of a row of four moons to the other
-- does not turn on the last leg: the moons between lie across the course, so the plan goes round them,
-- and its last corner before the goal, the entry of the goal moon's well, lies on the line from the
-- corner before it to the moon's centre, so the plan does not turn there. The ship flies the plan
-- and arrives. Logs each failed check and ends with a failure exit code.

local Application         = require("States.Application")
local ConstructComponents = require("Modules.Constructs.Components")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local AutoPilotSystem     = require("Modules.Constructs.Systems.AutoPilotSystem")
local ControlOwnerSystem  = require("Modules.Constructs.Systems.ControlOwnerSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem   = require("Modules.Constructs.Systems.TravelDriveSystem")
local Nav                 = require("Modules.Navigation")
local SystemPlanner       = require("Shared.Helpers.SystemPlanner")
local FlightScene         = require("States.App.Tests.Support.FlightScene")

---@class ConstructsMoonRowTest: Application
local MoonRowTest = Subclass("ConstructsMoonRowTest", Application)

local STEP = 1 / 60
-- Safety limits that turn a hang into a failure: the wait for the plan, and the flight, in seconds.
local PLAN_WAIT_S = 30
local MAX_FLIGHT_S = 120
local MS_PER_S = 1000

-- The moons, in game units: a generated moon's hull radius, the distance between neighbours along x,
-- and how far each lies off the row's axis (z), so that the course between the end moons passes
-- inside the safe sphere of the middle ones, on a known side.
local MOON_RADIUS = 136
local MOON_SPACING = 4000
local MOON_ASIDE = { 0, 60, -80, 0 }
local GOAL_MOON = #MOON_ASIDE
-- The ship starts this far from the first moon's centre, beyond its safe sphere.
local START_GAP = 1.5 * MOON_RADIUS + 10
-- The largest turn at the last corner before the goal, in radians: the legs on either side of it
-- point the same way.
local MAX_LAST_CORNER_TURN = 0.002
-- Where the planner puts the entry waypoint of a goal in a well: this share of the well's radius.
local WELL_ENTRY_RADIUS = (1 - SystemPlanner.WELL_ENTRY_MARGIN) * FlightScene.WELL_RADIUS

---The angle between two vectors in radians, zero for a vector of no length.
local function angleBetween(a, b)
    local la = math.sqrt(a.x * a.x + a.y * a.y + a.z * a.z)
    local lb = math.sqrt(b.x * b.x + b.y * b.y + b.z * b.z)
    if la == 0 or lb == 0 then return 0 end
    local cosine = (a.x * b.x + a.y * b.y + a.z * b.z) / (la * lb)
    return math.acos(math.max(-1, math.min(1, cosine)))
end

local function offset(from, to)
    return { x = to.x - from.x, y = to.y - from.y, z = to.z - from.z }
end

function MoonRowTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("ConstructsMoonRowTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---One simulation step in the order the apps use. Notes how the order ended, which the autopilot
---system reads and releases it after.
---@param order Entity
function MoonRowTest:step(order)
    Nav.update(STEP)
    if not self.endedAs and order:get(Nav.Components.OrderStatus):hasEnded() then self.endedAs = Nav.status(order) end
    AutoPilotSystem:update()
    TravelDriveSystem:update(STEP)
    ShipFlightSystem:update()
    self.world:update(STEP)
end

---Waits for the plan of the flight that has just been engaged, which a worker makes.
---@return boolean planned False when the wait ran out
function MoonRowTest:awaitPlan(ship)
    Nav.update(0)
    AutoPilotSystem:update()
    local waitStart = InstantTime.Now()
    while AutoPilotSystem:isPlanning(ship.entity)
        and InstantTime.Now():durationSince(waitStart) < PLAN_WAIT_S * MS_PER_S do
        Nav.update(0)
        AutoPilotSystem:update()
    end
    return not AutoPilotSystem:isPlanning(ship.entity)
end

---The plan's corners and goal as world points, and the turn at each corner, from the plan's follower.
---@param follower PlanFollower
---@param start Position
---@return Position[] path The start, the corners, the goal
---@return number[] turns The turn at each corner, radians
local function planPath(follower, start)
    local path = { start }
    for _, point in ipairs(follower.points) do path[#path + 1] = follower:pointPlanned(point) end
    path[#path + 1] = follower:pointPlanned(follower.goal)
    local turns = {}
    for i = 2, #path - 1 do
        turns[i - 1] = angleBetween(offset(path[i - 1], path[i]), offset(path[i], path[i + 1]))
    end
    return path, turns
end

function MoonRowTest:onInit()
    self.passed, self.failed = 0, 0
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    local moons = {}
    for i, aside in ipairs(MOON_ASIDE) do
        moons[i] = FlightScene.spawnMoon(self.world, Position((i - 1) * MOON_SPACING, 0, aside), MOON_RADIUS)
    end
    GravityWellSystem:collectZones()
    local goalMoon = moons[GOAL_MOON]
    local centre = goalMoon.rb:getPos()

    local start = Position(moons[1].rb:getPos().x + START_GAP, 0, 0)
    local ship = FlightScene.spawnShip(self.world, start, { autoPilot = true })
    ControlOwnerSystem:setManual(ship.entity, true)
    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    -- Sensing and the planner read the query index as the last physics update left it.
    self.world:update(STEP)
    Nav.attach(self.world)
    AutoPilotSystem:engage(ship.entity, goalMoon.entity)
    local order = ship.entity:get(ConstructComponents.AutoPilot):getOrder()
    local Status = Enums.NavOrderStatus

    self:check(self:awaitPlan(ship), "the plan comes")
    Nav.update(STEP)

    local path, turns, plannedAt = nil, nil, nil
    local driveSeen, arrivedAt = false, nil
    for stepIndex = 1, math.floor(MAX_FLIGHT_S / STEP) do
        self:step(order)
        local flight = order:isValid() and order:get(Nav.Components.OrderFlight)
        local plan = flight and flight.plan
        if plan and plan.follower and not path then
            path, turns = planPath(plan.follower, start)
            plannedAt = stepIndex * STEP
        end
        driveSeen = driveSeen or drive:isActive()
        if not AutoPilotSystem:isActive(ship.entity) then
            arrivedAt = stepIndex * STEP
            break
        end
    end

    self:check(path ~= nil, "the ship flies a plan")
    if path then
        local entry = path[#path - 1]
        local lastTurn = turns[#turns]
        local text = {}
        for i, turn in ipairs(turns) do text[i] = string.format("%.4f", turn) end
        Log.Info("ConstructsMoonRowTest: %d corners, turns %s rad, plan seen at %.1f s", #path - 2,
            table.concat(text, ", "), plannedAt)
        self:check(#path >= 4, "the plan goes round the moons between", string.format("%d corners", #path - 2))
        self:check(math.abs(entry:distance(centre) - WELL_ENTRY_RADIUS) < 1,
            "the last corner is the entry of the goal moon's well",
            string.format("%.1f from the centre, the entry is at %.1f", entry:distance(centre), WELL_ENTRY_RADIUS))
        self:check(lastTurn <= MAX_LAST_CORNER_TURN, "no turn at the last corner before the goal",
            string.format("%.4f rad against %.4f", lastTurn, MAX_LAST_CORNER_TURN))
    end
    self:check(arrivedAt ~= nil and self.endedAs == Status.Done, "the ship arrives",
        string.format("%s s, ended as %s", tostring(arrivedAt), tostring(self.endedAs)))
    self:check(driveSeen, "the long legs are flown under the travel drive")
    Log.Info("ConstructsMoonRowTest: arrival after %s s", tostring(arrivedAt))

    AutoPilotSystem:disengage(ship.entity)
    Nav.update(0)
    AutoPilotSystem:update()
    Nav.detach()
    FlightScene.despawn(self.world, { ship })

    Log.Info("ConstructsMoonRowTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return MoonRowTest
