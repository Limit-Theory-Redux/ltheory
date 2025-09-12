local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local CelestialObjects = require("Modules.CelestialObjects.Entities")
local SpatialEntities = require("Modules.Spatial.Entities")
local SpatialComponents = require("Modules.Spatial.Components")
local Constructs = require("Modules.Constructs.Entities")
local Core = require("Modules.Core.Components")

---@class UniverseGenerationSystem
---@overload fun(self: UniverseGenerationSystem): UniverseGenerationSystem class internal
---@overload fun(): UniverseGenerationSystem class external
local UniverseGenerationSystem = Class("UniverseGenerationSystem", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
end)

---@private
function UniverseGenerationSystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("UniverseGenerationSystem", false, false)
end

---@param seed integer
function UniverseGenerationSystem:createUniverse(seed)
    self.profiler:start()
    -- Construct universe entity
    local universe = CelestialObjects.Universe(seed)
    local universeRNG = RNG.Create(seed)

    -- Generate star systems
    local numStarSystems = universeRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numStarSystems do
        local starSystemSeed = universeRNG:get64()
        local starSystem = CelestialObjects.StarSystem(starSystemSeed)
        local starSystemRNG = RNG.Create(starSystemSeed)

        -- Generate star and celestial bodies
        self:generateStarAndCelestialBodies(starSystem, starSystemRNG)

        -- Generate constructs
        -- * currently spawned as a child of star system
        -- * idea could be that you´d have some kind of gravity well zone which adds / removes them while traveling
        -- todo: spawn in a seperate system via economy
        self:generateConstructs(starSystem, starSystemRNG)

        -- Add star system to universe
        Registry:attachEntity(universe, starSystem)
    end

    self.profiler:stop()
    return universe
end

---@private
---@param starSystem Entity
---@param rng RNG
function UniverseGenerationSystem:generateStarAndCelestialBodies(starSystem, rng)
    -- Generate star
    local starSeed = rng:get64()
    local star = CelestialObjects.Star(starSeed)
    local starRNG = RNG.Create(starSeed)

    -- Add star area
    local starZone = SpatialEntities.Zone()
    local starZoneShapeComponent = starZone:get(SpatialComponents.Shape)
    starZoneShapeComponent:setShape(Enums.ZoneShape.Sphere)
    starZoneShapeComponent:setRadius(1.7952e13) --* Hardcode to solar system radius for now

    Registry:attachEntity(starSystem, star)

    -- Generate planets
    local numPlanets = starRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numPlanets do
        local planetSeed = rng:get64()
        local planet = CelestialObjects.Planet(planetSeed)
        local planetSystemRNG = RNG.Create(planetSeed)
        Registry:attachEntity(star, planet)

        -- Generate planetary features
        self:generatePlanetaryFeatures(planet, planetSystemRNG)
    end

    -- Generate asteroid belts
    local numAsteroidBelts = starRNG:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidBelts do
        local beltSeed = starRNG:get64()
        local asteroidBelt = CelestialObjects.AsteroidBelt(beltSeed)
        local asteroidBeltRNG = RNG.Create(beltSeed)
        Registry:attachEntity(star, asteroidBelt)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidBeltRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidBeltRNG:get64()
            local asteroid = CelestialObjects.Asteroid(asteroidSeed)
            Registry:attachEntity(asteroidBelt, asteroid)
        end
    end
end

---@private
---@param planet Entity
---@param rng RNG
function UniverseGenerationSystem:generatePlanetaryFeatures(planet, rng)
    -- Generate moons
    local numMoons = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numMoons do
        local moonSeed = rng:get64()
        local moon = CelestialObjects.Moon(moonSeed)
        Registry:attachEntity(planet, moon)
    end

    -- Generate asteroid rings
    local numAsteroidRings = rng:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidRings do
        local ringSeed = rng:get64()
        local asteroidRing = CelestialObjects.AsteroidRing(ringSeed)
        local asteroidRingRNG = RNG.Create(ringSeed)
        Registry:attachEntity(planet, asteroidRing)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidRingRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidRingRNG:get64()
            local asteroid = CelestialObjects.Asteroid(asteroidSeed)
            Registry:attachEntity(asteroidRing, asteroid)
        end
    end
end

---@private
---@param starSystem Entity
---@param rng RNG
function UniverseGenerationSystem:generateConstructs(starSystem, rng)
    -- Generate space stations
    local numSpaceStations = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numSpaceStations do
        local spaceStationSeed = rng:get64()
        local spaceStation = CelestialObjects.SpaceStation(spaceStationSeed)
        Registry:attachEntity(starSystem, spaceStation)
    end

    -- Generate spaceships
    local numSpaceships = rng:getInt(0, 5) --* Replace with config later
    for i = 1, numSpaceships do
        local spaceshipSeed = rng:get64()
        local spaceship = CelestialObjects.Spaceship(spaceshipSeed)
        Registry:attachEntity(starSystem, spaceship)
    end
end

return UniverseGenerationSystem()
