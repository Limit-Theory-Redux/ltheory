-- Sensor Testbed: the sensor model (Modules/Sensors) in a scene. The player's
-- ship carries a sensor with a hull's dials (fighter, frigate, capital ship)
-- and a signature of its own (its power draw, louder and hotter while it
-- scans). Around it, one setup at a time (SensorTestbedSetups): small scenes
-- that each show one thing, such as how the hulls differ, how ships hide, what
-- passive and active sensing give, how resolution merges signals and who hears
-- you. The star system's star, planet and moon have signatures, and belt rocks
-- that come close get one too.
--
-- Passive contacts update all the time; X (held) scans the target, Z sends a
-- pulse. The sensor HUD (UIComponent.SensorHud) shows the radar and the
-- target; the text shows the setup, the settings and the keys, which change
-- the hull, the surroundings, a ship's activity and the model's rules.
-- Regenerate (B) builds the next setup around the ship as it is now, R the same
-- one again; both clear what the sensor learned.
--
-- Objects smaller than a pixel or so at their distance get no mesh, only their
-- signature: they would not show on screen anyway.

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
local SensorTuning = require("Modules.Sensors.SensorTuning")
local Setups = require("Config.Scenarios.SensorTestbedSetups")

-- A star is seen across its whole system: its signature is scaled so that at
-- its distance it reads as a star does at this distance in the sensor model's
-- reference scene.
local STAR_REFERENCE = { dist = 6e7, em = 1e7, heat = 1e10, glow = 3e10 }
local PLANET_HEAT = 50
-- Radius over distance below which an object gets no mesh (about a pixel on
-- a 1080p screen)
local MESH_MIN_SIZE = 0.001
local PULSE_HEARD_TIME = 4 -- seconds the text says who heard a pulse
local HINT_CHARS = 110     -- the setup's hint wraps at about this many characters
-- Holding left or right on the tuning panel steps its dial again after this
-- long, then this often (seconds)
local REPEAT_AFTER, REPEAT_EVERY = 0.4, 0.06

local G = UniverseScaleConfig.globalScale -- game units per real metre

-- The HUD's size: the window height, times the UI Scale setting
local function hudScale() return Window:height() / 1080 * GameState.ui.hudScale end
local sensorHud = UIComponent.SensorHud { scale = hudScale }
local tuningPanel = UIComponent.SensorTuningPanel { scale = hudScale }

---@param game table
---@return SensorComponent|nil
local function sensorOf(game)
    return game.playerShip and game.playerShip:get(SensorComponents.Sensor)
end

local function currentSetup(game)
    return Setups[game.scenarioData.setupIndex or 1]
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
        rb:setScale(rb:getScale() * radius * G / have)
    end
end

--- An object's entity (and its construct handle, for ships): a mesh of its
--- size, or a bare place in space when it is too small to see from `dist`.
---@return Entity, table|nil
local function newObject(game, spec, position, dist)
    local seed = game.rng:get31()
    if spec.radius / math.max(1, dist) < MESH_MIN_SIZE then
        local entity = Entity.Create("SensorMarker", PhysicsComponents.Transform())
        entity:get(PhysicsComponents.Transform):setPos(position)
        game:addSceneEntity(entity)
        return entity, nil
    end
    if spec.mesh == "station" then
        local station = StationGenerator:createStation(seed, { position = position, scale = 1, isKinematic = true })
        fitRadius(station, spec.radius)
        game:addSceneEntity(station, { addBody = true })
        return station, nil
    elseif spec.mesh == "asteroid" then
        -- An asteroid mesh has radius 1 (as the belt's rocks)
        return newAsteroid(game, seed, position, spec.radius * G), nil
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
    return handle.root, handle
end

local function distanceBetween(a, b)
    local dx, dy, dz = b.x - a.x, b.y - a.y, b.z - a.z
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

--- `v` turned `degrees` round the unit axis `k` (Rodrigues' formula).
local function turned(v, k, degrees)
    local a = math.rad(degrees)
    local c, s = math.cos(a), math.sin(a)
    local kv = k.x * v.x + k.y * v.y + k.z * v.z
    local cx, cy, cz = k.y * v.z - k.z * v.y, k.z * v.x - k.x * v.z, k.x * v.y - k.y * v.x
    return {
        x = v.x * c + cx * s + k.x * kv * (1 - c),
        y = v.y * c + cy * s + k.y * kv * (1 - c),
        z = v.z * c + cz * s + k.z * kv * (1 - c),
    }
end

--- Where an object goes, around the ship at `pos` (game units) facing
--- `forward` with `right` to its right and `up` above; and its distance in
--- metres.
---@return Position, number
local function placeObject(game, spec, pos, forward, right, up)
    local bodyName = spec.toward or spec.beyond
    local body = bodyName == "star" and game.starEntity or bodyName == "planet" and game.planets and game.planets[1]
    if body then
        local transform = body:get(PhysicsComponents.Transform)
        local c = transform:getPos()
        local len = distanceBetween(pos, c)
        local dx, dy, dz = (c.x - pos.x) / len, (c.y - pos.y) / len, (c.z - pos.z) / len
        if spec.toward then
            local dir = turned({ x = dx, y = dy, z = dz }, up, spec.offset or 0)
            local d = spec.dist * G
            return Position(pos.x + dir.x * d, pos.y + dir.y * d, pos.z + dir.z * d), spec.dist
        end
        local out = transform:getScale() + spec.gap * G
        local p = Position(c.x + dx * out, c.y + dy * out, c.z + dz * out)
        return p, distanceBetween(pos, p) / G
    end
    local b = math.rad(spec.bearing or 0)
    local dist = spec.dist or 11000
    local d = dist * G
    local dx = forward.x * math.cos(b) + right.x * math.sin(b)
    local dy = forward.y * math.cos(b) + right.y * math.sin(b)
    local dz = forward.z * math.cos(b) + right.z * math.sin(b)
    return Position(pos.x + dx * d, pos.y + dy * d, pos.z + dz * d), dist
end

--- Signatures for the star system's own bodies.
local function signCelestials(game)
    local shipPos = SensorSystem:positionOf(game.playerShip)
    if game.starEntity then
        local k = (distanceBetween(shipPos, SensorSystem:positionOf(game.starEntity)) / G / STAR_REFERENCE.dist) ^ 2
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

local function clearSetup(game)
    for _, object in ipairs(game.scenarioData.objects or {}) do
        if object.handle then
            game:removeSceneConstruct(object.handle)
        elseif object.entity:isValid() then
            game:removeSceneEntity(object.entity)
        end
    end
    game.scenarioData.objects = {}
end

--- Builds setup `index` around the ship as it is now, with its hull and
--- surroundings, and clears what the sensor learned.
local function buildSetup(game, index)
    local data = game.scenarioData
    clearSetup(game)
    data.setupIndex = index
    local setup = Setups[index]
    data.hull = setup.hull

    local sensor = sensorOf(game)
    sensor:setDials(SensorModel.hulls[setup.hull])
    sensor:setSurroundings(setup.surroundings or "open")
    sensor:setResolutionRule("background")
    sensor:setMerging(true)
    sensor:reset()

    local body = game.playerShip:get(PhysicsComponents.RigidBody)
    local pos, forward, right, up = body:getPos(), body:getForward(), body:getRight(), body:getUp()
    data.wantTarget = nil
    for _, spec in ipairs(setup.objects) do
        local position, dist = placeObject(game, spec, pos, forward, right, up)
        local entity, handle = newObject(game, spec, position, dist)
        local signature = SensorComponents.SensorSignature {
            kind = spec.kind, label = spec.label, radius = spec.radius, mw = spec.mw, activity = spec.activity,
            name = spec.name, details = spec.details, contents = spec.contents,
            shield = spec.shield, standing = spec.standing,
        }
        signature:getObject().hot = spec.hot or 0
        entity:add(signature)
        if spec.listener then
            entity:add(SensorComponents.Sensor(SensorModel.hulls.frigate))
        end
        data.objects[#data.objects + 1] = { entity = entity, handle = handle, spec = spec }
        if spec.label == setup.target and not data.wantTarget then data.wantTarget = entity end
    end
end

--- Your ship as others hear it: its power draw, plus the scan's power while
--- it scans, and the heat scanning builds.
local function updateOwnSignature(game, sensor)
    local signature = game.playerShip:get(SensorComponents.SensorSignature)
    if not signature then return end
    local own = SensorModel.ownEmission(sensor:getState())
    signature:setPowerDraw(own.mw)
    signature:getObject().hot = own.hot
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

--- Words of `str` in lines of at most `chars` characters.
local function wrap(str, chars)
    local lines, line = {}, ""
    for word in str:gmatch("%S+") do
        if line ~= "" and #line + 1 + #word > chars then
            lines[#lines + 1] = line
            line = word
        else
            line = line == "" and word or line .. " " .. word
        end
    end
    if line ~= "" then lines[#lines + 1] = line end
    return lines
end

--- One line per object with a sensor of its own: whether it hears you, and
--- whether it heard your last pulse.
local function listenerLines(game, sensor)
    local lines = {}
    local state = sensor:getState()
    local pulseHeard = state.pulseAge < PULSE_HEARD_TIME
    local shipPos = SensorSystem:positionOf(game.playerShip)
    for _, object in ipairs(game.scenarioData.objects) do
        local listener = object.entity:isValid() and object.entity:get(SensorComponents.Sensor)
        if listener then
            local reading
            for _, view in ipairs(listener:getObjects()) do
                if view.entity.id == game.playerShip.id then reading = listener:getReadings()[view] end
            end
            local heard = reading and reading.tier >= 1
            local line = string.format("%s: %s", string.upper(object.spec.label),
                heard and string.format("HEARS YOU, T%d (%s)", reading.tier, string.upper(reading.why))
                or "DOESN'T HEAR YOU" .. (reading and " (" .. string.upper(reading.why) .. ")" or ""))
            local dist = distanceBetween(shipPos, SensorSystem:positionOf(object.entity)) / G
            if pulseHeard and dist <= SensorModel.heardWithin(state.dials.power) then
                line = line .. " · HEARD YOUR PULSE"
            end
            lines[#lines + 1] = line
        end
    end
    return lines
end

--- The tuning panel's keys: up and down pick a dial, left and right change
--- it (again and again while held), Backspace puts the hull's values back.
local function tuningInput(game, sensor, dt)
    local data, tuning = game.scenarioData, game.scenarioData.tuning
    local count = #SensorTuning.dials
    if Input:isPressed(Button.KeyboardUp) then tuning.selected = (tuning.selected - 2) % count + 1 end
    if Input:isPressed(Button.KeyboardDown) then tuning.selected = tuning.selected % count + 1 end
    if Input:isPressed(Button.KeyboardBackspace) then sensor:setDials(SensorModel.hulls[data.hull]) end

    local direction = (Input:isDown(Button.KeyboardRight) and 1 or 0) - (Input:isDown(Button.KeyboardLeft) and 1 or 0)
    if direction == 0 then return end
    local spec = SensorTuning.dials[tuning.selected]
    local step = Input:isPressed(Button.KeyboardRight) or Input:isPressed(Button.KeyboardLeft)
    if step then
        tuning.repeatIn = REPEAT_AFTER
    elseif not spec.toggle then
        tuning.repeatIn = tuning.repeatIn - dt
        if tuning.repeatIn <= 0 then
            step, tuning.repeatIn = true, REPEAT_EVERY
        end
    end
    if step then
        local dials = SensorModel.copyDials(sensor:getDials())
        dials[spec.key] = SensorTuning.step(spec, dials[spec.key], direction)
        sensor:setDials(dials)
    end
end

return {
    name = "Sensor Testbed",
    description = "Passive and active sensors on your ship, in six setups (B for the next); X scans, Z pulses.",

    -- Halfway between the first planet's surface and the belt rock nearest to
    -- it (where the game starts the ship otherwise): clear of the belt, so the
    -- setups' own objects are what the sensor sees close by.
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
        signCelestials(game)
        game.playerShip:add(SensorComponents.Sensor(SensorModel.hulls.frigate))
        game.playerShip:add(SensorComponents.SensorSignature { kind = "ship", label = "You" })
        game.scenarioData.tuning = { shown = false, selected = 1, repeatIn = 0 }
        buildSetup(game, 1)
        sensorHud:update(nil)
        tuningPanel:update(nil)
    end,

    update = function(game, dt)
        local data = game.scenarioData
        local sensor = sensorOf(game)
        if not sensor then return end
        signBeltRocks()
        updateOwnSignature(game, sensor)
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
        sensorHud:update(SensorPicture.build(sensor, body:getForward(), body:getRight()), dt)
        tuningPanel:update(data.tuning.shown
            and { sensor = sensor, hull = data.hull, selected = data.tuning.selected } or nil)
    end,

    hud = { sensorHud, tuningPanel },

    regenerate = function(game)
        buildSetup(game, game.scenarioData.setupIndex % #Setups + 1)
    end,

    onInput = function(game, dt)
        local data = game.scenarioData
        local sensor = sensorOf(game)
        if not sensor then return end

        sensor:setScanning(Input:isDown(Button.KeyboardX))
        if Input:isPressed(Button.KeyboardZ) then sensor:requestPulse() end
        if Input:isPressed(Button.KeyboardTab) then nextTarget(sensor) end
        if Input:isPressed(Button.KeyboardR) then buildSetup(game, data.setupIndex) end

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
            local target = sensor:getTarget()
            local signature = target and target.kind == "ship" and target.entity:isValid()
                and target.entity:get(SensorComponents.SensorSignature)
            if signature then
                signature:setActivity(nextOf(SensorModel.activities, signature:getActivity()))
            end
        end
        if Input:isPressed(Button.KeyboardKey6) then
            sensor:setResolutionRule(nextOf(SensorModel.resolutionRules, sensor:getResolutionRule()))
        end
        if Input:isPressed(Button.KeyboardKey7) then
            sensor:setMerging(not sensor:isMerging())
        end
        if Input:isPressed(Button.KeyboardKey8) then data.tuning.shown = not data.tuning.shown end
        if data.tuning.shown then tuningInput(game, sensor, dt) end
    end,

    controls = {
        { keys = "X (hold)", action = "Scan the target" },
        { keys = "Z", action = "Pulse" },
        { keys = "Tab", action = "Next target (nearest first)" },
        { keys = "1 2 3", action = "Hull: fighter, frigate, capital ship" },
        { keys = "4", action = "Surroundings: open space, nebula, debris field, radiation belt" },
        { keys = "5", action = "The target's activity, if it is a ship: idle, thrusting, firing, cloaked" },
        { keys = "6", action = "Resolution rule: background in the cell, dims the target" },
        { keys = "7", action = "Merge signals in one cell on / off" },
        { keys = "B", action = "Next setup (rebuilt around the ship; clears the sensor)" },
        { keys = "R", action = "Start this setup again" },
        { keys = "8", action = "Tuning panel: the numbers behind the target's reading, and the dials" },
        { keys = "Up Down", action = "Tuning panel: pick a dial" },
        { keys = "Left Right", action = "Tuning panel: change the dial (hold to keep changing it)" },
        { keys = "Backspace", action = "Tuning panel: put the hull's dial values back" },
    },

    overlayLines = function(game)
        local data = game.scenarioData
        local sensor = sensorOf(game)
        if not sensor then return { "Sensor Testbed" } end
        local target = sensor:getTarget()
        local setup = currentSetup(game)
        local lines = {
            string.format("SETUP %d / %d · %s", data.setupIndex, #Setups, string.upper(setup.name)),
        }
        for _, line in ipairs(wrap(setup.hint or "", HINT_CHARS)) do lines[#lines + 1] = line end
        lines[#lines + 1] = ""
        for _, line in ipairs({
            string.format("SENSOR TESTBED · %s · %s · %s RULE · MERGE %s%s",
                string.upper(SensorModel.hullNames[data.hull]),
                string.upper(SensorModel.surroundingsNames[sensor:getSurroundings()]),
                string.upper(sensor:getResolutionRule()),
                sensor:isMerging() and "ON" or "OFF",
                target and target.kind == "ship" and target.activity and target.activity ~= "idle"
                    and " · TARGET " .. string.upper(SensorModel.activityNames[target.activity]) or ""),
            "X scan (hold) · Z pulse · Tab target · 1 2 3 hull · 4 surroundings · 5 activity · 6 rule · 7 merge"
                .. " · 8 tuning · B next setup · R again",
        }) do lines[#lines + 1] = line end
        for _, line in ipairs(listenerLines(game, sensor)) do lines[#lines + 1] = line end
        return lines
    end,
}
