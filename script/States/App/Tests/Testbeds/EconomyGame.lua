-- Economy game testbed: starts the game straight into the default scenario with its star system's
-- planned economy (factions, stations, haulers, miners, fighters, marauders) at the Normal density,
-- with the economy's state in the HUD, and runs it hands-off: time runs `TIME_SCALE` times fast, the
-- orbit camera visits each station and ship in turn, and the economy's state is logged now and then.
-- With the capture mode it gives screenshots of a running economy without anyone at the controls:
--
--   LTHEORY_CAPTURE=shot.png LTHEORY_CAPTURE_FRAME=1800 LTHEORY_CAPTURE_EXTRA_FRAMES=600,1200 \
--       ./bin/ltr.exe Testbeds/EconomyGame
--
-- Environment (all optional):
--   LTHEORY_ECONOMY_SEED      the universe seed (default 1)
--   LTHEORY_ECONOMY_TIMESCALE simulated seconds per second (default 4)
--   LTHEORY_ECONOMY_VISIT     frames the camera stays with each station or ship (default 300)
--   LTHEORY_ECONOMY_FOCUS     a name to keep the camera on (a station's or a ship's, e.g. "Fighter 2")
--   LTHEORY_ECONOMY_SHIPS     more ships to spawn at the start, dealt round the factions' stations (for
--                             measuring the simulation at a scale, with LTHEORY_PROFILE_SUBSCRIBERS)
-- Design: doc/script/spawning.md

local Registry        = require('Core.ECS.Registry')
local PhysicsComponents = require('Modules.Physics.Components')
local NameComponent   = require('Modules.Core.Components.NameComponent')
local Scenarios       = require('Config.Scenarios')
local EconomyOverlay  = require('Config.Scenarios.EconomyOverlay')
local SpawnerSystem   = require('Modules.Spawning.Systems.SpawnerSystem')
local Economy         = require('Modules.Economy.Components')
local Factions        = require('Modules.Factions.Components')
local FactionManager  = require('Modules.Factions.Managers.FactionManager')
local MarketplaceSystem = require('Modules.Economy.Systems.MarketplaceSystem')
local Game            = require('States.App.LTheoryRedux')

---@class EconomyGameTestbed: LimitTheoryRedux
local EconomyGame = Subclass("EconomyGameTestbed", Game)

local TIME_SCALE = tonumber(os.getenv('LTHEORY_ECONOMY_TIMESCALE')) or 4
local VISIT_FRAMES = tonumber(os.getenv('LTHEORY_ECONOMY_VISIT')) or 300
local FOCUS = os.getenv('LTHEORY_ECONOMY_FOCUS')
local EXTRA_SHIPS = tonumber(os.getenv('LTHEORY_ECONOMY_SHIPS')) or 0
-- The kinds of the extra ships, in turn: half haulers, three tenths miners, a fifth fighters.
local EXTRA_KINDS = { "Hauler", "Miner", "Hauler", "Fighter", "Hauler", "Miner", "Hauler", "Fighter", "Hauler", "Miner" }
local LOG_FRAMES = 300

-- The default scenario, with the economy at the Normal density and its state in the HUD.
local scenario = {}
for key, value in pairs(Scenarios.default) do scenario[key] = value end
scenario.economyDensity = 1
scenario.overlayLines = function() return EconomyGame.overlay() end
EconomyGame.autostartScenario = scenario
EconomyGame.autostartSeed = tonumber(os.getenv('LTHEORY_ECONOMY_SEED')) or 1

---The factions and their relations, then every station and ship.
---@return string[]
function EconomyGame.overlay()
    local lines = {}
    local factions = SpawnerSystem:getFactions()
    for i, a in ipairs(factions) do
        if Registry:hasEntity(a.entity) then
            local parts = {}
            for j, b in ipairs(factions) do
                if j ~= i and Registry:hasEntity(b.entity) then
                    parts[#parts + 1] = string.format("%s %.2f", b.entity:get(Factions.Faction):getName(), FactionManager:relation(a.entity, b.entity))
                end
            end
            lines[#lines + 1] = string.format("%s: %s", a.entity:get(Factions.Faction):getName(), table.concat(parts, ", "))
        end
    end
    for _, line in ipairs(EconomyOverlay()) do lines[#lines + 1] = line end
    return lines
end

---The stations and ships the camera visits, in order.
---@return Entity[]
local function sights()
    local list = {}
    for _, station in ipairs(SpawnerSystem:getStations()) do
        if Registry:hasEntity(station.entity) then list[#list + 1] = station.entity end
    end
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        if Registry:hasEntity(ship.entity) then list[#list + 1] = ship.entity end
    end
    return list
end

---@param entity Entity
---@return string
local function nameOf(entity)
    local name = entity:get(NameComponent)
    return name and name:getName() or tostring(entity)
end

---@param data EventData
function EconomyGame:onPreSim(data)
    Game.onPreSim(self, data)
    if not self.playerShip then return end
    self.directorFrames = (self.directorFrames or 0) + 1
    local frame = self.directorFrames

    if frame == 2 and EXTRA_SHIPS > 0 then self:spawnExtraShips() end
    local list = sights()
    if #list > 0 and (frame == 2 or frame % VISIT_FRAMES == 0) then
        local target = list[(math.floor(frame / VISIT_FRAMES)) % #list + 1]
        if FOCUS then
            for _, entity in ipairs(list) do
                if nameOf(entity) == FOCUS then target = entity end
            end
        end
        local body = target:get(PhysicsComponents.RigidBody):getRigidBody()
        self:setOrbitFocus(target, self:orbitDistances(body:getBoundingRadius() * (target:get(Economy.Marketplace) and 1 or 1.5)))
        self.directorTarget = target
        Log.Info("EconomyGame: frame %d, camera on %s", frame, nameOf(target))
    end
    if frame % LOG_FRAMES == 0 then
        Log.Info("EconomyGame: frame %d, %.0f s simulated", frame, MarketplaceSystem:now())
        self:logCamera()
        for _, line in ipairs(EconomyGame.overlay()) do Log.Info("EconomyGame: %s", line) end
    end
end

---Spawns `EXTRA_SHIPS` agents, dealt round the trading factions and their stations.
function EconomyGame:spawnExtraShips()
    local owners = {}
    for _, faction in ipairs(SpawnerSystem:getFactions()) do
        if faction.entity:get(Factions.Faction):getType() ~= Enums.FactionType.Marauders and #faction.stations > 0 then
            owners[#owners + 1] = faction
        end
    end
    if #owners == 0 then return end
    for i = 1, EXTRA_SHIPS do
        local faction = owners[(i - 1) % #owners + 1]
        local station = faction.stations[(math.floor((i - 1) / #owners)) % #faction.stations + 1]
        if Registry:hasEntity(station) then
            -- Round the station, and stacked in height, so that no two start inside each other.
            local at = SpawnerSystem:besideStation(station, 40 + i)
            at = Position(at.x, at.y + ((i % 30) - 15) * 0.04, at.z)
            SpawnerSystem:spawnAgent({ kind = EXTRA_KINDS[(i - 1) % #EXTRA_KINDS + 1], owner = faction.leader, position = at })
        end
    end
    Log.Info("EconomyGame: %d more ships spawned, %d agents in all", EXTRA_SHIPS, #SpawnerSystem:getAgents())
end

---Where the camera is and what it follows, for checking the director against the screenshots.
function EconomyGame:logCamera()
    local CameraManager = require('Modules.Cameras.Managers.CameraManager')
    local orbit = self.player and self.player.controllers[Enums.CameraMode.Orbit]
    local eye = CameraManager:getEye()
    local target = self.directorTarget
    local distance = "-"
    if target and Registry:hasEntity(target) and eye then
        local p = target:get(PhysicsComponents.RigidBody):getRigidBody():getPos()
        local dx, dy, dz = p.x - eye.x, p.y - eye.y, p.z - eye.z
        distance = string.format("%.3f", math.sqrt(dx * dx + dy * dy + dz * dz))
    end
    Log.Info("EconomyGame: camera mode %s, orbit target %s, eye to %s %s units", tostring(self.player and self.player:getModeName()),
        orbit and orbit.target and nameOf(orbit.target) or "-", target and nameOf(target) or "-", distance)
end

---The game's input sets the clock to real time each frame: the testbed takes it back after.
---@param data EventData
function EconomyGame:onInput(data)
    Game.onInput(self, data)
    EventBus:setTimeScale(TIME_SCALE)
end

return EconomyGame
