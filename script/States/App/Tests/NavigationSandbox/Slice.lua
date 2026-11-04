-- 100 ships loop through their own four waypoints among 60 asteroids and a planet that circles
-- the middle of the field. Waypoints are clear of every asteroid and of the planet's whole path.
-- The seed fixes the field, the routes and the starts.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

local SEED               = 7
local SHIP_COUNT         = 100
local WAYPOINT_COUNT     = 4
local ASTEROID_COUNT     = 60
local ASTEROID_MIN_M     = 100
local ASTEROID_MAX_M     = 800
-- The field is a slab: x and z within +-HALF_EXTENT_M of the middle, y within +-HALF_HEIGHT_M.
local HALF_EXTENT_M      = 10000
local HALF_HEIGHT_M      = 1000
local PLANET_RADIUS_M    = 3000
local PLANET_ORBIT_M     = 6000
local PLANET_PERIOD_S    = 60
-- Gap kept between asteroids, and from an asteroid or the planet's path to a waypoint or start.
local ASTEROID_GAP_M     = 500
local WAYPOINT_CLEARANCE_M = 1000
local START_SPACING_M    = 300
-- Consecutive waypoints are at least this far apart: shorter legs end inside the arrival range.
local MIN_LEG_M          = 1000
local TIME_LIMIT_S       = 60
-- Tries for one placement before the generator gives up.
local MAX_ATTEMPTS       = 10000

local rng = RNG.Create(SEED)

---@return number[] A point in the slab, in metres
local function randomPoint()
    return {
        rng:getUniformRange(-HALF_EXTENT_M, HALF_EXTENT_M),
        rng:getUniformRange(-HALF_HEIGHT_M, HALF_HEIGHT_M),
        rng:getUniformRange(-HALF_EXTENT_M, HALF_EXTENT_M),
    }
end

local function distance(a, b)
    local dx, dy, dz = a[1] - b[1], a[2] - b[2], a[3] - b[3]
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

---Distance from `p` to the circle the planet's centre follows.
local function distanceToPlanetPath(p)
    local ring = math.sqrt(p[1] * p[1] + p[3] * p[3]) - PLANET_ORBIT_M
    return math.sqrt(ring * ring + p[2] * p[2])
end

---A random point in the slab that `accept(point)` takes; fails loudly when there is no room.
local function place(what, accept)
    for _ = 1, MAX_ATTEMPTS do
        local point = randomPoint()
        if accept(point) then return point end
    end
    error(string.format("Slice: no room for %s after %d tries", what, MAX_ATTEMPTS))
end

local asteroids = {}
for _ = 1, ASTEROID_COUNT do
    local radius = rng:getUniformRange(ASTEROID_MIN_M, ASTEROID_MAX_M)
    local position = place("an asteroid", function(p)
        if distanceToPlanetPath(p) <= PLANET_RADIUS_M + radius + ASTEROID_GAP_M then return false end
        for _, other in ipairs(asteroids) do
            if distance(p, other.position) <= radius + other.radius + ASTEROID_GAP_M then return false end
        end
        return true
    end)
    table.insert(asteroids, { position = position, radius = radius })
end

---Whether a ship may have a waypoint or start at `p`.
local function clear(p)
    if distanceToPlanetPath(p) <= PLANET_RADIUS_M + WAYPOINT_CLEARANCE_M then return false end
    for _, asteroid in ipairs(asteroids) do
        if distance(p, asteroid.position) <= asteroid.radius + WAYPOINT_CLEARANCE_M then return false end
    end
    return true
end

local ships, starts = {}, {}
for _ = 1, SHIP_COUNT do
    local route = {}
    for i = 1, WAYPOINT_COUNT do
        local previous = route[i - 1]
        route[i] = place("a waypoint", function(p)
            if not clear(p) then return false end
            -- The route is a loop: the last waypoint is also the ship's start, so it keeps clear
            -- of the other starts, and the leg back to the first one is long enough.
            if previous and distance(p, previous) < MIN_LEG_M then return false end
            if i < WAYPOINT_COUNT then return true end
            if distance(p, route[1]) < MIN_LEG_M then return false end
            for _, start in ipairs(starts) do
                if distance(p, start) < START_SPACING_M then return false end
            end
            return true
        end)
    end
    local start = route[WAYPOINT_COUNT]
    table.insert(starts, start)
    table.insert(ships, { start = start, route = route })
end

local obstacles = {
    {
        radius = PLANET_RADIUS_M,
        circle = { centre = { 0, 0, 0 }, radius = PLANET_ORBIT_M, periodS = PLANET_PERIOD_S },
    },
}
for _, asteroid in ipairs(asteroids) do
    table.insert(obstacles, { position = asteroid.position, radius = asteroid.radius })
end

return NavigationSandbox.ScenarioApp {
    name = "Slice",
    ships = ships,
    obstacles = obstacles,
    timeLimit = TIME_LIMIT_S,
    avoidance = { horizonS = 10, clearanceRadii = 2, aimFactor = 1.4 },
}
