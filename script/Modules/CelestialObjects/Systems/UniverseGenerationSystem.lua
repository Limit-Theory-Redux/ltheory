local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")

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
    local CelestialObjects = require("Modules.CelestialObjects")

    self.profiler:start()
    -- Construct universe entity
    local universe = CelestialObjects.Entities.Universe(seed)
    local universeRNG = RNG.Create(seed)
    local universeEntityId = Registry:storeEntity(universe)

    -- Generate star systems
    local numStarSystems = universeRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numStarSystems do
        local starSystemSeed = universeRNG:get64()
        local starSystem = CelestialObjects.Entities.StarSystem(starSystemSeed)
        local starSystemRNG = RNG.Create(starSystemSeed)
        local starSystemEntityId = Registry:storeEntity(starSystem)

        -- Generate star and celestial bodies
        self:generateStarAndCelestialBodies(starSystem, starSystemRNG)

        -- Generate constructs
        -- * currently spawned as a child of star system
        -- * idea could be that you´d have some kind of gravity well zone which adds / removes them while traveling
        -- todo: spawn in a seperate system via economy
        self:generateConstructs(starSystem, starSystemRNG)

        -- Add star system to universe
        self:addChildEntity(universe, starSystemEntityId)
    end

    self.profiler:stop()
    return universeEntityId
end

---@private
---@param starSystem StarSystemEntity
---@param rng RNG
function UniverseGenerationSystem:generateStarAndCelestialBodies(starSystem, rng)
    local CelestialObjects = require("Modules.CelestialObjects")
    local Spatial = require("Modules.Spatial")

    -- Generate star
    local starSeed = rng:get64()
    local star = CelestialObjects.Entities.Star(starSeed)
    local starRNG = RNG.Create(starSeed)
    local starEntityId = Registry:storeEntity(star)

    -- Add star area
    local starZone = Spatial.Entities.Zone()
    ---@type SpatialShapeComponent
    local starZoneShapeComponent = starZone:getComponent(Spatial.Components.Shape)
    starZoneShapeComponent:setShape(Enums.ZoneShape.Sphere)
    starZoneShapeComponent:setRadius(1.7952e13) --* Hardcode to solar system radius for now

    self:addChildEntity(starSystem, starEntityId)

    -- Generate planets
    local numPlanets = starRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numPlanets do
        local planetSeed = rng:get64()
        local planet = CelestialObjects.Entities.Planet(planetSeed)
        local planetSystemRNG = RNG.Create(planetSeed)
        local planetEntityId = Registry:storeEntity(planet)
        self:addChildEntity(star, planetEntityId)

        -- Generate planetary features
        self:generatePlanetaryFeatures(planet, planetSystemRNG)
    end

    -- Generate asteroid belts
    local numAsteroidBelts = starRNG:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidBelts do
        local beltSeed = starRNG:get64()
        local asteroidBelt = CelestialObjects.Entities.AsteroidBelt(beltSeed)
        local asteroidBeltEntityId = Registry:storeEntity(asteroidBelt)
        local asteroidBeltRNG = RNG.Create(starSeed)
        self:addChildEntity(star, asteroidBeltEntityId)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidBeltRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidBeltRNG:get64()
            local asteroid = CelestialObjects.Entities.Asteroid(asteroidSeed)
            local asteroidEntityId = Registry:storeEntity(asteroid)
            self:addChildEntity(asteroidBelt, asteroidEntityId)
        end
    end
end

---@private
---@param planet PlanetEntity
---@param rng RNG
function UniverseGenerationSystem:generatePlanetaryFeatures(planet, rng)
    local CelestialObjects = require("Modules.CelestialObjects")

    -- Generate moons
    local numMoons = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numMoons do
        local moonSeed = rng:get64()
        local moon = CelestialObjects.Entities.Moon(moonSeed)
        local moonEntityId = Registry:storeEntity(moon)
        self:addChildEntity(planet, moonEntityId)
    end

    -- Generate asteroid rings
    local numAsteroidRings = rng:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidRings do
        local ringSeed = rng:get64()
        local asteroidRing = CelestialObjects.Entities.AsteroidRing(ringSeed)
        local asteroidRingRNG = RNG.Create(ringSeed)
        local asteroidRingEntityId = Registry:storeEntity(asteroidRing)
        self:addChildEntity(planet, asteroidRingEntityId)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidRingRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidRingRNG:get64()
            local asteroid = CelestialObjects.Entities.Asteroid(asteroidSeed)
            local asteroidEntityId = Registry:storeEntity(asteroid)
            self:addChildEntity(asteroidRing, asteroidEntityId)
        end
    end
end

---@private
---@param starSystem StarSystemEntity
---@param rng RNG
function UniverseGenerationSystem:generateConstructs(starSystem, rng)
    local Constructs = require("Modules.Constructs")

    -- Generate space stations
    local numSpaceStations = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numSpaceStations do
        local spaceStationSeed = rng:get64()
        local spaceStation = Constructs.Entities.SpaceStation(spaceStationSeed)
        local spaceStationEntityId = Registry:storeEntity(spaceStation)
        self:addChildEntity(starSystem, spaceStationEntityId)
    end

    -- Generate spaceships
    local numSpaceships = rng:getInt(0, 5) --* Replace with config later
    for i = 1, numSpaceships do
        local spaceshipSeed = rng:get64()
        local spaceship = Constructs.Entities.Spaceship(spaceshipSeed)
        local spaceshipEntityId = Registry:storeEntity(spaceship)
        self:addChildEntity(starSystem, spaceshipEntityId)
    end
end

---@private
function UniverseGenerationSystem:addChildEntity(parentEntity, childEntityId)
    local Core = require("Modules.Core")
    local hierarchyComponent = parentEntity:getComponent(Core.Components.Hierarchy)
    hierarchyComponent:addChild(childEntityId)
end

return UniverseGenerationSystem()
