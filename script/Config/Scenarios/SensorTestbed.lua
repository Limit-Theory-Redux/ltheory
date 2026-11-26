-- Sensor Testbed: the sensor model (Modules/Sensors) in a scene. The player's
-- ship carries a sensor with a hull's dials (fighter, frigate, capital ship).
-- Around it, top down from its nose: a station at 12 km, a freighter at 4 km,
-- a fighter parked next to an asteroid at 11 km, a nearer asteroid and a cold
-- wreck at 400 m; further off, the star system's star, planet and moon. Belt
-- rocks that come close get a signature too.
--
-- Passive contacts update all the time; X (held) scans the target, Z sends a
-- pulse. The sensor HUD (UIComponent.SensorHud) shows the radar, the contacts
-- and the scan; the text shows the settings and the keys, which change the
-- hull, the surroundings, the fighter's activity and the model's rules.
-- Regenerate (B) puts everything back around the ship as it is now and clears
-- what the sensor learned.
--
-- The objects' sensor values (size, power draw, what a scan reads) are the
-- sensor model's reference scene, so they read as in the model's own test.
-- Their meshes are scaled to roughly that size.

local Entity = require("Core.ECS.Entity")
local CoreComponents = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local RenderComp = require("Modules.Rendering.Components").Render
local Materials = require("Shared.Registries.Materials")
local AsteroidMeshPool = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local AsteroidFieldSystem = require("Modules.CelestialObjects.Systems.AsteroidFieldSystem")
local StationGenerator = require("Modules.Constructs.Managers.Generators.StationGenerator")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local SensorComponents = require("Modules.Sensors.Components")
local SensorSystem = require("Modules.Sensors.Systems.SensorSystem")
local SensorModel = require("Modules.Sensors.SensorModel")
local SensorPicture = require("Modules.Sensors.SensorPicture")

-- The scene around the ship: distance (m) and bearing (degrees clockwise from
-- the nose, in the ship's plane), the sensor values, and what a T3 and a T4
-- read show. `mesh` is how it is built (`hull`: an index into
-- Config.game.shipHulls).
local OBJECTS = {
    {
        kind = "station", label = "Station", dist = 1.2e4, bearing = 150, radius = 316, mw = 500,
        mesh = "station",
        name = "COMMERCIAL NEXUS", details = "FACTION: LIMTECH",
        contents = { "TRADE DATA: 14 MARKETS", "DOCKING LOG: 31 SHIPS TODAY" },
    },
    {
        kind = "ship", label = "Freighter", dist = 4e3, bearing = -28, radius = 20, mw = 50,
        mesh = "ship", shipType = "Basic", hull = 3,
        name = "LIMTECH FREIGHTER (Y142)", details = "SHIELDS 78%",
        contents = { "CARGO: 40 T IRON ORE", "LOGS: 3 CONTRACTS, LAST DOCK 2 H AGO" },
    },
    {
        kind = "ship", label = "Fighter", dist = 1.06e4, bearing = 73.5, radius = 8, mw = 8,
        mesh = "ship", shipType = "Fighter", hull = 1,
        name = "VEGA DART (Y151)", details = "SHIELDS 100%",
        contents = { "LOADOUT: 2 PULSE CANNONS", "LOGS: PATROL ROUTE, 4 WAYPOINTS" },
    },
    {
        kind = "asteroid", label = "Asteroid", dist = 6e3, bearing = 64, radius = 500,
        mesh = "asteroid",
        name = "ASTEROID, 1 KM ACROSS",
        contents = { "IRON-RICH, YIELD HIGH", "NO SIGNS OF MINING" },
    },
    {
        kind = "asteroid", label = "Asteroid", dist = 1.1e4, bearing = 71, radius = 500,
        mesh = "asteroid",
        name = "ASTEROID, 1 KM ACROSS",
        contents = { "IRON-RICH, YIELD HIGH", "NO SIGNS OF MINING" },
    },
    {
        kind = "wreck", label = "Wreck", dist = 400, bearing = 4, radius = 30,
        mesh = "ship", shipType = "Basic", hull = 3,
        name = "DERELICT LIMTECH FALCON (Y98)", details = "NO POWER",
        contents = { "BLACKBOX: LAST ORDERS, 6 DAYS OLD", "BLUEPRINT FRAGMENT: 'FALCON' 4/6" },
    },
}
local FIGHTER = 3 -- the ship whose activity the keys change
local FIRST_TARGET = 2 -- the freighter

-- A star is seen across its whole system: its signature is scaled so that at
-- its distance it reads as a star does at this distance in the reference scene.
local STAR_REFERENCE = { dist = 6e7, em = 1e7, heat = 1e10, glow = 3e10 }
local PLANET_HEAT = 50

local sensorHud = UIComponent.SensorHud {
    -- The HUD's size: the window height, times the UI Scale setting
    scale = function() return Window:height() / 1080 * GameState.ui.hudScale end,
    scanHint = "HOLD X TO SCAN",
}

---@param game table
---@return SensorComponent|nil
local function sensorOf(game)
    return game.playerShip and game.playerShip:get(SensorComponents.Sensor)
end

--- Where each object goes: around the ship as it is now, in its plane.
local function placements(game)
    local body = game.playerShip:get(PhysicsComponents.RigidBody)
    local pos, forward, right = body:getPos(), body:getForward(), body:getRight()
    local result = {}
    for i, spec in ipairs(OBJECTS) do
        local b = math.rad(spec.bearing)
        local d = spec.dist * UniverseScaleConfig.globalScale
        local dx = forward.x * math.cos(b) + right.x * math.sin(b)
        local dy = forward.y * math.cos(b) + right.y * math.sin(b)
        local dz = forward.z * math.cos(b) + right.z * math.sin(b)
        result[i] = Position(pos.x + dx * d, pos.y + dy * d, pos.z + dz * d)
    end
    return result
end

local function newAsteroid(game, seed, position, scale)
    local entity = Entity.Create("AsteroidEntity",
        CoreComponents.Seed(seed),
        CoreComponents.Type("Asteroid"),
        PhysicsComponents.Transform())
    local transform = entity:get(PhysicsComponents.Transform)
    transform:setPos(position)
    transform:setScale(scale)
    local lodMesh = AsteroidMeshPool:getFromSeed(seed)
    local mesh = lodMesh and lodMesh:get(0)
    if mesh then
        entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))
    end
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(scale)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    game:addSceneEntity(entity, { addBody = true })
    return entity
end

--- Scales an entity's rigid body (and so its mesh) to a bounding radius of
--- `radius` metres. Generated ships and stations come in sizes of their own,
--- which their scale multiplies.
local function fitRadius(entity, radius)
    local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    local have = rb:getBoundingRadiusCompound()
    if have and have > 0 then
        rb:setScale(rb:getScale() * radius * UniverseScaleConfig.globalScale / have)
    end
end

local function newObject(game, spec, position)
    local seed = game.rng:get31()
    if spec.mesh == "station" then
        local station = StationGenerator:createStation(seed, {
            position = position,
            scale = 1,
            isKinematic = true,
        })
        fitRadius(station, spec.radius)
        game:addSceneEntity(station, { addBody = true })
        return station
    elseif spec.mesh == "asteroid" then
        -- An asteroid mesh has radius 1 (as the belt's rocks)
        return newAsteroid(game, seed, position, spec.radius * UniverseScaleConfig.globalScale)
    end
    local handle = game.constructManager:createTarget({
        seed = seed,
        shipType = Enums.ShipType[spec.shipType],
        hull = spec.hull,
        position = position,
        isKinematic = true,
    })
    fitRadius(handle.root, spec.radius)
    game:addSceneConstruct(handle)
    return handle.root
end

local function moveTo(entity, position)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    if rb then rb:setPos(position) end
    local transform = entity:get(PhysicsComponents.Transform)
    if transform then transform:setPos(position) end
end

local function distanceTo(game, entity)
    local a = SensorSystem:positionOf(game.playerShip)
    local b = SensorSystem:positionOf(entity)
    local dx, dy, dz = b.x - a.x, b.y - a.y, b.z - a.z
    return math.sqrt(dx * dx + dy * dy + dz * dz) / UniverseScaleConfig.globalScale
end

--- Signatures for the star system's own bodies.
local function signCelestials(game)
    if game.starEntity then
        local k = (distanceTo(game, game.starEntity) / STAR_REFERENCE.dist) ^ 2
        game.starEntity:add(SensorComponents.SensorSignature {
            kind = "star", label = "Star",
            fixedEm = STAR_REFERENCE.em * k, fixedHeat = STAR_REFERENCE.heat * k, glow = STAR_REFERENCE.glow * k,
            name = "STAR, CLASS G", contents = { "FLARE ACTIVITY: LOW" },
        })
    end
    for i, planet in ipairs(game.planets or {}) do
        planet:add(SensorComponents.SensorSignature {
            kind = "planet", label = "Planet", fixedHeat = PLANET_HEAT,
            name = string.format("PLANET %d", i), details = "ATMOSPHERE: THIN",
            contents = { "SURFACE: 2 COLONIES", "ORBIT: 1 STATION" },
        })
    end
    for _, labeled in ipairs(game.labeledEntities or {}) do
        if tostring(labeled.entity):find("MoonEntity") and not labeled.entity:get(SensorComponents.SensorSignature) then
            labeled.entity:add(SensorComponents.SensorSignature {
                kind = "planet", label = "Moon", fixedHeat = PLANET_HEAT, name = "MOON",
                contents = { "SURFACE: BARREN" },
            })
        end
    end
end

--- Belt rocks near the ship are spawned and removed as it moves; each new
--- one gets a signature (removed with it).
local function signBeltRocks()
    for _, info in ipairs(AsteroidFieldSystem:getSpawnedEntities()) do
        local entity = info.entity
        if entity and entity:isValid() and not entity:get(SensorComponents.SensorSignature) then
            entity:add(SensorComponents.SensorSignature { kind = "asteroid", label = "Belt rock" })
        end
    end
end

--- Everything the sensor sees, nearest first: what Tab cycles through.
local function byDistance(sensor)
    local list = {}
    for _, o in ipairs(sensor:getObjects()) do list[#list + 1] = o end
    table.sort(list, function(a, b) return SensorModel.distance(a) < SensorModel.distance(b) end)
    return list
end

local function nextTarget(sensor)
    local list = byDistance(sensor)
    if #list == 0 then return end
    local current = sensor:getTarget()
    local index = 0
    for i, o in ipairs(list) do
        if o == current then index = i end
    end
    sensor:setTarget(list[index % #list + 1])
end

local function nextOf(list, current)
    for i, value in ipairs(list) do
        if value == current then return list[i % #list + 1] end
    end
    return list[1]
end

local function place(game)
    local data = game.scenarioData
    for i, position in ipairs(placements(game)) do
        moveTo(data.objects[i], position)
    end
end

local function reset(game)
    local data = game.scenarioData
    place(game)
    local fighter = data.objects[FIGHTER]:get(SensorComponents.SensorSignature)
    fighter:setActivity("idle")
    fighter:getObject().hot = 0
    local sensor = sensorOf(game)
    sensor:reset()
    data.wantTarget = data.objects[FIRST_TARGET]
end

return {
    name = "Sensor Testbed",
    description = "Passive and active sensors on your ship; X scans, Z pulses, 1 2 3 pick a hull.",

    -- Halfway between the first planet's surface and the belt rock nearest to
    -- it (where the game starts the ship otherwise): clear of the belt, so the
    -- scene's own objects are what the sensor sees close by.
    spawnShip = function(game)
        local spawn = game:defaultShipSpawn()
        local planet = game.planets and game.planets[1]
        if not planet then return spawn end
        local transform = planet:get(PhysicsComponents.Transform)
        local p, radius = transform:getPos(), transform:getScale()
        local dx, dy, dz = spawn.x - p.x, spawn.y - p.y, spawn.z - p.z
        local d = math.sqrt(dx * dx + dy * dy + dz * dz)
        if d <= radius then return spawn end
        local at = (radius + 0.5 * (d - radius)) / d
        return Position(p.x + dx * at, p.y + dy * at, p.z + dz * at)
    end,

    setup = function(game)
        local data = game.scenarioData
        data.hull = "frigate"
        data.objects = {}
        for i, position in ipairs(placements(game)) do
            local spec = OBJECTS[i]
            local entity = newObject(game, spec, position)
            entity:add(SensorComponents.SensorSignature {
                kind = spec.kind, label = spec.label, radius = spec.radius, mw = spec.mw,
                name = spec.name, details = spec.details, contents = spec.contents,
            })
            data.objects[i] = entity
        end
        signCelestials(game)
        game.playerShip:add(SensorComponents.Sensor(SensorModel.hulls[data.hull]))
        data.wantTarget = data.objects[FIRST_TARGET]
        sensorHud:update(nil)
    end,

    update = function(game, dt)
        local data = game.scenarioData
        local sensor = sensorOf(game)
        if not sensor then return end
        signBeltRocks()
        SensorSystem:update(dt)
        -- The target is an object of the sensor's: pick the wanted entity's
        -- once the sensor has seen it
        if data.wantTarget then
            for _, o in ipairs(sensor:getObjects()) do
                if o.entity.id == data.wantTarget.id then
                    sensor:setTarget(o)
                    data.wantTarget = nil
                    break
                end
            end
        end
        local body = game.playerShip:get(PhysicsComponents.RigidBody)
        sensorHud:update(SensorPicture.build(sensor, body:getForward(), body:getRight(),
            SensorModel.hullNames[data.hull]), dt)
    end,

    hud = { sensorHud },

    regenerate = reset,

    onInput = function(game, dt)
        local data = game.scenarioData
        local sensor = sensorOf(game)
        if not sensor then return end

        sensor:setScanning(Input:isDown(Button.KeyboardX))
        if Input:isPressed(Button.KeyboardZ) then sensor:requestPulse() end
        if Input:isPressed(Button.KeyboardTab) then nextTarget(sensor) end

        local hullKeys = { Button.KeyboardKey1, Button.KeyboardKey2, Button.KeyboardKey3 }
        for i, key in ipairs(hullKeys) do
            if Input:isPressed(key) then
                data.hull = SensorModel.hullOrder[i]
                sensor:setDials(SensorModel.hulls[data.hull])
            end
        end
        if Input:isPressed(Button.KeyboardKey4) then
            sensor:setSurroundings(nextOf(SensorModel.surroundingsOrder, sensor:getSurroundings()))
        end
        if Input:isPressed(Button.KeyboardKey5) then
            local fighter = data.objects[FIGHTER]:get(SensorComponents.SensorSignature)
            fighter:setActivity(nextOf(SensorModel.activities, fighter:getActivity()))
        end
        if Input:isPressed(Button.KeyboardKey6) then
            sensor:setResolutionRule(nextOf(SensorModel.resolutionRules, sensor:getResolutionRule()))
        end
        if Input:isPressed(Button.KeyboardKey7) then
            sensor:setMerging(not sensor:isMerging())
        end
    end,

    controls = {
        { keys = "X (hold)", action = "Scan the target" },
        { keys = "Z", action = "Pulse" },
        { keys = "Tab", action = "Next target (nearest first)" },
        { keys = "1 2 3", action = "Hull: fighter, frigate, capital ship" },
        { keys = "4", action = "Surroundings: open space, nebula, debris field, radiation belt" },
        { keys = "5", action = "The fighter's activity: idle, thrusting, firing, cloaked" },
        { keys = "6", action = "Resolution rule: background in the cell, dims the target" },
        { keys = "7", action = "Merge signals in one cell on / off" },
        { keys = "B", action = "Put everything back around the ship; clear the sensor" },
    },

    overlayLines = function(game)
        local data = game.scenarioData
        local sensor = sensorOf(game)
        if not sensor then return { "Sensor Testbed" } end
        local fighter = data.objects[FIGHTER]:get(SensorComponents.SensorSignature)
        return {
            string.format("SENSOR TESTBED · %s · %s · %s RULE · MERGE %s · FIGHTER %s",
                string.upper(SensorModel.hullNames[data.hull]),
                string.upper(SensorModel.surroundingsNames[sensor:getSurroundings()]),
                string.upper(sensor:getResolutionRule()),
                sensor:isMerging() and "ON" or "OFF",
                string.upper(SensorModel.activityNames[fighter:getActivity()])),
            "X scan (hold) · Z pulse · Tab target · 1 2 3 hull · 4 surroundings · 5 fighter · 6 rule · 7 merge · B reset",
        }
    end,
}
