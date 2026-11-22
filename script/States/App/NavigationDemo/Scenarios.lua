-- The scenarios of the navigation demo: each is a group of ships that fly one order or behaviour of
-- the navigation API in the authored system. A scenario gives the position a ship of its starts at and
-- the order it flies at each step; the demo gives the next order when one is done, so a scenario runs
-- for as long as the app does. Distances are metres, converted with `Scene.toGame`.

local Nav                = require("Modules.Navigation")
local PhysicsComponents  = require("Modules.Physics.Components")
local EntityPosition     = require("Shared.Helpers.EntityPosition")
local Scene              = require("States.App.NavigationDemo.Scene")

local toGame = Scene.toGame

-- The gap between the ships of a group at the start.
local SPACING_M = 1000

-- Two points across the system, the star between them: the plan goes round it, with the travel drive.
local FAR_A = Scene.pointAt(0, 3.5e9)
local FAR_B = Scene.pointAt(180, 9.0e9)

-- The same for the group that follows its leader.
local FOLLOW_A = Scene.pointAt(300, 3.0e9)
local FOLLOW_B = Scene.pointAt(240, 6.5e9)
-- Where a follower keeps from its leader: right, up, forward, metres.
local FOLLOW_OFFSETS_M = {
    { 150, 0, -100 },
    { -150, 0, -100 },
    { 0, 100, -250 },
}

-- A square of open space the chased leader patrols, and its start.
local CHASE_BASE = Scene.pointAt(30, 3.0e9)
local CHASE_SIDES_M = { 100e3, 60e3 }
local CHASE_GAP_M = 1500

-- The leader of the chase through the rock cluster patrols this many points of it; each keeps this
-- clear of every rock, and the start is outside the cluster at this angle and multiple of its extent.
local FIELD_PATROL_POINTS = 4
local FIELD_PATROL_MARGIN_M = 500
local FIELD_START_ANGLE_DEG = 180
local FIELD_START_FACTOR = 1.6
local FIELD_POINT_SEED = 7

-- The patrol round the cluster: corners of a square this many times its extent.
local PERIMETER_ANGLES_DEG = { 45, 135, 225, 315 }
local PERIMETER_FACTOR = 2.0

-- The ships orbit this body in order of slot; each starts this many of its radii from it.
local ORBIT_START_RADII = 2.5

-- The holding group's place: a planet's well carries them along with it.
local HOLD_ANGLE_DEG = 330
local HOLD_DISTANCE_M = 7.5e7

-- The traffic: pairs of points ships fly between, in open space, centred on this.
local TRAFFIC_CENTRE = Scene.pointAt(330, 3.2e9)
local TRAFFIC_PAIRS_M = {
    { { -100e3, 0, 0 }, { 100e3, 0, 200 } },
    { { 100e3, 0, 200 }, { -100e3, 0, 0 } },
    { { 0, 0, -100e3 }, { 0, 0, 100e3 } },
    { { 0, 0, 100e3 }, { 0, 0, -100e3 } },
    { { -70e3, 0, -70e3 }, { 70e3, 0, 70e3 } },
    { { 70e3, 0, 70e3 }, { -70e3, 0, -70e3 } },
}

---@param position Position
---@param dxM number
---@param dyM number|nil
---@param dzM number|nil
---@return Position
local function shift(position, dxM, dyM, dzM)
    return Position(position.x + toGame(dxM), position.y + toGame(dyM or 0), position.z + toGame(dzM or 0))
end

---@param entity Entity
---@return gameunit
local function radiusOf(entity)
    return entity:get(PhysicsComponents.Transform):getScale()
end

---The order of a ship that flies between two points: to `b` at its first step, then back and forth.
---@param ship NavigationDemoShip
---@param a Position
---@param b Position
local function betweenPoints(ship, a, b)
    return Nav.Orders.MoveTo { target = ship.step % 2 == 1 and b or a }
end

---A value made once for a scenario and kept for the run.
---@param ctx NavigationDemoContext
---@param scenario NavigationDemoScenario
---@param build fun(): any
---@return any
local function made(ctx, scenario, build)
    local value = ctx.made[scenario.name]
    if value == nil then
        value = build()
        ctx.made[scenario.name] = value
    end
    return value
end

---@class NavigationDemoContext
---@field scene NavigationDemoScene
---@field made table<string, any> What the scenarios made once
---@field leaderOf fun(ship: NavigationDemoShip): NavigationDemoShip The leader of the ship's scenario

---@class NavigationDemoScenario
---@field name string
---@field orders string The orders it shows, for the label
---@field count integer Ships
---@field leader integer|nil The slot of the ship the others follow or chase
---@field spawn fun(ctx: NavigationDemoContext, slot: integer): Position Where the ship of a slot starts
---@field order fun(ctx: NavigationDemoContext, ship: NavigationDemoShip): table The order of the ship's current step
---@type NavigationDemoScenario[]
local Scenarios = {}

local farCrossing = {
    name = "Far crossing", orders = "MoveTo a point, with the travel drive, round the star",
    count = 3,
    spawn = function(_, slot) return shift(FAR_A, 0, 0, (slot - 1) * SPACING_M) end,
    order = function(_, ship)
        local offset = (ship.slot - 1) * SPACING_M
        return betweenPoints(ship, shift(FAR_A, 0, 0, offset), shift(FAR_B, 0, 0, offset))
    end,
}

local function bodyTargets(ctx)
    return { ctx.scene.moons.Luna, ctx.scene.moons.Pebble, ctx.scene.station }
end

local moonsAndStation = {
    name = "Moons and the station", orders = "MoveTo a moving body (an entity), in turn",
    count = 3,
    spawn = function(ctx, slot)
        return shift(Scene.around(EntityPosition.Get(ctx.scene.planets.Aurora), 290, 2.5e7), 0, 0,
            (slot - 1) * SPACING_M)
    end,
    order = function(ctx, ship)
        local targets = bodyTargets(ctx)
        return Nav.Orders.MoveTo { target = targets[(ship.slot + ship.step - 2) % #targets + 1] }
    end,
}

local chase = {
    name = "Chase", orders = "MoveTo a moving ship; the leader patrols",
    count = 4, leader = 1,
    spawn = function(_, slot) return shift(CHASE_BASE, -(slot - 1) * CHASE_GAP_M) end,
    order = function(ctx, ship)
        if ship.slot == 1 then
            local side = CHASE_SIDES_M
            return Nav.Orders.Patrol {
                loop = true,
                points = {
                    CHASE_BASE, shift(CHASE_BASE, side[1], 0, 0),
                    shift(CHASE_BASE, side[1], 0, side[2]), shift(CHASE_BASE, 0, 0, side[2]),
                },
            }
        end
        return Nav.Orders.MoveTo { target = ctx.leaderOf(ship).entity }
    end,
}

local chaseInField
chaseInField = {
    name = "Chase through the rocks", orders = "MoveTo a moving ship through the rock cluster; the leader patrols it",
    count = 3, leader = 1,
    spawn = function(ctx, slot)
        local start = Scene.aroundField(ctx.scene, FIELD_START_ANGLE_DEG, FIELD_START_FACTOR)
        return shift(start, -(slot - 1) * CHASE_GAP_M)
    end,
    order = function(ctx, ship)
        if ship.slot == 1 then
            local points = made(ctx, chaseInField, function()
                local rng = RNG.Create(FIELD_POINT_SEED)
                local found = {}
                for i = 1, FIELD_PATROL_POINTS do
                    found[i] = Scene.freePointInField(ctx.scene, rng, FIELD_PATROL_MARGIN_M)
                end
                return found
            end)
            return Nav.Orders.Patrol { loop = true, points = points }
        end
        return Nav.Orders.MoveTo { target = ctx.leaderOf(ship).entity }
    end,
}

---The corners of the perimeter, from the one at `first`, in the direction of `direction` (1 or -1).
---@param ctx NavigationDemoContext
---@param first integer
---@param direction integer
---@return Position[]
local function perimeter(ctx, first, direction)
    local count = #PERIMETER_ANGLES_DEG
    local points = {}
    for i = 0, count - 1 do
        local index = (first - 1 + direction * i) % count + 1
        points[#points + 1] = Scene.aroundField(ctx.scene, PERIMETER_ANGLES_DEG[index], PERIMETER_FACTOR)
    end
    return points
end

local patrol = {
    name = "Patrol", orders = "Patrol a loop round the rock cluster, in both directions",
    count = 3,
    spawn = function(ctx, slot)
        return shift(perimeter(ctx, slot, 1)[1], 0, (slot - 1) * SPACING_M, 0)
    end,
    order = function(ctx, ship)
        return Nav.Orders.Patrol { loop = true, points = perimeter(ctx, ship.slot, ship.slot == 3 and -1 or 1) }
    end,
}

local function orbitTargets(ctx)
    return { ctx.scene.planets.Aurora, ctx.scene.moons.Luna, ctx.scene.planets.Goliath }
end

local orbit = {
    name = "Orbit", orders = "Orbit a planet, a moon and a gas giant",
    count = 3,
    spawn = function(ctx, slot)
        local target = orbitTargets(ctx)[slot]
        local at = EntityPosition.Get(target)
        return shift(at, ORBIT_START_RADII * radiusOf(target) / toGame(1))
    end,
    order = function(ctx, ship)
        return Nav.Orders.Orbit { target = orbitTargets(ctx)[ship.slot] }
    end,
}

local follow = {
    name = "Follow", orders = "Follow a leader that flies across the system, at offsets",
    count = 4, leader = 1,
    spawn = function(_, slot) return shift(FOLLOW_A, (slot - 1) * SPACING_M) end,
    order = function(ctx, ship)
        if ship.slot == 1 then return betweenPoints(ship, FOLLOW_A, FOLLOW_B) end
        local offset = FOLLOW_OFFSETS_M[ship.slot - 1]
        return Nav.Orders.Follow {
            target = ctx.leaderOf(ship).entity,
            offset = Vec3d(offset[1], offset[2], offset[3]),
        }
    end,
}

local hold = {
    name = "Hold", orders = "Hold in a planet's well, which carries the ships along",
    count = 3,
    spawn = function(ctx, slot)
        local goliath = ctx.scene.planets.Goliath
        return shift(Scene.around(EntityPosition.Get(goliath), HOLD_ANGLE_DEG, HOLD_DISTANCE_M), 0, 0,
            (slot - 1) * SPACING_M)
    end,
    order = function() return Nav.Orders.Hold {} end,
}

local traffic = {
    name = "Traffic", orders = "MoveTo back and forth: head-on and crossing streams",
    count = #TRAFFIC_PAIRS_M,
    spawn = function(_, slot)
        local a = TRAFFIC_PAIRS_M[slot][1]
        return shift(TRAFFIC_CENTRE, a[1], a[2], a[3])
    end,
    order = function(_, ship)
        local pair = TRAFFIC_PAIRS_M[ship.slot]
        return betweenPoints(ship, shift(TRAFFIC_CENTRE, pair[1][1], pair[1][2], pair[1][3]),
            shift(TRAFFIC_CENTRE, pair[2][1], pair[2][2], pair[2][3]))
    end,
}

for _, scenario in ipairs({ farCrossing, moonsAndStation, chase, chaseInField, patrol, orbit, follow, hold, traffic }) do
    table.insert(Scenarios, scenario)
end

return Scenarios
