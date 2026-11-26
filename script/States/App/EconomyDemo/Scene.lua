-- The economy demo's star system: the navigation demo's system, with a station of each economic
-- kind around the planet Aurora and its moons, and a field of iron ore rocks between them. Sizes and
-- distances are authored in metres. The bodies don't orbit: the stations stay where they are put.

local Registry          = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")
local CelestialEntities = require("Modules.CelestialObjects.Entities")
local RenderComp        = require("Modules.Rendering.Components").Render
local Materials         = require("Shared.Registries.Materials")
local AsteroidMeshPool  = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local StationGenerator  = require("Modules.Constructs.Managers.Generators.StationGenerator")
local EntityPosition    = require("Shared.Helpers.EntityPosition")
local Items             = require("Shared.Registries.Items")
local StationEconomies  = require("Shared.Definitions.StationEconomyDefs")
local EconomySetup      = require("Modules.Economy.Managers.EconomySetup")
local NavScene          = require("States.App.NavigationDemo.Scene")

---@class EconomyDemoStation
---@field entity Entity
---@field definition StationEconomyDefinition
---@field name string

---@class EconomyDemoScene: NavigationDemoScene
---@field stations EconomyDemoStation[]
---@field oreRocks Entity[]
---@field oreCentre Position

local Scene = {}

local SEED = 20261126
local STATION_RADIUS_M = 3000

-- Where each station is: around which body (a planet or a moon of the navigation scene), at which
-- angle in degrees and distance in metres from its centre. The navigation scene's own station, on
-- Aurora's orbit, is the colony.
local STATIONS = {
    { economy = "PowerPlant", name = "Luna Power Plant", body = "Luna", angleDeg = 90, distanceM = 5.0e6 },
    { economy = "Refinery", name = "Aurora Refinery", body = "Aurora", angleDeg = 250, distanceM = 2.2e7 },
    { economy = "Foundry", name = "Pebble Foundry", body = "Pebble", angleDeg = 300, distanceM = 3.0e6 },
}
local COLONY = { economy = "Colony", name = "Aurora Colony" }

-- The ore field: rocks spread in a box around a point by Aurora, not touching.
local ORE = {
    body = "Aurora", angleDeg = 235, distanceM = 2.6e7,
    halfExtentM = 8000, count = 8, minRadiusM = 250, maxRadiusM = 500, gapM = 800,
    quantity = 400, regrowPerSecond = 0.5,
}

---@param scene NavigationDemoScene
---@param name string
---@return Entity
local function bodyNamed(scene, name)
    return assert(scene.planets[name] or scene.moons[name], "EconomyDemo: no body named " .. name)
end

---@param world Physics
---@param seed integer
---@param position Position
---@param parent Entity
---@return Entity
local function createStation(world, seed, position, parent)
    local toGame = NavScene.toGame
    local station = StationGenerator:createStation(seed, {
        position = position,
        scale = toGame(STATION_RADIUS_M),
        isKinematic = true,
    })
    -- The generator scales a mesh of its own size, so the bounding radius is rescaled to the authored one.
    local body = station:get(PhysicsComponents.RigidBody):getRigidBody()
    body:setScale(body:getScale() * toGame(STATION_RADIUS_M) / body:getBoundingRadius())
    Registry:attachEntity(parent, station)
    world:addRigidBody(body)
    return station
end

---An ore rock: an `Asteroid` entity that yields iron ore, with a mesh from the pool and a kinematic
---sphere body in `world`.
---@param world Physics
---@param seed integer
---@param position Position
---@param radius gameunit
---@return Entity
local function createOreRock(world, seed, position, radius)
    local entity = CelestialEntities.Asteroid(seed)
    local transform = entity:get(PhysicsComponents.Transform)
    transform:setPos(position)
    transform:setScale(radius)
    -- The constructor's render component holds no mesh; it is removed before the one with a mesh is added.
    entity:remove(RenderComp)
    local mesh = AsteroidMeshPool:getFromSeed(seed):get(0)
    entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))

    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    world:addRigidBody(rb)
    EconomySetup.equipAsteroid(entity, Items.RawMaterials.IronOre, ORE.quantity, ORE.regrowPerSecond)
    return entity
end

---@param world Physics
---@param scene EconomyDemoScene
---@param rng RNG
local function buildOreField(world, scene, rng)
    local toGame = NavScene.toGame
    local centre = NavScene.around(EntityPosition.Get(bodyNamed(scene, ORE.body)), ORE.angleDeg, ORE.distanceM)
    scene.oreCentre = centre
    scene.oreRocks = {}
    local placed = {}
    for _ = 1, ORE.count * 50 do
        if #placed >= ORE.count then break end
        local radiusM = rng:getUniformRange(ORE.minRadiusM, ORE.maxRadiusM)
        local x, y, z = rng:getUniformRange(-1, 1), rng:getUniformRange(-0.25, 0.25), rng:getUniformRange(-1, 1)
        local p = { x = x * ORE.halfExtentM, y = y * ORE.halfExtentM, z = z * ORE.halfExtentM, r = radiusM }
        local free = true
        for _, q in ipairs(placed) do
            local dx, dy, dz = p.x - q.x, p.y - q.y, p.z - q.z
            if math.sqrt(dx * dx + dy * dy + dz * dz) < p.r + q.r + ORE.gapM then
                free = false
                break
            end
        end
        if free then
            placed[#placed + 1] = p
            local position = Position(centre.x + toGame(p.x), centre.y + toGame(p.y), centre.z + toGame(p.z))
            scene.oreRocks[#scene.oreRocks + 1] = createOreRock(world, rng:get31(), position, toGame(radiusM))
        end
    end
end

---Builds the navigation demo's system, then the stations and the ore field, and equips them.
---@param world Physics
---@return EconomyDemoScene
function Scene.build(world)
    local scene = NavScene.build(world)
    ---@cast scene EconomyDemoScene
    local rng = RNG.Create(SEED)
    scene.stations = {}

    local function add(entity, spec)
        local definition = assert(StationEconomies[spec.economy], "EconomyDemo: no station economy " .. spec.economy)
        EconomySetup.equipStation(entity, definition, spec.name)
        scene.stations[#scene.stations + 1] = { entity = entity, definition = definition, name = spec.name }
    end

    add(scene.station, COLONY)
    for _, spec in ipairs(STATIONS) do
        local parent = bodyNamed(scene, spec.body)
        local position = NavScene.around(EntityPosition.Get(parent), spec.angleDeg, spec.distanceM)
        add(createStation(world, rng:get31(), position, parent), spec)
    end
    buildOreField(world, scene, rng)
    return scene
end

Scene.spawnShip = NavScene.spawnShip
Scene.toGame = NavScene.toGame
Scene.around = NavScene.around

return Scene
