-- The autopilot's use of the system planner: sends a request for the path to a goal, takes the plan
-- when it comes, and says which phase of the flight the ship is in. The plan holds the corners
-- (and the legs that are flown under the travel drive); the final approach to the goal is flown by
-- the autopilot's own steering.
-- Design: doc/script/system-planner.md

local ConstructComponents  = require("Modules.Constructs.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local PlanClient           = require("Shared.Helpers.PlanClient")
local PlanFollower         = require("Shared.Helpers.PlanFollower")
local PlannerSnapshot      = require("Shared.Helpers.PlannerSnapshot")
local SystemPlanner        = require("Shared.Helpers.SystemPlanner")

-- How many times a leg may be split; the plan holds at most one waypoint per leg, and the goal.
local DEPTH_LIMIT = 5
local CAPACITY    = 2 ^ DEPTH_LIMIT

-- A corner is reached within the distance the ship covers at its top speed in this many seconds:
-- longer than a frame's move, and well inside the corner's arc (whose radius is the planner's turn
-- radius at the corner's speed, so a wider reach would put the ship off the arc's circle).
local PASS_TIME_S = 0.1

local Status = SystemPlanner.Status
local Drive  = SystemPlanner.Drive

---The phases of an engagement. `Planning`: the request is out, and the ship steers straight to
---the goal in normal flight. `Plan`: the corners of the plan are flown. `Final`: the way to the
---goal from the last corner (or from the start, when the plan has no corner). `Direct`: there is no
---usable plan, and the ship flies straight to the goal as in `Final`, with no detours.
local Phase = { Planning = "planning", Plan = "plan", Final = "final", Direct = "direct" }

-- The plans that are flown. A plan with another status has a leg that is left straight or no path.
local FLYABLE = { [Status.Ok] = true, [Status.GoalNotSettled] = true, [Status.DriveNotTimed] = true }

local AutoPilotPlan = { Phase = Phase }

---The travel-drive block of a request: the drive's numbers and the shortest leg it is used on.
---@param shipEntity Entity
---@param minLength gameunit
---@return number[]|nil block Nil when the ship has no drive or the drive is switched off
local function driveBlock(shipEntity, minLength)
    local cfg = Config.game.autoPilot
    if not shipEntity:get(ConstructComponents.TravelDrive) or cfg.enableTravelDrive == false then return nil end
    local drive = Config.game.travelDrive
    local block = {}
    block[Drive.OpenCap + 1] = drive.maxSpeedByZone.openSpace
    block[Drive.Charge + 1] = drive.chargeRequired
    block[Drive.RampUp + 1] = drive.rampUpSpeed
    block[Drive.RampDown + 1] = drive.rampDownSpeed
    block[Drive.MinLength + 1] = minLength
    return block
end

---Sends the request for the path from the ship to `goal` and starts the engagement's plan state.
---@param ap AutoPilotComponent
---@param shipEntity Entity
---@param rb RigidBody
---@param profile FlightProfile
---@param goal Position The stand-off point, in the world
---@param frameEntity Entity|nil The entity the goal rides on, when it is a body of the system
---@param driveMinLength gameunit Legs shorter than this are not flown under the drive
---@param arrivalRadius gameunit How close to the goal the autopilot calls the ship arrived
function AutoPilotPlan.request(ap, shipEntity, rb, profile, goal, frameEntity, driveMinLength, arrivalRadius)
    local snapshot = PlannerSnapshot.take(NavigationComponents.CostProfile.ProfileOf(shipEntity))
    local frameIndex = frameEntity and snapshot:indexOf(frameEntity)
    local goalFrame, gx, gy, gz = SystemPlanner.SYSTEM_FRAME, goal.x, goal.y, goal.z
    if frameIndex then
        local centre = snapshot.frames[frameIndex].rb:getPos()
        goalFrame, gx, gy, gz = frameIndex - 1, goal.x - centre.x, goal.y - centre.y, goal.z - centre.z
    end
    local at = rb:getPos()
    local taskId = PlanClient.request({
        bodies = snapshot.bodies,
        count = snapshot.count,
        start = { at.x, at.y, at.z },
        goalFrame = goalFrame,
        goal = { gx, gy, gz },
        speed = profile.maxSpeed,
        accel = profile.thrustForward / profile.mass,
        drag = profile.linearDrag,
        depth = DEPTH_LIMIT,
        capacity = CAPACITY,
        drive = driveBlock(shipEntity, driveMinLength),
        arrivalRadius = arrivalRadius,
        zones = snapshot.zones,
        zoneCount = snapshot.zoneCount,
    })
    ap:setPlan({
        phase = Phase.Planning, snapshot = snapshot, taskId = taskId, start = { x = at.x, y = at.y, z = at.z },
        released = nil,
    })
end

---Makes the plan's follower for the corners of a plan.
---@param plan table The engagement's plan state
---@param result Plan
---@param spec { nav: NavWorld, agent: integer, rb: RigidBody, profile: FlightProfile }
local function follow(plan, result, spec)
    local points = {}
    for i = 1, #result.waypoints - 1 do points[i] = result.waypoints[i] end
    plan.goal = result.waypoints[#result.waypoints]
    if #points == 0 then
        plan.phase = Phase.Final
        return
    end
    -- The follower reports the end of the corners; the legs and arcs need no other note.
    plan.listener = {
        pointPassed = function() end,
        arcFlown = function() end,
        legChanged = function() end,
        finished = function() plan.phase = Phase.Final end,
    }
    plan.follower = PlanFollower.new({
        nav = spec.nav, agent = spec.agent, rb = spec.rb, frames = plan.snapshot.frames, points = points,
        goal = plan.goal, start = plan.start, passRadius = spec.profile.maxSpeed * PASS_TIME_S,
        accel = spec.profile.thrustForward / spec.profile.mass, drag = spec.profile.linearDrag,
    })
    plan.phase = Phase.Plan
end

---Takes the plan if it has come: the ship goes on to fly its corners, or, when there is no usable
---plan, straight to the goal. Call while the phase is `Planning`.
---@param ap AutoPilotComponent
---@param spec { nav: NavWorld, agent: integer, rb: RigidBody, profile: FlightProfile }
function AutoPilotPlan.poll(ap, spec)
    local plan = ap:getPlan()
    PlanClient.poll()
    local result = PlanClient.take(plan.taskId)
    if not result then return end
    plan.taskId = nil
    if not FLYABLE[result.status] then
        Log.Warn("AutoPilot: no usable plan (status %s%s), flying straight to the goal with no detours",
            tostring(result.status), result.error and (", " .. result.error) or "")
        plan.phase = Phase.Direct
        return
    end
    follow(plan, result, spec)
end

---Drops the engagement's plan; a request still out has its reply thrown away.
---@param ap AutoPilotComponent
function AutoPilotPlan.drop(ap)
    local plan = ap:getPlan()
    if plan and plan.taskId then PlanClient.discard(plan.taskId) end
    ap:setPlan(nil)
end

return AutoPilotPlan
