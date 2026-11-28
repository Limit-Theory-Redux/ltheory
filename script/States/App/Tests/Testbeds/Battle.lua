-- Battle testbed: two hostile fleets of armed fighters meet over open space, to watch and to measure
-- heavy weapon fire. A trading guild's fighters and a marauder band's start a few kilometres apart;
-- each fighter is on a patrol round the middle (a fighting job, not one the economy gave it), so
-- every one engages the nearest enemy within its guard range and its weapons do the rest. Lost
-- fighters leave wrecks (data, made hulks near the camera) and are replaced at their side's start,
-- so the battle keeps its size for as long as it runs.
-- The log reports, every few seconds, the ships each side has, the hits, damage and kills since the
-- start, and the frame time; the orbit camera follows a ship, Left and Right pick another.
--
-- Launch: ./bin/ltr.exe Testbeds/Battle
--         (or: cargo run -- Testbeds/Battle)
-- Environment:
--   LTHEORY_BATTLE_SHIPS      fighters a side (default 20)
--   LTHEORY_BATTLE_SECONDS    simulated seconds to run, then quit with a summary (default: until closed)
--   LTHEORY_BATTLE_REINFORCE  0 to let the fleets wear down instead of replacing losses (default 1)
--   LTHEORY_BATTLE_HULL       the fighters' hull strength (default: their own), lower for quick kills
-- Camera: right mouse turns, the wheel zooms.

local Application             = require("States.Application")
local Registry                = require("Core.ECS.Registry")
local DrawEx                  = require("UI.DrawEx")
local Nav                     = require("Modules.Navigation")
local PhysicsComponents       = require("Modules.Physics.Components")
local CelestialEntities       = require("Modules.CelestialObjects.Entities")
local SolarSystemVisualizer   = require("Modules.CelestialObjects.Managers.SolarSystemVisualizer")
local CelestialLightingSystem = require("Modules.Rendering.Systems.CelestialLightingSystem")
local RenderCoreSystem        = require("Modules.Rendering.Systems.RenderCoreSystem")
local ShipFlightSystem        = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem       = require("Modules.Constructs.Systems.TravelDriveSystem")
local WeaponCombatSystem      = require("Modules.Constructs.Systems.WeaponCombatSystem")
local ExplosionEffectSystem   = require("Modules.Constructs.Systems.ExplosionEffectSystem")
local ConstructManager        = require("Modules.Constructs.Managers.ConstructManager")
local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
local _                       = require("Modules.Cameras.Systems.CameraSystem")
local Skybox                  = require("Shared.Generation.Skybox")
local ScaleConfig             = require("Config.Gen.UniverseScaleConfig")
local DamageEvents            = require("Shared.Helpers.DamageEvents")
local Factions                = require("Modules.Factions.Components")
local FactionManager          = require("Modules.Factions.Managers.FactionManager")
-- Required for its event subscriptions: it flies the fighting jobs.
local _                       = require("Modules.Factions.Systems.CombatAgentSystem")
local SpawnerSystem           = require("Modules.Spawning.Systems.SpawnerSystem")
local WreckSystem             = require("Modules.Salvage.Systems.WreckSystem")
local Presence                = require("Modules.Presence")
local CoreComponents          = require("Modules.Core.Components")
local Actions                 = require("Input.ActionBindings.EconomyDemoActions")

---@class BattleTestbed: Application
local BattleTestbed = Subclass("BattleTestbed", Application)

local SEED = 20261127
local CAMERA_NAME = "BattleTestbed"
local STAR_RADIUS_M, STAR_DISTANCE_M = 7e8, 1.5e11
local SHIPS = tonumber(os.getenv('LTHEORY_BATTLE_SHIPS')) or 20
local SECONDS = tonumber(os.getenv('LTHEORY_BATTLE_SECONDS'))
local REINFORCE = os.getenv('LTHEORY_BATTLE_REINFORCE') ~= '0'
local HULL = tonumber(os.getenv('LTHEORY_BATTLE_HULL'))
-- In game units (a unit is 10 km): where each side starts, and the patrol both fly round the middle.
local STARTS = { Position(-0.8, 0, 0), Position(0.8, 0, 0) }
local PATROL_RADIUS, PATROL_POINTS = 0.3, 6
local SIM_STEP_S = 0.05
local LOG_EVERY_S = 5

local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 22

function BattleTestbed:onInit()
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)
    self.skybox = Skybox.Create(SEED)

    local star = CelestialEntities.Star(SEED)
    local transform = star:get(PhysicsComponents.Transform)
    transform:setScale(ScaleConfig:toGameUnits(STAR_RADIUS_M, "star"))
    local d = ScaleConfig:toGameUnits(STAR_DISTANCE_M, "starSystem")
    transform:setPos(Position(-d * 0.6, d * 0.3, d * 0.75))
    SolarSystemVisualizer:materializeEntity(star, self.world)
    self.star = star

    Nav.attach(self.world)
    self.constructs = ConstructManager(Registry, self.world)
    self.weaponCombat = WeaponCombatSystem(self.world)
    SpawnerSystem:attach(self.world, self.constructs, { weaponCombat = self.weaponCombat })

    -- No credits: neither side buys ships of its own.
    local _, guildLeader = SpawnerSystem:spawnFaction({ type = Enums.FactionType.TradingGuild, seed = SEED, credits = 0 })
    local _, boss = SpawnerSystem:spawnFaction({ type = Enums.FactionType.Marauders, seed = SEED + 1, credits = 0 })
    self.patrol = {}
    for i = 1, PATROL_POINTS do
        local angle = (i - 1) / PATROL_POINTS * 2 * math.pi
        self.patrol[i] = Position(math.cos(angle) * PATROL_RADIUS, 0, math.sin(angle) * PATROL_RADIUS)
    end
    ---@type { leader: Entity, start: Position, ships: Entity[], lost: integer }[]
    self.sides = {
        { name = "guild", leader = guildLeader, start = STARTS[1], ships = {}, lost = 0 },
        { name = "marauders", leader = boss, start = STARTS[2], ships = {}, lost = 0 },
    }
    for _, side in ipairs(self.sides) do
        for i = 1, SHIPS do self:spawnFighter(side, i) end
    end

    self.hits, self.damage = 0, 0
    self.unsubscribeDamage = DamageEvents.subscribe(function(_, _, amount)
        self.hits = self.hits + 1
        self.damage = self.damage + amount
    end)
    self.simTime, self.logClock, self.frames = 0, 0, 0
    self.realClock = TimeStamp.Now()
    self.lastLogReal = 0

    self:createCamera()
    self.selected = 1
    self:select(1)
end

---A fighter for `side`, in a block round its start, on a patrol round the middle.
---@param side table
---@param index integer
function BattleTestbed:spawnFighter(side, index)
    local row, column = math.floor((index - 1) / 10), (index - 1) % 10
    local x = side.start.x + (side.start.x < 0 and -1 or 1) * row * 0.04
    local position = Position(x, (column - 4.5) * 0.04, (row % 2) * 0.02)
    local ship = SpawnerSystem:spawnAgent({ kind = "Fighter", owner = side.leader, position = position, economy = false })
    ship:add(Factions.CombatAgent(Enums.CombatTask.Patrol, self.patrol))
    local health = HULL and ship:get(CoreComponents.Health)
    if health then
        health:setMaxHealth(HULL)
        health:setCurrentHealth(HULL)
    end
    side.ships[#side.ships + 1] = ship
end

---Drops the lost ships from each side and, when reinforcing, replaces them.
function BattleTestbed:upkeep()
    for _, side in ipairs(self.sides) do
        local alive = {}
        for _, ship in ipairs(side.ships) do
            if Registry:hasEntity(ship) then alive[#alive + 1] = ship else side.lost = side.lost + 1 end
        end
        side.ships = alive
        if REINFORCE then
            for i = #alive + 1, SHIPS do self:spawnFighter(side, i) end
        end
    end
end

function BattleTestbed:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, {
        initialPitch = 0.35, distance = 0.4, minDistance = 0.01, maxDistance = 50, zoomRatio = 0.15, mouseSensitivity = 0.01,
    })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
    self.camera:enable()
end

---@return Entity[]
function BattleTestbed:ships()
    local list = {}
    for _, side in ipairs(self.sides) do
        for _, ship in ipairs(side.ships) do
            if Registry:hasEntity(ship) then list[#list + 1] = ship end
        end
    end
    return list
end

---@param index integer
function BattleTestbed:select(index)
    local ships = self:ships()
    if #ships == 0 then return end
    self.selected = (index - 1) % #ships + 1
    self.followed = ships[self.selected]
    self.camera:setTarget(self.followed)
end

---@param data EventData
function BattleTestbed:onSim(data)
    local dt = math.min(data:deltaTime(), SIM_STEP_S)
    if dt <= 0 then return end
    self.simTime = self.simTime + dt
    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    self.weaponCombat:preUpdate(dt)
    self.world:update(dt)
    self.weaponCombat:update(dt)
end

---What the battle is now, as lines of text.
---@return string[]
function BattleTestbed:statusLines()
    local real = self.realClock:getElapsed()
    local frameMs = self.frames > 0 and (real - self.lastLogReal) * 1000 / self.frames or 0
    local lines = {
        string.format("%.0f s simulated, %.1f ms a frame | hits %d, damage %.0f", self.simTime, frameMs, self.hits, self.damage),
    }
    for _, side in ipairs(self.sides) do
        local engaging = 0
        for _, ship in ipairs(side.ships) do
            local agent = Registry:hasEntity(ship) and ship:get(Factions.CombatAgent)
            if agent and agent:getTarget() then engaging = engaging + 1 end
        end
        lines[#lines + 1] = string.format("%s: %d ships, %d engaging, %d lost", side.name, #side.ships, engaging, side.lost)
    end
    local wrecks, hulks = 0, 0
    for _, wreck in ipairs(WreckSystem:getWrecks()) do
        wrecks = wrecks + 1
        if Presence.System:bodyOf(wreck) then hulks = hulks + 1 end
    end
    lines[#lines + 1] = string.format("wrecks: %d, %d of them hulks in the scene", wrecks, hulks)
    return lines
end

---@param data EventData
function BattleTestbed:onPostSim(data)
    self.weaponCombat:postUpdate(data:deltaTime())
    ExplosionEffectSystem:update(data:deltaTime())
    self:upkeep()
    if self.followed and not Registry:hasEntity(self.followed) then self:select(self.selected) end
    self.frames = self.frames + 1
    self.logClock = self.logClock + data:deltaTime()
    if self.logClock >= LOG_EVERY_S then
        self.logClock = 0
        for _, line in ipairs(self:statusLines()) do Log.Info("BattleTestbed: %s", line) end
        self.frames, self.lastLogReal = 0, self.realClock:getElapsed()
    end
    if SECONDS and self.simTime >= SECONDS then
        Log.Info("BattleTestbed: done after %.0f simulated seconds", self.simTime)
        self:quit()
    end
end

---@param data EventData
function BattleTestbed:onInput(data)
    if self:handleAppInput() then return end
    local dt = data:deltaTime()
    for _, action in pairs(Actions) do action:update(dt) end
    if Actions.Next:isPressed() then self:select(self.selected + 1) end
    if Actions.Prev:isPressed() then self:select(self.selected - 1) end
end

---@param data EventData
function BattleTestbed:onRender(data)
    CelestialLightingSystem:update(self.star)
    RenderCoreSystem:render(data)
    local lines = { "Battle testbed (Left/Right: another ship)" }
    for _, line in ipairs(self:statusLines()) do lines[#lines + 1] = line end
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, 40, 20,
                0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        end
    end)
end

return BattleTestbed
