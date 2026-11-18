-- The flight to a goal under a system plan, shared by every ship that flies one: the stand-off
-- contract, the request for the path and the plan state that comes of it, the phase of the flight,
-- and the travel drive on the plan's legs and on the final approach. The caller owns the NavWorld,
-- steps it, and writes the axes; this module decides the goals and the drive.
-- Design: doc/script/nav/system-planner.md

local ConstructComponents  = require("Modules.Constructs.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local PhysicsComponents    = require("Modules.Physics.Components")
local PlanClient           = require("Shared.Helpers.PlanClient")
local PlanFollower         = require("Shared.Helpers.PlanFollower")
local PlannerSnapshot      = require("Shared.Helpers.PlannerSnapshot")
local SystemPlanner        = require("Shared.Helpers.SystemPlanner")
local TravelDriveMath      = require("Shared.Helpers.TravelDriveMath")
local NavConfig            = require("Config.Navigation.NavConfig")

-- How many times a leg may be split; the plan holds at most one waypoint per leg, and the goal.
local DEPTH_LIMIT = 5
local CAPACITY    = 2 ^ DEPTH_LIMIT

-- A corner is reached within the distance the ship covers at its top speed in this many seconds:
-- longer than a frame's move, and well inside the corner's arc (whose radius is the planner's turn
-- radius at the corner's speed, so a wider reach would put the ship off the arc's circle).
local PASS_TIME_S = 0.1

-- Below this drive multiplier the drive is released and the NavWorld flies the rest.
local HANDOVER_MULT = TravelDriveMath.HANDOVER_MULT

-- The ship has arrived when it is within this share of the gap between the target's hull and the
-- stand-off distance from the stand-off point, so a ship that coasts in afterwards stops outside the hull.
local ARRIVAL_SHARE = 0.5
-- A stand-off given in radii is at least this far from the target's centre.
local MIN_STAND_OFF = 50
-- The drive engages only beyond this many stand-off distances from the target.
local DRIVE_ENGAGE_STAND_OFFS = 100

local Status = SystemPlanner.Status
local Drive  = SystemPlanner.Drive

---The phases of a flight. `Planning`: the request is out, and the ship steers straight to the
---goal in normal flight. `Plan`: the corners of the plan are flown. `Final`: the way to the
---goal from the last corner (or from the start, when the plan has no corner). `Direct`: there is no
---usable plan, and the ship flies straight to the goal as in `Final`, with no detours.
local Phase = { Planning = "planning", Plan = "plan", Final = "final", Direct = "direct" }

-- The plans that are flown. A plan with another status has a leg that is left straight or no path.
local FLYABLE = { [Status.Ok] = true, [Status.GoalNotSettled] = true, [Status.DriveNotTimed] = true }

---The state of one flight under a plan: its phase, the snapshot the plan was made from, the
---follower of its corners, and the drive leg already released.
---@class PlanFlightState
---@field phase string A `PlanFlight.Phase`
---@field snapshot PlannerSnapshot|nil
---@field taskId integer|nil The request that is out
---@field start table|nil { x, y, z } where the ship was when the plan was asked for
---@field goal PlanWaypoint|nil The plan's last waypoint
---@field follower PlanFollower|nil
---@field listener table|nil
---@field released integer|nil The corner whose drive run was released

---@class PlanFlight
local PlanFlight = { Phase = Phase }

---The distance from a target's centre at which a ship stops.
---@param targetRadius gameunit 0 for a point
---@param gap gameunit|nil The wanted distance from the target's hull; `radii` decide when nil
---@param radii number|nil The wanted distance in target radii, at least `MIN_STAND_OFF`; `NavConfig.standOffRadii` when nil
---@return gameunit
function PlanFlight.standOff(targetRadius, gap, radii)
    if gap then return targetRadius + gap end
    return math.max(MIN_STAND_OFF, targetRadius * (radii or NavConfig.standOffRadii))
end

---How far from its stand-off point a ship counts as arrived.
---@param targetRadius gameunit 0 for a point
---@param gap gameunit|nil As in `standOff`
---@param radii number|nil As in `standOff`
---@return gameunit
function PlanFlight.arrivalRadius(targetRadius, gap, radii)
    return ARRIVAL_SHARE * (PlanFlight.standOff(targetRadius, gap, radii) - targetRadius)
end

---The shortest leg the travel drive is used on: beyond this many stand-off distances, and a goal
---with a smaller stand-off than the default (a point) counts the default.
---@param safeRange gameunit The stand-off distance
---@return gameunit
function PlanFlight.driveMinLength(safeRange)
    return DRIVE_ENGAGE_STAND_OFFS * math.max(safeRange, MIN_STAND_OFF)
end

---Radius of the target for the stand-off distance.
---@param targetEntity Entity|nil
---@return gameunit
function PlanFlight.targetRadius(targetEntity)
    if not targetEntity then return 0 end
    local rbCmp = targetEntity:get(PhysicsComponents.RigidBody)
    if rbCmp and rbCmp:getRigidBody() then
        return rbCmp:getRigidBody():getBoundingRadius()
    end
    local transform = targetEntity:get(PhysicsComponents.Transform)
    return transform and transform:getScale() or 0
end

---The goal a ship flies to: the point on the line from the steering point to the ship that is
---`safeRange` from the steering point. A ship that is already within `safeRange` of its target has
---arrived where it is.
---@param shipPos Position
---@param steerPos Position The point the ship steers at (the target, or its lead intercept)
---@param steerDist number Distance from the ship to `steerPos`
---@param targetDist number Distance from the ship to the target itself
---@param safeRange gameunit Stand-off distance from the target's centre
---@return Position goal
function PlanFlight.standOffGoal(shipPos, steerPos, steerDist, targetDist, safeRange)
    if targetDist <= safeRange or steerDist <= safeRange then
        return shipPos
    end
    local back = safeRange / steerDist
    return Position(
        steerPos.x + (shipPos.x - steerPos.x) * back,
        steerPos.y + (shipPos.y - steerPos.y) * back,
        steerPos.z + (shipPos.z - steerPos.z) * back)
end

---Fly the agent to `goal`: a point that moves with `frameBody` keeps its place relative to the
---body, so the NavWorld matches the body's velocity; a bare point stays where it is.
---@param nav NavWorld
---@param agent integer
---@param frameBody RigidBody|nil
---@param goal Position
---@param arrivalRadius gameunit
function PlanFlight.setGoal(nav, agent, frameBody, goal, arrivalRadius)
    if not frameBody then
        nav:setArrive(agent, goal, arrivalRadius)
        return
    end
    local bodyPos = frameBody:getPos()
    nav:setArriveRelative(agent, frameBody,
        Vec3d(goal.x - bodyPos.x, goal.y - bodyPos.y, goal.z - bodyPos.z), arrivalRadius)
end

---The drive multiplier the braking law wants at a distance from the goal (`TravelDriveMath`).
---@param goalDist gameunit
---@param profile FlightProfile
---@return number
local function brakingMultiplier(goalDist, profile)
    local lag = TravelDriveMath.brakingTime(Config.game.travelDrive.rampDownSpeed, profile.linearDrag)
    return TravelDriveMath.brakingMultiplier(goalDist, profile.maxSpeed, lag)
end

---Seconds until the ship is within `arrivalRadius` of the goal, from the same law that
---`updateDrive` flies (`TravelDriveMath.travelTime`).
---@param goalDist gameunit Distance to the goal, 0 or more
---@param arrivalRadius gameunit
---@param profile FlightProfile
---@param zoneMax number The drive multiplier the zone allows
---@param driveUsed boolean Whether the drive is, or will be, engaged
---@return number seconds
function PlanFlight.travelTime(goalDist, arrivalRadius, profile, zoneMax, driveUsed)
    return TravelDriveMath.travelTime(goalDist, arrivalRadius, profile.maxSpeed,
        Config.game.travelDrive.rampDownSpeed, profile.linearDrag, zoneMax, driveUsed)
end

---Is the drive engaged, or far enough away that it will be?
---@param drive TravelDriveComponent|nil
---@param dist gameunit Distance to the target's centre
---@param safeRange gameunit
---@return boolean
function PlanFlight.driveWillRun(drive, dist, safeRange)
    local cfg = Config.game.autoPilot or {}
    if not drive or cfg.enableTravelDrive == false then return false end
    return drive:isActive() or dist > PlanFlight.driveMinLength(safeRange)
end

---Drive the travel drive from the braking law: the cap follows the distance to the goal, and the
---drive is released for the final approach, which the NavWorld flies at normal speed.
---@param drive TravelDriveComponent
---@param input ControlInputComponent
---@param profile FlightProfile
---@param zoneMax number
---@param dist gameunit Distance to the target's centre
---@param safeRange gameunit
function PlanFlight.updateDrive(drive, input, profile, zoneMax, dist, safeRange)
    local wanted = brakingMultiplier(math.max(0, dist - safeRange), profile)
    -- The drive system ramps its multiplier toward this cap over time.
    input:setDriveSpeedCap(TravelDriveMath.cappedMultiplier(wanted, zoneMax))
    if drive:getState() ~= "idle" and wanted < HANDOVER_MULT then
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
    end

    local cfg = Config.game.autoPilot or {}
    if cfg.enableTravelDrive ~= false and dist > PlanFlight.driveMinLength(safeRange)
        and drive:getState() == "idle" then
        input:setDriveRequest(Enums.TravelDriveRequest.Engage)
    end
end

---Runs the drive on a leg of the plan: engaged at the leg's start, its cap following the braking
---law to the end of the drive run, released for the end of the run and not engaged again on that
---leg. A run goes on through a waypoint the plan marks `through`, so the drive is kept there and
---the law brakes on the distance to the run's end. A leg that the plan does not fly under the
---drive, and an arc, are flown with it off.
---@param plan PlanFlightState
---@param drive TravelDriveComponent
---@param input ControlInputComponent
---@param profile FlightProfile
---@param zoneMax number
function PlanFlight.driveLeg(plan, drive, input, profile, zoneMax)
    local follower = plan.follower
    local state = drive:getState()
    if not follower:drives() then
        if state ~= "idle" then input:setDriveRequest(Enums.TravelDriveRequest.Disengage) end
        return
    end
    local wanted = brakingMultiplier(follower:runRemaining(), profile)
    input:setDriveSpeedCap(TravelDriveMath.cappedMultiplier(wanted, zoneMax))
    if state ~= "idle" and wanted < HANDOVER_MULT then
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
        plan.released = follower.index
    elseif state == "idle" and wanted >= HANDOVER_MULT and plan.released ~= follower.index then
        input:setDriveRequest(Enums.TravelDriveRequest.Engage)
    end
end

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

---Sends the request for the path from the ship to `goal` and starts the flight's plan state.
---@param shipEntity Entity
---@param rb RigidBody
---@param profile FlightProfile
---@param goal Position The stand-off point, in the world
---@param frameEntity Entity|nil The entity the goal rides on, when it is a body of the system
---@param driveMinLength gameunit|nil Legs shorter than this are not flown under the drive; nil asks for no drive
---@param arrivalRadius gameunit How close to the goal the ship counts as arrived
---@return PlanFlightState
local function request(shipEntity, rb, profile, goal, frameEntity, driveMinLength, arrivalRadius)
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
        drive = driveMinLength and driveBlock(shipEntity, driveMinLength) or nil,
        arrivalRadius = arrivalRadius,
        zones = snapshot.zones,
        zoneCount = snapshot.zoneCount,
    })
    return { phase = Phase.Planning, snapshot = snapshot, taskId = taskId, start = { x = at.x, y = at.y, z = at.z } }
end

---Starts a flight to `targetPos`: asks the planner for the path from the ship to the stand-off
---point. A ship already within the stand-off distance has no path to plan.
---@param shipEntity Entity
---@param rb RigidBody
---@param targetPos Position|nil The target now; nil when there is none, which flies nothing
---@param safeRange gameunit The stand-off distance from the target's centre; 0 stops at the target
---@param arrivalRadius gameunit
---@param frameEntity Entity|nil The entity the goal rides on
---@param withoutDrive boolean|nil Plan the path with no leg under the travel drive
---@return PlanFlightState
function PlanFlight.begin(shipEntity, rb, targetPos, safeRange, arrivalRadius, frameEntity, withoutDrive)
    local shipPos = rb:getPos()
    local dist = targetPos and shipPos:distance(targetPos) or 0
    -- Within the stand-off distance, or within arrival of a goal that has none, there is nothing to plan.
    local range = math.max(safeRange, arrivalRadius)
    if not targetPos or dist <= range then
        return { phase = Phase.Final }
    end
    local profile = ConstructComponents.FlightProfile.ProfileOf(shipEntity)
    return request(shipEntity, rb, profile, PlanFlight.standOffGoal(shipPos, targetPos, dist, dist, safeRange),
        frameEntity, not withoutDrive and PlanFlight.driveMinLength(safeRange) or nil, arrivalRadius)
end

---Makes the plan's follower for the corners of a plan.
---@param plan PlanFlightState
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
---@param plan PlanFlightState
---@param spec { nav: NavWorld, agent: integer, rb: RigidBody, profile: FlightProfile }
function PlanFlight.poll(plan, spec)
    PlanClient.poll()
    local result = PlanClient.take(plan.taskId)
    if not result then return end
    plan.taskId = nil
    if not FLYABLE[result.status] then
        Log.Warn("PlanFlight: no usable plan (status %s%s), flying straight to the goal with no detours",
            tostring(result.status), result.error and (", " .. result.error) or "")
        plan.phase = Phase.Direct
        return
    end
    follow(plan, result, spec)
end

---Drops a flight's plan; a request still out has its reply thrown away.
---@param plan PlanFlightState|nil
function PlanFlight.drop(plan)
    if plan and plan.taskId then PlanClient.discard(plan.taskId) end
end

return PlanFlight
