-- Checks cost profiles and the zone buffer they weight: the profile data and its validation, the
-- weight of a zone's tags (the largest, one for what a profile doesn't name), the component's
-- revision, and the rows `CostZones.collect` and `PlannerSnapshot.take` give the planner. Logs each
-- failed check and ends with a failure exit code.

local Application          = require("States.Application")
local Entity               = require("Core.ECS.Entity")
local Registry             = require("Core.ECS.Registry")
local CoreComponents       = require("Modules.Core.Components")
local PhysicsComponents    = require("Modules.Physics.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local SpatialComponents    = require("Modules.Spatial.Components")
local SpatialEntities      = require("Modules.Spatial.Entities")
local CostProfile          = require("Shared.Types.CostProfile")
local CostProfileRegistry  = require("Shared.Registries.CostProfileRegistry")
local CostZones            = require("Shared.Helpers.CostZones")
local PlannerSnapshot      = require("Shared.Helpers.PlannerSnapshot")
local SystemPlanner        = require("Shared.Helpers.SystemPlanner")

---@class NavigationCostProfileTest: Application
local CostProfileTest = Subclass("NavigationCostProfileTest", Application)

local PATH_COST   = 2.5
local RADIUS      = 400
local CENTRE      = { x = 1000, y = 20, z = -3000 }
local CAUTIOUS    = Enums.CostProfile.Cautious
local BOLD        = Enums.CostProfile.Bold
local STANDARD    = Enums.CostProfile.Standard
local CAUTIOUS_X  = 10 -- the Cautious weight of a belt, as CostProfileDefs.lua gives it
local Zone        = SystemPlanner.Zone
local ZONE_STRIDE = SystemPlanner.ZONE_STRIDE

function CostProfileTest:check(ok, what)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("NavigationCostProfileTest: %s failed", what)
    end
end

---A sphere zone with a path cost and tags.
---@param cost number|nil
---@param ... string Tags
---@return Entity
local function makeZone(cost, ...)
    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Sphere)
    zone:get(SpatialComponents.Shape):setRadius(RADIUS)
    zone:get(PhysicsComponents.Transform):setPos(Position(CENTRE.x, CENTRE.y, CENTRE.z))
    if cost then zone:add(NavigationComponents.PathCost(cost)) end
    if select("#", ...) > 0 then zone:add(CoreComponents.Tag(...)) end
    return zone
end

local function noBodies() return nil end

---The registered profiles, and the validation of a profile: unknown tags, bad weights, no name, no id.
function CostProfileTest:checkData()
    self:check(CostProfileRegistry:has(STANDARD) and CostProfileRegistry:has(CAUTIOUS) and CostProfileRegistry:has(BOLD),
        "the three profiles are registered")
    self:check(CostProfileRegistry:get(CAUTIOUS).name == "Cautious", "a registered profile keeps its name")
    self:check(not pcall(CostProfile, { name = "Typo", weights = { Asteroids = 2 } }), "an unknown tag is rejected")
    self:check(not pcall(CostProfile, { name = "Negative", weights = { AsteroidField = -1 } }),
        "a negative weight is rejected")
    self:check(not pcall(CostProfile, { name = "Infinite", weights = { AsteroidField = math.huge } }),
        "an infinite weight is rejected")
    self:check(CostProfile({ weights = {} }) == nil, "a profile with no name is not made")
    local inline = CostProfile { name = "Inline", weights = { HostileSpace = 3 } }
    self:check(inline.id == nil and not CostProfileRegistry:has(inline.id or 0), "a profile with no id is not registered")
end

---The weight of a zone's tags: the largest the profile gives, and one for a tag it doesn't name or a zone
---with no tags.
function CostProfileTest:checkWeights()
    local standard = CostProfileRegistry:get(STANDARD)
    local cautious = CostProfileRegistry:get(CAUTIOUS)
    local bold = CostProfileRegistry:get(BOLD)
    local belt = CoreComponents.Tag("AsteroidField")
    local belt_and_well = CoreComponents.Tag("AsteroidField", "GravityWell")
    local well = CoreComponents.Tag("GravityWell")

    self:check(cautious:weightOf(belt) == CAUTIOUS_X, "a tag the profile names takes its weight")
    self:check(bold:weightOf(belt) == 0, "a weight of zero is kept")
    self:check(standard:weightOf(belt) == 1, "a profile that names nothing weighs a tag at one")
    self:check(cautious:weightOf(well) == 1, "a tag the profile doesn't name weighs one")
    self:check(cautious:weightOf(nil) == 1 and bold:weightOf(nil) == 1, "a zone with no tags weighs one")
    self:check(bold:weightOf(belt_and_well) == 1, "the largest weight of the tags applies: an unnamed tag beats zero")
    self:check(cautious:weightOf(belt_and_well) == CAUTIOUS_X, "the largest weight applies for a named tag too")
end

---A ship's cost profile component: it keeps its profile, a change raises its revision, and `ProfileOf`
---finds it (nil for a ship with none).
function CostProfileTest:checkComponent()
    local standard = CostProfileRegistry:get(STANDARD)
    local component = NavigationComponents.CostProfile(standard)
    self:check(component:getProfile() == standard, "the component keeps its profile")
    local before = component:getRevision()
    component:setProfile(CostProfileRegistry:get(BOLD))
    self:check(component:getRevision() == before + 1, "a change of profile raises the revision")
    self:check(component:getProfile() == CostProfileRegistry:get(BOLD), "a change of profile is kept")
    self:check(not pcall(NavigationComponents.CostProfile, nil), "a missing profile is rejected")

    local ship = Entity.Create("CostProfileTestShip", NavigationComponents.CostProfile(standard))
    self:check(NavigationComponents.CostProfile.ProfileOf(ship) == standard, "ProfileOf finds a ship's profile")
    local bare = Entity.Create("CostProfileTestBare")
    self:check(NavigationComponents.CostProfile.ProfileOf(bare) == nil, "ProfileOf gives nil for a ship with none")
end

---The row of zone `index` (1-based) of a buffer.
local function row(values, index)
    local base = (index - 1) * ZONE_STRIDE
    return {
        frame = values[base + Zone.Frame + 1], x = values[base + Zone.X + 1], y = values[base + Zone.Y + 1],
        z = values[base + Zone.Z + 1], radius = values[base + Zone.Radius + 1], cost = values[base + Zone.Cost + 1],
    }
end

---The planner rows `CostZones.collect` gives: only spheres with a path cost, the cost weighed by the
---profile, a zone on a body riding it, and the errors for other shapes.
function CostProfileTest:checkRows()
    local belt = makeZone(PATH_COST, "AsteroidField")
    local untagged = makeZone(1)
    local plain = makeZone(nil, "AsteroidField") -- no path cost: the planner never sees it

    local values, count = CostZones.collect(CostProfileRegistry:get(CAUTIOUS), noBodies)
    self:check(count == 2 and #values == 2 * ZONE_STRIDE, "only zones with a path cost are rows")
    local first = row(values, 1)
    self:check(first.frame == SystemPlanner.SYSTEM_FRAME and first.x == CENTRE.x and first.y == CENTRE.y
        and first.z == CENTRE.z, "a free zone is a point of the system")
    self:check(first.radius == RADIUS, "the row has the zone's radius")
    local rows = { row(values, 1), row(values, 2) }
    local costs = { [rows[1].cost] = true, [rows[2].cost] = true }
    self:check(costs[PATH_COST * CAUTIOUS_X] and costs[1], "the cost is the path cost times the profile's weight")

    values = CostZones.collect(CostProfileRegistry:get(BOLD), noBodies)
    local bold = { [row(values, 1).cost] = true, [row(values, 2).cost] = true }
    self:check(bold[0] and bold[1], "a profile that weighs a tag at zero makes the zone free")
    values = CostZones.collect(nil, noBodies)
    local none = { [row(values, 1).cost] = true, [row(values, 2).cost] = true }
    self:check(none[PATH_COST] and none[1], "a ship with no profile takes the path cost as it is")
    self:check(select(2, CostZones.collect(nil, noBodies)) == 2, "a zone that has no path cost stays out whatever the profile")

    -- A zone attached to a body rides on it, with no offset.
    local body = Entity.Create("CostProfileTestBody")
    Registry:attachEntity(body, belt)
    values, count = CostZones.collect(nil, function(entity) return entity.id == body.id and 3 or nil end)
    local frames = { [row(values, 1).frame] = true, [row(values, 2).frame] = true }
    self:check(count == 2 and frames[2] and frames[SystemPlanner.SYSTEM_FRAME], "a zone on a body rides on it")
    for i = 1, count do
        local r = row(values, i)
        if r.frame == 2 then
            self:check(r.x == 0 and r.y == 0 and r.z == 0, "a zone on a body has no offset from it")
        end
    end

    -- The shapes: only a sphere.
    plain:add(NavigationComponents.PathCost(1))
    plain:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Cube)
    self:check(not pcall(CostZones.collect, nil, noBodies), "a path cost on a zone that is not a sphere is an error")
    plain:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Sphere)
    plain:get(SpatialComponents.Shape):setRadius(0)
    self:check(not pcall(CostZones.collect, nil, noBodies), "a path cost on a zone with no radius is an error")
    Registry:destroyEntity(plain)
    Registry:destroyEntity(untagged)
    Registry:destroyEntity(belt)
end

---A planner snapshot carries the zones weighed by the ship's profile, as authored with none.
function CostProfileTest:checkSnapshot()
    local zone = makeZone(PATH_COST, "AsteroidField")
    local withProfile = PlannerSnapshot.take(CostProfileRegistry:get(CAUTIOUS))
    self:check(withProfile.zoneCount == 1 and withProfile.zones[Zone.Cost + 1] == PATH_COST * CAUTIOUS_X,
        "the snapshot's zones are weighted by the profile")
    local without = PlannerSnapshot.take(nil)
    self:check(without.zoneCount == 1 and without.zones[Zone.Cost + 1] == PATH_COST,
        "the snapshot's zones take the path cost with no profile")
    Registry:destroyEntity(zone)
    self:check(PlannerSnapshot.take(nil).zoneCount == 0, "a snapshot of a system with no cost zone has no zones")
end

function CostProfileTest:onInit()
    self.passed, self.failed = 0, 0

    self:checkData()
    self:checkWeights()
    self:checkComponent()
    self:checkRows()
    self:checkSnapshot()

    Log.Info("NavigationCostProfileTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return CostProfileTest
