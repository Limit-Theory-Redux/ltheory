-- Builds the input of a navigation estimate from the game: the bodies and cost zones of the system
-- where they are now, the ship's numbers, and the stand-off point of each target. What it makes is
-- plain numbers, so `NavEstimate` runs on it inline or on a worker.
-- Design: doc/script/nav/navigation.md

local ConstructComponents  = require("Modules.Constructs.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local NavConfig            = require("Config.Navigation.NavConfig")
local EntityPosition       = require("Shared.Helpers.EntityPosition")
local NavEstimate          = require("Shared.Helpers.NavEstimate")
local OrderGoals           = require("Shared.Helpers.OrderGoals")
local PlanFlight           = require("Shared.Helpers.PlanFlight")
local PlannerSnapshot      = require("Shared.Helpers.PlannerSnapshot")
local SystemPlanner        = require("Shared.Helpers.SystemPlanner")

local PlannerBody, PlannerZone = SystemPlanner.Body, SystemPlanner.Zone
local EstBody, EstZone, EstGoal = NavEstimate.Body, NavEstimate.Zone, NavEstimate.Goal

---@class EstimateRequest
local EstimateRequest = {}

---@param entity Entity|Position
---@return boolean
local function isEntity(entity)
    return rawtype(entity) == "table" and entity.isValid ~= nil
end

---The scene: each body's centre now, hull and drive zone, and each cost zone's centre now.
---@param snapshot PlannerSnapshot
---@return NavEstimateScene
local function sceneOf(snapshot)
    local bodies, centres = {}, {}
    for i = 1, snapshot.count do
        local at = snapshot.frames[i].rb:getPos()
        centres[i] = at
        local row, base = {}, (i - 1) * SystemPlanner.BODY_STRIDE + 1
        row[EstBody.X + 1], row[EstBody.Y + 1], row[EstBody.Z + 1] = at.x, at.y, at.z
        row[EstBody.Hull + 1] = snapshot.bodies[base + PlannerBody.Radius]
        row[EstBody.ZoneRadius + 1] = snapshot.bodies[base + PlannerBody.ZoneRadius]
        row[EstBody.ZoneCap + 1] = snapshot.bodies[base + PlannerBody.ZoneCap]
        for k = 1, NavEstimate.BODY_STRIDE do bodies[#bodies + 1] = row[k] end
    end
    local zones = {}
    for i = 1, snapshot.zoneCount do
        local base = (i - 1) * SystemPlanner.ZONE_STRIDE + 1
        local frame = snapshot.zones[base + PlannerZone.Frame]
        -- A zone on a body has no offset from it; a free zone's numbers are the system's.
        local at = frame ~= SystemPlanner.SYSTEM_FRAME and centres[frame + 1] or
            { x = snapshot.zones[base + PlannerZone.X], y = snapshot.zones[base + PlannerZone.Y],
              z = snapshot.zones[base + PlannerZone.Z] }
        local row = {}
        row[EstZone.X + 1], row[EstZone.Y + 1], row[EstZone.Z + 1] = at.x, at.y, at.z
        row[EstZone.Radius + 1] = snapshot.zones[base + PlannerZone.Radius]
        row[EstZone.Cost + 1] = snapshot.zones[base + PlannerZone.Cost]
        for k = 1, NavEstimate.ZONE_STRIDE do zones[#zones + 1] = row[k] end
    end
    return { bodies = bodies, bodyCount = snapshot.count, zones = zones, zoneCount = snapshot.zoneCount }
end

---The ship's numbers. The drive counts for a ship that has one, when the game uses it.
---@param from Entity|Position
---@param profile FlightProfile
---@return NavEstimateShip
local function shipOf(from, profile)
    local ship = { maxSpeed = profile.maxSpeed, linearDrag = profile.linearDrag, detourFactor = NavConfig.estimateDetourFactor }
    local driven = isEntity(from) and from:get(ConstructComponents.TravelDrive)
        and Config.game.autoPilot.enableTravelDrive ~= false
    if driven then
        ship.openCap = Config.game.travelDrive.maxSpeedByZone.openSpace
        ship.rampDownSpeed = Config.game.travelDrive.rampDownSpeed
    end
    return ship
end

---The goals: where a ship stops for each target. A target that is gone has no goal.
---@param start Position
---@param targets NavTarget[]
---@param profile FlightProfile
---@return number[] goals `NavEstimate.GOAL_STRIDE` numbers per target
local function goalsOf(start, targets, profile)
    local goals = {}
    local driveMinLength = PlanFlight.driveMinLength(profile)
    for _, target in ipairs(targets) do
        local row = {}
        if isEntity(target) and not target:isValid() then
            row[EstGoal.X + 1], row[EstGoal.Y + 1], row[EstGoal.Z + 1] = start.x, start.y, start.z
            row[EstGoal.ArrivalRadius + 1], row[EstGoal.DriveMinLength + 1] = NavEstimate.NO_GOAL, 0
        else
            local goal = OrderGoals.moveTo(target)
            local dist = start:distance(goal.targetPos)
            local stop = PlanFlight.standOffGoal(start, goal.targetPos, dist, dist, goal.safeRange)
            row[EstGoal.X + 1], row[EstGoal.Y + 1], row[EstGoal.Z + 1] = stop.x, stop.y, stop.z
            row[EstGoal.ArrivalRadius + 1] = goal.arrivalRadius
            row[EstGoal.DriveMinLength + 1] = driveMinLength
        end
        for k = 1, NavEstimate.GOAL_STRIDE do goals[#goals + 1] = row[k] end
    end
    return goals
end

---The scene of an estimate now (the bodies and cost zones where they are), for code that estimates
---many flights itself, on a worker (`JobPlanner`).
---@param costProfile CostProfile|nil None weighs each zone at its own cost
---@return NavEstimateScene
function EstimateRequest.scene(costProfile)
    return sceneOf(PlannerSnapshot.take(costProfile))
end

---The request for the estimates of flights from `from` to each target.
---@param from NavTarget The start: a ship (its flight and cost profile and drive are the defaults) or a point
---@param targets NavTarget[]
---@param flightProfile FlightProfile|nil The ship's, when `from` is a ship
---@param costProfile CostProfile|nil The ship's, when `from` is a ship and has one; none weighs each zone at its own cost
---@return NavEstimateRequest
function EstimateRequest.build(from, targets, flightProfile, costProfile)
    local fromEntity = isEntity(from)
    local profile = flightProfile
    if not profile then
        assert(fromEntity, "Nav.estimate: a start that is a point needs a flight profile")
        profile = ConstructComponents.FlightProfile.ProfileOf(from)
    end
    costProfile = costProfile or (fromEntity and NavigationComponents.CostProfile.ProfileOf(from)) or nil
    local start = fromEntity and EntityPosition.Get(from) or Position(from.x, from.y, from.z)

    local request = sceneOf(PlannerSnapshot.take(costProfile))
    request.ship = shipOf(from, profile)
    request.start = { start.x, start.y, start.z }
    request.goals = goalsOf(start, targets, profile)
    request.goalCount = #targets
    return request
end

return EstimateRequest
