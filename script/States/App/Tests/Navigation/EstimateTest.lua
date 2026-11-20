-- Checks the navigation estimates: a flight in open space (with and without the travel drive), a stand-off
-- to an entity, a destination or a start inside a hull and a target that is gone, the risk of a
-- cost zone on the line and how a cost profile weighs it, the slowdown in a planet's drive zone, and the
-- batch form on a worker (the same numbers as the single form, in the targets' order, and a discarded
-- request). Logs each failed check and ends with a failure exit code.

local Application          = require("States.Application")
local Registry             = require("Core.ECS.Registry")
local CoreComponents       = require("Modules.Core.Components")
local PhysicsComponents    = require("Modules.Physics.Components")
local SpatialComponents    = require("Modules.Spatial.Components")
local SpatialEntities      = require("Modules.Spatial.Entities")
local ConstructComponents  = require("Modules.Constructs.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local Nav                  = require("Modules.Navigation")
local NavConfig            = require("Config.Navigation.NavConfig")
local CostProfileRegistry  = require("Shared.Registries.CostProfileRegistry")
local PlanFlight           = require("Shared.Helpers.PlanFlight")
local TravelDriveMath      = require("Shared.Helpers.TravelDriveMath")
local FlightScene          = require("States.App.Tests.Support.FlightScene")

---@class NavigationEstimateTest: Application
local EstimateTest = Subclass("NavigationEstimateTest", Application)

-- A bound that turns a worker that never answers into a failure.
local WORKER_WAIT_S = 60
local MS_PER_S      = 1000
-- Relative tolerance between two estimates that are computed the same way.
local SAME = 1e-9

-- Distances, in game units. A flight beyond the drive's shortest leg (about 53 for the test ship, and
-- twice `NEAR` is under it) uses the drive.
local FAR       = 20000
local NEAR      = 20
local BODY_AT   = 80
local BODY_RADIUS = 25
-- The cost zone on the line: its place along it, radius and cost per unit of length.
local ZONE_AT, ZONE_RADIUS, ZONE_COST = 5000, 400, 2.5
-- The planet that slows the drive: where it lies across the line, its radius and the speed it orbits at.
-- Early on the line, where the drive wants more than the planet's zone allows near its centre.
local PLANET_AT = { x = 3000, y = 300, z = 0 }
local PLANET_RADIUS = 50
local PLANET_SPEEDUP = 0.01

function EstimateTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("NavigationEstimateTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---@param a number|nil
---@param b number|nil
---@return boolean
local function same(a, b)
    return a ~= nil and b ~= nil and math.abs(a - b) <= SAME * math.max(1, math.abs(a), math.abs(b))
end

---A sphere cost zone on the line, with a cost and tags.
---@param at Position
---@param tag string
---@return Entity
local function costZone(at, tag)
    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Sphere)
    zone:get(SpatialComponents.Shape):setRadius(ZONE_RADIUS)
    zone:get(PhysicsComponents.Transform):setPos(at)
    zone:add(NavigationComponents.PathCost(ZONE_COST))
    zone:add(CoreComponents.Tag(tag))
    return zone
end

function EstimateTest:ship(at)
    local ship = FlightScene.spawnShip(self.world, at)
    self.world:update(1 / 60)
    return ship
end

function EstimateTest:release(things)
    FlightScene.despawn(self.world, things)
    self.world:update(1 / 60)
end

---What the travel-drive law says for a flight in open space.
---@param profile FlightProfile
---@param length gameunit The straight distance
---@param arrival gameunit
---@param driveUsed boolean
---@return number seconds
local function lawTime(profile, length, arrival, driveUsed)
    return TravelDriveMath.travelTime(length * NavConfig.estimateDetourFactor, arrival, profile.maxSpeed,
        Config.game.travelDrive.rampDownSpeed, profile.linearDrag,
        Config.game.travelDrive.maxSpeedByZone.openSpace, driveUsed)
end

function EstimateTest:caseOpenSpace()
    local name = "open space: "
    local ship = self:ship(Position(0, 0, 0))
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)
    local arrival = NavConfig.pointArrival()

    local far = Nav.estimate(ship.entity, Position(FAR, 0, 0))
    self:check(far.reachable, name .. "a point is reachable")
    self:check(same(far.distance, FAR * NavConfig.estimateDetourFactor), name .. "the distance is the line's, lengthened",
        tostring(far.distance))
    self:check(same(far.time, lawTime(profile, FAR, arrival, true)), name .. "a far point is timed under the drive",
        tostring(far.time))
    self:check(far.risk == 0, name .. "no cost zone, no risk")

    local near = Nav.estimate(ship.entity, Position(NEAR, 0, 0))
    self:check(same(near.time, lawTime(profile, NEAR, arrival, false)), name .. "a near point is flown at top speed",
        tostring(near.time))
    local twice = Nav.estimate(ship.entity, Position(2 * NEAR, 0, 0))
    self:check(near.time > 0 and twice.time > near.time, name .. "a farther point takes longer (no drive)")
    self:check(far.time < lawTime(profile, FAR, arrival, false),name .. "the drive beats top speed on a far point")

    local noDrive = Nav.estimate(Position(0, 0, 0), Position(FAR, 0, 0), profile)
    self:check(same(noDrive.time, lawTime(profile, FAR, arrival, false)), name .. "a start that is a point flies with no drive")
    self:check(noDrive.time > far.time, name .. "the drive is faster")

    local here = Nav.estimate(ship.entity, ship.rb:getPos())
    self:check(here.reachable and here.time == 0 and here.distance == 0, name .. "a ship at its goal needs no time")
    self:check(not pcall(Nav.estimate, Position(0, 0, 0), Position(NEAR, 0, 0)), name .. "a point start needs a flight profile")
    self:release({ ship })
end

function EstimateTest:caseEntityTarget()
    local name = "an entity: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(BODY_AT, 0, 0), BODY_RADIUS)
    self.world:update(1 / 60)
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)

    local standOff = PlanFlight.standOff(BODY_RADIUS, nil, NavConfig.standOffRadii)
    local result = Nav.estimate(ship.entity, body.entity)
    self:check(result.reachable and same(result.distance, (BODY_AT - standOff) * NavConfig.estimateDetourFactor),
        name .. "the path ends at the stand-off", tostring(result.distance))
    self:check(same(result.time, lawTime(profile, BODY_AT - standOff, PlanFlight.arrivalRadius(BODY_RADIUS, nil,
        NavConfig.standOffRadii), false)), name .. "and counts the stand-off's arrival radius", tostring(result.time))

    local inside = Nav.estimate(Position(BODY_AT - standOff / 2, 0, 0), body.entity, profile)
    self:check(inside.reachable and inside.time == 0, name .. "a ship already inside the stand-off has arrived")

    local gone = FlightScene.spawnBody(self.world, Position(BODY_AT, BODY_AT, 0), BODY_RADIUS)
    self:release({ gone })
    local missing = Nav.estimate(ship.entity, gone.entity)
    self:check(not missing.reachable and missing.time == nil, name .. "a target that is gone is not reachable")
    self:release({ ship, body })
end

function EstimateTest:caseHull()
    local name = "a hull: "
    local ship = self:ship(Position(0, 0, 0))
    -- A station is a body of the system, which the estimate keeps clear of.
    local station = FlightScene.spawnBody(self.world, Position(BODY_AT, 0, 0), BODY_RADIUS)
    station.entity:add(ConstructComponents.SpaceStation())
    self.world:update(1 / 60)

    local inside = Nav.estimate(ship.entity, Position(BODY_AT + BODY_RADIUS / 2, 0, 0))
    self:check(not inside.reachable and inside.time == nil and inside.distance == nil and inside.risk == nil,
        name .. "a point inside a body is not reachable")
    local outside = Nav.estimate(ship.entity, Position(BODY_AT + BODY_RADIUS * 2, 0, 0))
    self:check(outside.reachable, name .. "a point outside it is")
    local startInside = Nav.estimate(Position(BODY_AT, 0, 0), Position(0, 0, 0),
        ConstructComponents.FlightProfile.ProfileOf(ship.entity))
    self:check(not startInside.reachable, name .. "a start inside a body is not reachable")
    self:release({ ship, station })
end

function EstimateTest:caseRisk()
    local name = "a cost zone: "
    local ship = self:ship(Position(0, 0, 0))
    local zone = costZone(Position(ZONE_AT, 0, 0), "AsteroidField")
    local goal = Position(FAR, 0, 0)

    local plain = Nav.estimate(ship.entity, goal)
    self:check(same(plain.risk, ZONE_COST * 2 * ZONE_RADIUS), name .. "the risk is the cost times the chord",
        tostring(plain.risk))

    local cautious = Nav.estimate(ship.entity, goal, nil, CostProfileRegistry:get(Enums.CostProfile.Cautious))
    local weight = CostProfileRegistry:get(Enums.CostProfile.Cautious):weightOf(CoreComponents.Tag("AsteroidField"))
    self:check(weight > 1 and same(cautious.risk, plain.risk * weight), name .. "a cautious profile weighs it more",
        tostring(cautious.risk))
    local bold = Nav.estimate(ship.entity, goal, nil, CostProfileRegistry:get(Enums.CostProfile.Bold))
    self:check(bold.risk == 0, name .. "a bold profile makes it free")

    local aside = Nav.estimate(ship.entity, Position(FAR, 2 * ZONE_RADIUS * 4, 0))
    self:check(aside.risk == 0, name .. "a line that misses the zone has no risk")
    local short = Nav.estimate(ship.entity, Position(ZONE_AT, 0, 0))
    self:check(same(short.risk, ZONE_COST * ZONE_RADIUS), name .. "a line that ends in the zone counts the part inside",
        tostring(short.risk))
    Registry:destroyEntity(zone)
    self:release({ ship })
end

function EstimateTest:caseDriveZone()
    local name = "a drive zone: "
    local ship = self:ship(Position(0, 0, 0))
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)
    local goal = Position(FAR, 0, 0)
    local clear = Nav.estimate(ship.entity, goal)

    local well = FlightScene.buildWell(self.world, Position(PLANET_AT.x, PLANET_AT.y, PLANET_AT.z), PLANET_RADIUS,
        PLANET_SPEEDUP)
    self.world:update(1 / 60)
    local slowed = Nav.estimate(ship.entity, goal)
    self:check(slowed.reachable and slowed.time > clear.time, name .. "a planet's zone on the line slows the drive",
        string.format("%.1f against %.1f", slowed.time, clear.time))
    -- The zone covers a short stretch; it can't cost more than flying it at top speed, with no drive.
    local ceiling = clear.time + 2 * FlightScene.WELL_RADIUS * TravelDriveMath.INFLUENCE_RADIUS_FACTOR / profile.maxSpeed
    self:check(slowed.time < ceiling, name .. "by less than a flight across it at top speed",
        string.format("%.1f of at most %.1f", slowed.time, ceiling))
    local noDrive = Nav.estimate(Position(0, 0, 0), goal, profile)
    self:check(same(noDrive.time, lawTime(profile, FAR, NavConfig.pointArrival(), false)),
        name .. "a flight with no drive is not slowed by a zone")
    local behind = Nav.estimate(ship.entity, Position(-FAR, 0, 0))
    self:check(same(behind.time, clear.time), name .. "a line that misses the zone is not slowed")
    FlightScene.despawnWell(self.world, well)
    FlightScene.despawn(self.world, { well.planetBody })
    self:release({ ship })
end

---Waits for the estimates of a request, for at most `WORKER_WAIT_S` of real time.
---@param request integer
---@return NavEstimateResult[]|nil
local function awaitEstimates(request)
    local start = InstantTime.Now()
    while InstantTime.Now():durationSince(start) < WORKER_WAIT_S * MS_PER_S do
        local estimates = Nav.takeEstimates(request)
        if estimates then return estimates end
    end
    return nil
end

function EstimateTest:caseBatch()
    local name = "a batch: "
    local ship = self:ship(Position(0, 0, 0))
    local body = FlightScene.spawnBody(self.world, Position(BODY_AT, 0, 0), BODY_RADIUS)
    local station = FlightScene.spawnBody(self.world, Position(0, BODY_AT, 0), BODY_RADIUS)
    station.entity:add(ConstructComponents.SpaceStation())
    local gone = FlightScene.spawnBody(self.world, Position(0, 0, BODY_AT), BODY_RADIUS)
    local zone = costZone(Position(ZONE_AT, 0, 0), "AsteroidField")
    self.world:update(1 / 60)
    local targets = { Position(FAR, 0, 0), body.entity, Position(0, BODY_AT, 0), gone.entity, Position(NEAR, 0, 0) }
    self:release({ gone })

    local request = Nav.estimateMany(ship.entity, targets, nil, CostProfileRegistry:get(Enums.CostProfile.Cautious))
    local estimates = awaitEstimates(request)
    self:check(estimates ~= nil, name .. "the estimates come back")
    if estimates then
        self:check(#estimates == #targets, name .. "one estimate per target", tostring(#estimates))
        for i, target in ipairs(targets) do
            local single = Nav.estimate(ship.entity, target, nil, CostProfileRegistry:get(Enums.CostProfile.Cautious))
            local batch = estimates[i]
            self:check(batch.reachable == single.reachable and (not single.reachable or
                same(batch.time, single.time) and same(batch.distance, single.distance) and same(batch.risk, single.risk)),
                name .. "target " .. i .. " is the single estimate's")
        end
        self:check(estimates[1].reachable and not estimates[3].reachable and not estimates[4].reachable,
            name .. "the order is the targets'")
    end
    self:check(Nav.takeEstimates(request) == nil, name .. "estimates are given once")

    local dropped = Nav.estimateMany(ship.entity, targets)
    Nav.discardEstimates(dropped)
    local after = Nav.estimateMany(ship.entity, { Position(FAR, 0, 0) })
    local second = awaitEstimates(after)
    self:check(second ~= nil and #second == 1, name .. "a discarded request doesn't block the next")
    self:check(Nav.takeEstimates(dropped) == nil, name .. "a discarded request is never given")
    Registry:destroyEntity(zone)
    self:release({ ship, body, station })
end

function EstimateTest:onInit()
    self.passed, self.failed = 0, 0
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    self:caseOpenSpace()
    self:caseEntityTarget()
    self:caseHull()
    self:caseRisk()
    self:caseDriveZone()
    self:caseBatch()

    Log.Info("NavigationEstimateTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return EstimateTest
