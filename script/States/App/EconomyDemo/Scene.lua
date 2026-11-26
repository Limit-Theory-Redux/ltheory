-- The economy demo's star system: the navigation demo's system, with a station of each economic
-- kind around the planet Aurora and its moons, and a field of iron ore rocks between them. Sizes and
-- distances are authored in metres. The bodies don't orbit: the stations stay where they are put.

local Registry          = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")
local StationGenerator  = require("Modules.Constructs.Managers.Generators.StationGenerator")
local EntityPosition    = require("Shared.Helpers.EntityPosition")
local Items             = require("Shared.Registries.Items")
local StationEconomies  = require("Shared.Definitions.StationEconomyDefs")
local EconomySetup      = require("Modules.Economy.Managers.EconomySetup")
local OreField          = require("Shared.Helpers.OreField")
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

---@param world Physics
---@param scene EconomyDemoScene
---@param rng RNG
local function buildOreField(world, scene, rng)
    local toGame = NavScene.toGame
    scene.oreCentre = NavScene.around(EntityPosition.Get(bodyNamed(scene, ORE.body)), ORE.angleDeg, ORE.distanceM)
    scene.oreRocks = OreField.build(world, rng, {
        centre = scene.oreCentre, halfExtent = toGame(ORE.halfExtentM), count = ORE.count,
        minRadius = toGame(ORE.minRadiusM), maxRadius = toGame(ORE.maxRadiusM), gap = toGame(ORE.gapM),
        item = Items.RawMaterials.IronOre, quantity = ORE.quantity, regrowPerSecond = ORE.regrowPerSecond,
    })
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
