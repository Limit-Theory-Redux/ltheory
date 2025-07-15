-- Systems
local Registry = require("Core.ECS.Registry")

-- Entities
local UniverseEntity = require("Entities.CelestialObjects.UniverseEntity")
local StarSystemEntity = require("Entities.CelestialObjects.StarSystemEntity")
local StarEntity = require("Entities.CelestialObjects.StarEntity")
local AsteroidBeltEntity = require("Entities.CelestialObjects.AsteroidBeltEntity")
local AsteroidRingEntity = require("Entities.CelestialObjects.AsteroidRingEntity")
local PlanetEntity = require("Entities.CelestialObjects.PlanetEntity")
local MoonEntity = require("Entities.CelestialObjects.MoonEntity")
local AsteroidEntity = require("Entities.CelestialObjects.AsteroidEntity")
local SpaceStationEntity = require("Entities.Constructs.SpaceStationEntity")
local SpaceshipEntity = require("Entities.Constructs.SpaceshipEntity")

local ZoneEntity = require("Entities.Spatial.ZoneEntity")

-- Components
local ShapeComponent = require("Components.Spatial.ShapeComponent")
local HierarchyComponent = require("Components.Core.HierarchyComponent")

-- Utilities
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
    self.profiler:start()
    -- Construct universe entity
    local universe = UniverseEntity(seed)
    local universeRNG = RNG.Create(seed)
    local universeEntityId = Registry:storeEntity(universe)

    -- Generate star systems
    local numStarSystems = universeRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numStarSystems do
        local starSystemSeed = universeRNG:get64()
        local starSystem = StarSystemEntity(starSystemSeed)
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
---@param rng RandomNumberGenerator
function UniverseGenerationSystem:generateStarAndCelestialBodies(starSystem, rng)
    -- Generate star
    local starSeed = rng:get64()
    local star = StarEntity(starSeed)
    local starRNG = RNG.Create(starSeed)
    local starEntityId = Registry:storeEntity(star)

    -- Add star area
    local starZone = ZoneEntity()
    ---@type SpatialShapeComponent
    local starZoneShapeComponent = starZone:getComponent(ShapeComponent)
    starZoneShapeComponent:setShape(Enums.ZoneShape.Sphere)
    starZoneShapeComponent:setRadius(1.7952e13) --* Hardcode to solar system radius for now

    self:addChildEntity(starSystem, starEntityId)

    -- Generate planets
    local numPlanets = starRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numPlanets do
        local planetSeed = rng:get64()
        local planet = PlanetEntity(planetSeed)
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
        local asteroidBelt = AsteroidBeltEntity(beltSeed)
        local asteroidBeltEntityId = Registry:storeEntity(asteroidBelt)
        local asteroidBeltRNG = RNG.Create(starSeed)
        self:addChildEntity(star, asteroidBeltEntityId)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidBeltRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidBeltRNG:get64()
            local asteroid = AsteroidEntity(asteroidSeed)
            local asteroidEntityId = Registry:storeEntity(asteroid)
            self:addChildEntity(asteroidBelt, asteroidEntityId)
        end
    end
end

---@private
---@param planet PlanetEntity
---@param rng RandomNumberGenerator
function UniverseGenerationSystem:generatePlanetaryFeatures(planet, rng)
    -- Generate moons
    local numMoons = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numMoons do
        local moonSeed = rng:get64()
        local moon = MoonEntity(moonSeed)
        local moonEntityId = Registry:storeEntity(moon)
        self:addChildEntity(planet, moonEntityId)
    end

    -- Generate asteroid rings
    local numAsteroidRings = rng:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidRings do
        local ringSeed = rng:get64()
        local asteroidRing = AsteroidRingEntity(ringSeed)
        local asteroidRingRNG = RNG.Create(ringSeed)
        local asteroidRingEntityId = Registry:storeEntity(asteroidRing)
        self:addChildEntity(planet, asteroidRingEntityId)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidRingRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidRingRNG:get64()
            local asteroid = AsteroidEntity(asteroidSeed)
            local asteroidEntityId = Registry:storeEntity(asteroid)
            self:addChildEntity(asteroidRing, asteroidEntityId)
        end
    end
end

---@private
---@param starSystem StarSystemEntity
---@param rng RandomNumberGenerator
function UniverseGenerationSystem:generateConstructs(starSystem, rng)
    -- Generate space stations
    local numSpaceStations = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numSpaceStations do
        local spaceStationSeed = rng:get64()
        local spaceStation = SpaceStationEntity(spaceStationSeed)
        local spaceStationEntityId = Registry:storeEntity(spaceStation)
        self:addChildEntity(starSystem, spaceStationEntityId)
    end

    -- Generate spaceships
    local numSpaceships = rng:getInt(0, 5) --* Replace with config later
    for i = 1, numSpaceships do
        local spaceshipSeed = rng:get64()
        local spaceship = SpaceshipEntity(spaceshipSeed)
        local spaceshipEntityId = Registry:storeEntity(spaceship)
        self:addChildEntity(starSystem, spaceshipEntityId)
    end
end

---@private
function UniverseGenerationSystem:addChildEntity(parentEntity, childEntityId)
    local hierarchyComponent = parentEntity:getComponent(HierarchyComponent)
    hierarchyComponent:addChild(childEntityId)
end

return UniverseGenerationSystem()
