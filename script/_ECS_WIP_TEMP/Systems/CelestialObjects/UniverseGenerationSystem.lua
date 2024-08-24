-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Entities
local UniverseEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.UniverseEntity")
local StarSystemEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.StarSystemEntity")
local StarEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.StarEntity")
local AsteroidBeltEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.AsteroidBeltEntity")
local AsteroidRingEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.AsteroidRingEntity")
local PlanetEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.PlanetEntity")
local MoonEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.MoonEntity")
local AsteroidEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.AsteroidEntity")
local SpaceStationEntity = require("_ECS_WIP_TEMP.Entities.Constructs.SpaceStationEntity")
local SpaceshipEntity = require("_ECS_WIP_TEMP.Entities.Constructs.SpaceshipEntity")

local ZoneEntity = require("_ECS_WIP_TEMP.Entities.Spatial.ZoneEntity")

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler")

---@class UniverseGenerationSystem
---@overload fun(self: UniverseGenerationSystem): UniverseGenerationSystem class internal
---@overload fun(): UniverseGenerationSystem class external
local UniverseGenerationSystem = Class(function(self)
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
    local universeRNG = RNG.Create(seed):managed()
    local universeEntityInfo = GlobalStorage:storeEntity(universe)

    -- Generate star systems
    local numStarSystems = universeRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numStarSystems do
        local starSystemSeed = universeRNG:get64()
        local starSystem = StarSystemEntity(starSystemSeed)
        local starSystemRNG = RNG.Create(starSystemSeed):managed()
        local starSystemEntityInfo = GlobalStorage:storeEntity(starSystem)

        -- Generate star and celestial bodies
        self:generateStarAndCelestialBodies(starSystem, starSystemRNG)

        -- Generate constructs
        -- * currently spawned as a child of star system
        -- * idea could be that you´d have some kind of gravity well zone which adds / removes them while traveling
        -- todo: spawn in a seperate system via economy
        self:generateConstructs(starSystem, starSystemRNG)

        -- Add star system to universe
        self:addChildEntity(universe, starSystemEntityInfo)
    end

    self.profiler:stop()
    return universeEntityInfo
end

---@param starSystem StarSystemEntity
---@param rng RandomNumberGenerator
function UniverseGenerationSystem:generateStarAndCelestialBodies(starSystem, rng)
    -- Generate star
    local starSeed = rng:get64()
    local star = StarEntity(starSeed)
    local starRNG = RNG.Create(starSeed):managed()
    local starEntityInfo = GlobalStorage:storeEntity(star)

    -- Add star area
    local starZone = ZoneEntity()
    ---@type SpatialShapeComponent
    local starZoneShapeComponent = starZone:findComponentByArchetype(Enums.ComponentArchetype.ShapeComponent)
    starZoneShapeComponent:setShape(Enums.ZoneShape.Sphere)
    starZoneShapeComponent:setRadius(1.7952e13) --* Hardcode to solar system radius for now

    self:addChildEntity(starSystem, starEntityInfo)

    -- Generate planets
    local numPlanets = starRNG:getInt(1, 3) --* Replace with config later
    for i = 1, numPlanets do
        local planetSeed = rng:get64()
        local planet = PlanetEntity(planetSeed)
        local planetSystemRNG = RNG.Create(planetSeed):managed()
        local planetEntityInfo = GlobalStorage:storeEntity(planet)
        self:addChildEntity(star, planetEntityInfo)

        -- Generate planetary features
        self:generatePlanetaryFeatures(planet, planetSystemRNG)
    end

    -- Generate asteroid belts
    local numAsteroidBelts = starRNG:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidBelts do
        local beltSeed = starRNG:get64()
        local asteroidBelt = AsteroidBeltEntity(beltSeed)
        local asteroidBeltEntityInfo = GlobalStorage:storeEntity(asteroidBelt)
        local asteroidBeltRNG = RNG.Create(starSeed):managed()
        self:addChildEntity(star, asteroidBeltEntityInfo)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidBeltRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidBeltRNG:get64()
            local asteroid = AsteroidEntity(asteroidSeed)
            local asteroidEntityInfo = GlobalStorage:storeEntity(asteroid)
            self:addChildEntity(asteroidBelt, asteroidEntityInfo)
        end
    end
end

---@param planet PlanetEntity
---@param rng RandomNumberGenerator
function UniverseGenerationSystem:generatePlanetaryFeatures(planet, rng)
    -- Generate moons
    local numMoons = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numMoons do
        local moonSeed = rng:get64()
        local moon = MoonEntity(moonSeed)
        local moonEntityInfo = GlobalStorage:storeEntity(moon)
        self:addChildEntity(planet, moonEntityInfo)
    end

    -- Generate asteroid rings
    local numAsteroidRings = rng:getInt(0, 1) --* Replace with config later
    for i = 1, numAsteroidRings do
        local ringSeed = rng:get64()
        local asteroidRing = AsteroidRingEntity(ringSeed)
        local asteroidRingRNG = RNG.Create(ringSeed):managed()
        local asteroidRingEntityInfo = GlobalStorage:storeEntity(asteroidRing)
        self:addChildEntity(planet, asteroidRingEntityInfo)

        -- Add individual asteroids to the belt
        local numAsteroids = asteroidRingRNG:getInt(5, 10) --* Replace with config later
        for j = 1, numAsteroids do
            local asteroidSeed = asteroidRingRNG:get64()
            local asteroid = AsteroidEntity(asteroidSeed)
            local asteroidEntityInfo = GlobalStorage:storeEntity(asteroid)
            self:addChildEntity(asteroidRing, asteroidEntityInfo)
        end
    end
end

---@param starSystem StarSystemEntity
---@param rng RandomNumberGenerator
function UniverseGenerationSystem:generateConstructs(starSystem, rng)
    -- Generate space stations
    local numSpaceStations = rng:getInt(0, 3) --* Replace with config later
    for i = 1, numSpaceStations do
        local spaceStationSeed = rng:get64()
        local spaceStation = SpaceStationEntity(spaceStationSeed)
        local spaceStationEntityInfo = GlobalStorage:storeEntity(spaceStation)
        self:addChildEntity(starSystem, spaceStationEntityInfo)
    end

    -- Generate spaceships
    local numSpaceships = rng:getInt(0, 5) --* Replace with config later
    for i = 1, numSpaceships do
        local spaceshipSeed = rng:get64()
        local spaceship = SpaceshipEntity(spaceshipSeed)
        local spaceshipEntityInfo = GlobalStorage:storeEntity(spaceship)
        self:addChildEntity(starSystem, spaceshipEntityInfo)
    end
end

function UniverseGenerationSystem:addChildEntity(parentEntity, childEntityInfo)
    local hierarchyComponent = parentEntity:findComponentByArchetype(Enums.ComponentArchetype.HierarchyComponent)
    hierarchyComponent:addChild(childEntityInfo)
end

return UniverseGenerationSystem()
