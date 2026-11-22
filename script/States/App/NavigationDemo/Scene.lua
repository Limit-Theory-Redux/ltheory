-- The authored star system of the navigation demo: a star, four planets (two with moons), a station on
-- a planet's orbit, and a cluster of rocks, on the runtime orbit model. Every size and distance is
-- authored in metres and converted with `UniverseScaleConfig`; angles are degrees from the +x axis
-- toward +z, in the plane of the orbits.

local Registry              = require("Core.ECS.Registry")
local PhysicsComponents     = require("Modules.Physics.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local SpatialComponents     = require("Modules.Spatial.Components")
local SpatialEntities       = require("Modules.Spatial.Entities")
local CelestialEntities     = require("Modules.CelestialObjects.Entities")
local RenderComp            = require("Modules.Rendering.Components").Render
local Materials             = require("Shared.Registries.Materials")
local SolarSystemVisualizer = require("Modules.CelestialObjects.Managers.SolarSystemVisualizer")
local AsteroidMeshPool      = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local OrbitalSystem         = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local StationGenerator      = require("Modules.Constructs.Managers.Generators.StationGenerator")
local ShipGenerator         = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local ScaleConfig           = require("Config.Gen.UniverseScaleConfig")

---@class NavigationDemoScene
---@field star Entity
---@field planets table<string, Entity>
---@field moons table<string, Entity>
---@field station Entity
---@field rocks { entity: Entity, rb: RigidBody, radius: gameunit }[]
---@field orbiters table[] For `OrbitalSystem:update`
---@field followers table[] For `OrbitalSystem:update`
---@field fieldCentre Position The middle of the rock cluster
local Scene = {}

local SEED = 20261122
local UP   = Vec3f(0, 1, 0)

-- Kind names of `UniverseScaleConfig.objectScales`.
local KIND_SYSTEM, KIND_ZONE = "starSystem", "zone"

local STAR = { radiusM = 3.5e8, zoneM = 1e9 }

---@class DemoMoonSpec
---@field name string
---@field radiusM number
---@field orbitM number Distance from the planet's centre
---@field angleDeg number
---@field zoneM number

---@class DemoPlanetSpec
---@field name string
---@field radiusM number
---@field orbitM number Distance from the star
---@field angleDeg number
---@field zoneM number Radius of the planet's well
---@field moons DemoMoonSpec[]|nil
---@field stationOrbitM number|nil The station's distance from this planet's centre
---@type DemoPlanetSpec[]
local PLANETS = {
    { name = "Cinder", radiusM = 3.0e6, orbitM = 2.5e9, angleDeg = 80, zoneM = 3e7 },
    {
        name = "Aurora", radiusM = 6.4e6, orbitM = 4.5e9, angleDeg = 150, zoneM = 8.5e7,
        stationOrbitM = 1.5e7,
        moons = {
            { name = "Luna", radiusM = 1.7e6, orbitM = 3.0e7, angleDeg = 20, zoneM = 1.2e7 },
            { name = "Pebble", radiusM = 8.0e5, orbitM = 5.5e7, angleDeg = 200, zoneM = 6e6 },
        },
    },
    {
        name = "Goliath", radiusM = 2.5e7, orbitM = 7.5e9, angleDeg = 280, zoneM = 3e8,
        moons = {
            { name = "Ember", radiusM = 1.8e6, orbitM = 8.0e7, angleDeg = 60, zoneM = 1.5e7 },
            { name = "Shard", radiusM = 1.5e6, orbitM = 1.3e8, angleDeg = 240, zoneM = 1.2e7 },
        },
    },
    { name = "Frost", radiusM = 5.0e6, orbitM = 1.05e10, angleDeg = 20, zoneM = 5e7 },
}

local STATION_ANGLE_DEG = 110
local STATION_RADIUS_M  = 3000

-- Every body moves on its orbit at this fraction of the Solo top speed, so that a ship that has
-- arrived keeps up with its goal on its own.
local ORBIT_SPEED_RATIO = 0.1

-- The rock cluster: an ellipsoid in the plane of the orbits, on its own orbit radius between two planets'.
local FIELD = {
    angleDeg = 215, orbitM = 6.0e9,
    halfExtentM = { x = 12000, y = 2000, z = 12000 },
    rockCount = 50, minRadiusM = 300, maxRadiusM = 700,
    -- Rocks keep this multiple of their radii apart.
    clearance = 1.2,
    placementAttempts = 500,
}
local ROCK_MESH_POOL_SIZE, ROCK_MESH_POOL_SEED = 8, 42

---@param metres number
---@return gameunit
local function toGame(metres)
    return ScaleConfig:toGameUnits(metres, KIND_SYSTEM)
end
Scene.toGame = toGame

---A point of the orbit plane at `orbitM` from the star, `heightM` above it.
---@param angleDeg number
---@param orbitM number
---@param heightM number|nil
---@return Position
function Scene.pointAt(angleDeg, orbitM, heightM)
    local angle = math.rad(angleDeg)
    return Position(toGame(orbitM * math.cos(angle)), toGame(heightM or 0), toGame(orbitM * math.sin(angle)))
end

---@param centre Position
---@param angleDeg number
---@param distanceM number
---@return Position
local function around(centre, angleDeg, distanceM)
    local angle = math.rad(angleDeg)
    return Position(centre.x + toGame(distanceM * math.cos(angle)), centre.y,
        centre.z + toGame(distanceM * math.sin(angle)))
end

Scene.around = around

---A body of the system with its place and size in its Transform, which the materializer reads.
---@param factory fun(seed: integer): Entity
---@param seed integer
---@param radiusM number
---@param kind string Name in `UniverseScaleConfig.objectScales`
---@param position Position
---@return Entity
local function createBody(factory, seed, radiusM, kind, position)
    local entity = factory(seed)
    local transform = entity:get(PhysicsComponents.Transform)
    transform:setScale(ScaleConfig:toGameUnits(radiusM, kind))
    transform:setPos(position)
    return entity
end

---The gravity well of a body: a sphere zone on it.
---@param body Entity
---@param radiusM number
local function attachWell(body, radiusM)
    local zone = SpatialEntities.Zone()
    local shape = zone:get(SpatialComponents.Shape)
    shape:setShape(Enums.ZoneShape.Sphere)
    shape:setRadius(ScaleConfig:toGameUnits(radiusM, KIND_ZONE))
    Registry:attachEntity(body, zone)
end

---A planet with its moons, as children of `star`.
---@param star Entity
---@param spec DemoPlanetSpec
---@param seeds RNG
---@param scene NavigationDemoScene
local function buildPlanet(star, spec, seeds, scene)
    local position = Scene.pointAt(spec.angleDeg, spec.orbitM)
    local planet = createBody(CelestialEntities.Planet, seeds:get31(), spec.radiusM, "planet", position)
    planet:add(SpatialComponents.Orbit(toGame(spec.orbitM)))
    attachWell(planet, spec.zoneM)
    Registry:attachEntity(star, planet)
    scene.planets[spec.name] = planet

    for _, moonSpec in ipairs(spec.moons or {}) do
        local moon = createBody(CelestialEntities.Moon, seeds:get31(), moonSpec.radiusM, "moon",
            around(position, moonSpec.angleDeg, moonSpec.orbitM))
        moon:add(SpatialComponents.Orbit(toGame(moonSpec.orbitM)))
        attachWell(moon, moonSpec.zoneM)
        Registry:attachEntity(planet, moon)
        scene.moons[moonSpec.name] = moon
    end

    if spec.stationOrbitM then
        local station = StationGenerator:createStation(seeds:get31(), {
            position = around(position, STATION_ANGLE_DEG, spec.stationOrbitM),
            scale = toGame(STATION_RADIUS_M),
            isKinematic = true,
        })
        station:add(SpatialComponents.Orbit(toGame(spec.stationOrbitM)))
        Registry:attachEntity(planet, station)
        scene.station = station
    end
end

---Puts every orbiting body's rigid body into `world`, which the materializer did for the bodies it
---renders; the station's body is its own.
---@param world Physics
---@param scene NavigationDemoScene
local function addStationBody(world, scene)
    world:addRigidBody(scene.station:get(PhysicsComponents.RigidBody):getRigidBody())
end

---A rock: an `Asteroid` entity with a mesh from the pool and a kinematic sphere body in `world`.
---@param world Physics
---@param seed integer
---@param position Position
---@param radius gameunit
---@return { entity: Entity, rb: RigidBody, radius: gameunit }
local function createRock(world, seed, position, radius)
    local entity = CelestialEntities.Asteroid(seed)
    local transform = entity:get(PhysicsComponents.Transform)
    transform:setPos(position)
    transform:setScale(radius)
    -- The constructor's render component holds no mesh; it is removed first, as a component added
    -- over another would leave the old one in the registry's dense storage.
    entity:remove(RenderComp)
    local mesh = AsteroidMeshPool:getFromSeed(seed):get(0)
    entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))

    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    world:addRigidBody(rb)
    return { entity = entity, rb = rb, radius = radius }
end

---A point of the unit ball, uniformly.
---@param rng RNG
---@return number x, number y, number z
local function inUnitBall(rng)
    while true do
        local x, y, z = rng:getUniformRange(-1, 1), rng:getUniformRange(-1, 1), rng:getUniformRange(-1, 1)
        if x * x + y * y + z * z <= 1 then return x, y, z end
    end
end

---The rock cluster, from a fixed seed: rocks spread through an ellipsoid and not touching.
---@param world Physics
---@param scene NavigationDemoScene
local function buildField(world, scene)
    local rng = RNG.Create(SEED)
    local centre = Scene.pointAt(FIELD.angleDeg, FIELD.orbitM)
    local half = FIELD.halfExtentM
    scene.fieldCentre = centre
    scene.rocks = {}
    for _ = 1, FIELD.rockCount do
        local radius = toGame(rng:getUniformRange(FIELD.minRadiusM, FIELD.maxRadiusM))
        local position
        for _ = 1, FIELD.placementAttempts do
            local x, y, z = inUnitBall(rng)
            local candidate = Position(centre.x + toGame(x * half.x), centre.y + toGame(y * half.y),
                centre.z + toGame(z * half.z))
            local clear = true
            for _, rock in ipairs(scene.rocks) do
                if candidate:distance(rock.rb:getPos()) < (radius + rock.radius) * FIELD.clearance then
                    clear = false
                    break
                end
            end
            if clear then
                position = candidate
                break
            end
        end
        assert(position, "NavigationDemo: no room for a rock in the cluster")
        table.insert(scene.rocks, createRock(world, rng:get31(), position, radius))
    end
end

---A point of the rock cluster that keeps `marginM` metres from every rock's surface.
---@param scene NavigationDemoScene
---@param rng RNG
---@param marginM number
---@return Position
function Scene.freePointInField(scene, rng, marginM)
    local half, centre = FIELD.halfExtentM, scene.fieldCentre
    local margin = toGame(marginM)
    for _ = 1, FIELD.placementAttempts do
        local x, y, z = inUnitBall(rng)
        local candidate = Position(centre.x + toGame(x * half.x), centre.y + toGame(y * half.y),
            centre.z + toGame(z * half.z))
        local clear = true
        for _, rock in ipairs(scene.rocks) do
            if candidate:distance(rock.rb:getPos()) < rock.radius + margin then
                clear = false
                break
            end
        end
        if clear then return candidate end
    end
    error("NavigationDemo: no free point in the rock cluster")
end

---A point `outsideFactor` times the cluster's extent from its middle, in the orbit plane, at `angleDeg`.
---@param scene NavigationDemoScene
---@param angleDeg number
---@param outsideFactor number
---@return Position
function Scene.aroundField(scene, angleDeg, outsideFactor)
    local angle = math.rad(angleDeg)
    local half, centre = FIELD.halfExtentM, scene.fieldCentre
    return Position(centre.x + toGame(outsideFactor * half.x * math.cos(angle)), centre.y,
        centre.z + toGame(outsideFactor * half.z * math.sin(angle)))
end

---Gives every orbit the speed `ORBIT_SPEED_RATIO` of the Solo top speed, as an angular speed.
---@param orbiters table[]
local function setOrbitSpeeds(orbiters)
    local topSpeed = FlightProfileRegistry:get(Enums.FlightProfile.Solo).maxSpeed
    for _, orbiter in ipairs(orbiters) do
        orbiter.orbit:setAngularSpeed(ORBIT_SPEED_RATIO * topSpeed / orbiter.orbit:getOrbitRadius())
    end
end

---Builds the system, with its rigid bodies in `world`, and starts its orbits and wells.
---@param world Physics
---@return NavigationDemoScene
function Scene.build(world)
    AsteroidMeshPool:init(ROCK_MESH_POOL_SIZE, ROCK_MESH_POOL_SEED)
    local seeds = RNG.Create(SEED)
    local scene = { planets = {}, moons = {} }

    local star = createBody(CelestialEntities.Star, seeds:get31(), STAR.radiusM, "star", Position(0, 0, 0))
    attachWell(star, STAR.zoneM)
    scene.star = star
    for _, spec in ipairs(PLANETS) do buildPlanet(star, spec, seeds, scene) end

    SolarSystemVisualizer:materializeEntity(star, world)
    addStationBody(world, scene)
    buildField(world, scene)

    scene.orbiters, scene.followers = OrbitalSystem:collectOrbiters(star)
    setOrbitSpeeds(scene.orbiters)
    GravityWellSystem:collectZones()
    return scene
end

---A Solo ship set up as the player's is, at `position` facing `lookAt`, with its body in `world`.
---@param world Physics
---@param seed integer
---@param position Position
---@param lookAt Position
---@return Entity
function Scene.spawnShip(world, seed, position, lookAt)
    local hull = Enums.ShipHulls.Solo
    local facing = Vec3f(lookAt.x - position.x, lookAt.y - position.y, lookAt.z - position.z):normalize()
    local entity = ShipGenerator:createFighter(seed, {
        hull = hull,
        position = position,
        rotation = Quat.FromLook(facing, UP),
        scale = Config.game.shipHulls.scale[hull],
        isKinematic = false,
        flightProfile = Enums.FlightProfile.Solo,
    })
    entity:add(ConstructComponents.TravelDrive())
    entity:add(ConstructComponents.ControlInput())
    entity:add(PhysicsComponents.GravityWell(ConstructComponents.FlightProfile.ProfileOf(entity).linearDrag))
    world:addRigidBody(entity:get(PhysicsComponents.RigidBody):getRigidBody())
    return entity
end

return Scene
