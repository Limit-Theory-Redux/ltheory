-- Factions testbed: a light scene to watch factions trade, guard and raid, with no player ship. A
-- trading guild runs a power plant and an outpost, with haulers and two armed fighters; a marauder
-- faction has armed fighters near the trade lane. No ship has a fixed role: the marauders' fighters
-- take raid jobs on the guild's haulers; the raids raise the threat at the guild's stations, which
-- then post paid security jobs that the guild's fighters take (until then they haul what pays). The
-- weapons fight under the factions' hostility. The orbit camera follows a marauder; Left and Right
-- pick another ship. The corner text shows every ship's kind, activity and earnings by job, the
-- factions' relation and the stations' threat.
--
-- Launch: ./bin/ltr.exe Testbeds/Factions
--         (or: cargo run -- Testbeds/Factions)
-- Camera: right mouse turns, the wheel zooms. Hold the time acceleration key to speed it up.
-- Design: doc/script/factions.md

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
local NameComponent           = require("Modules.Core.Components.NameComponent")
local Economy                 = require("Modules.Economy.Components")
local PeopleManager           = require("Modules.People.Managers.PeopleManager")
local Factions                = require("Modules.Factions.Components")
local FactionManager          = require("Modules.Factions.Managers.FactionManager")
local SpawnerSystem           = require("Modules.Spawning.Systems.SpawnerSystem")
local EconomyOverlay          = require("Config.Scenarios.EconomyOverlay")
local Actions                 = require("Input.ActionBindings.EconomyDemoActions")

---@class FactionsTestbed: Application
local FactionsTestbed = Subclass("FactionsTestbed", Application)

local SEED = 20261128
local CAMERA_NAME = "FactionsTestbed"
local STAR_RADIUS_M, STAR_DISTANCE_M = 7e8, 1.5e11
-- In game units (a unit is 10 km): the two stations, and the pirates' den halfway between them.
local POWER_AT, OUTPOST_AT, DEN_AT = Position(-6, 0, 0), Position(6, 0, 0), Position(0, 1.5, 2.5)
local TRADERS, SECURITY, PIRATES = 3, 2, 2 -- haulers and fighters of the guild, fighters of the marauders
-- A frame advances at most one step: navigation writes the controls once a frame.
local SIM_STEP_S = 0.05

local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 22

function FactionsTestbed:onInit()
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

    local guild, guildLeader = SpawnerSystem:spawnFaction({ type = Enums.FactionType.TradingGuild, seed = SEED, credits = 50000 })
    local marauders, boss = SpawnerSystem:spawnFaction({ type = Enums.FactionType.Marauders, seed = SEED + 1, credits = 10000 })
    self.factions = { guild, marauders }

    self.power = SpawnerSystem:spawnStation({ kind = "PowerPlant", name = "Power Plant", position = POWER_AT, owner = guildLeader })
    self.outpost = SpawnerSystem:spawnStation({ kind = "Outpost", name = "Outpost", position = OUTPOST_AT, owner = guildLeader })
    for i = 1, TRADERS do
        SpawnerSystem:spawnAgent({ kind = "Hauler", owner = guildLeader, position = Position(POWER_AT.x + 0.6, 0.3 * i, -0.6) })
    end
    for i = 1, SECURITY do
        SpawnerSystem:spawnAgent({ kind = "Fighter", owner = guildLeader, position = Position((i == 1 and POWER_AT or OUTPOST_AT).x, 0.8, 0.5) })
    end
    self.marauders = {}
    for i = 1, PIRATES do
        self.marauders[i] = SpawnerSystem:spawnAgent({ kind = "Fighter", owner = boss,
            position = Position(DEN_AT.x + 0.4 * i, DEN_AT.y, DEN_AT.z) })
    end

    self:createCamera()
    self.selected = 1
    self:select(1)
end

function FactionsTestbed:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, {
        initialPitch = 0.35, distance = 0.12, minDistance = 0.01, maxDistance = 50, zoomRatio = 0.15, mouseSensitivity = 0.01,
    })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
    self.camera:enable()
end

---The ships Left and Right step through: the marauders first, then the guild's.
---@return Entity[]
function FactionsTestbed:ships()
    local list = {}
    for _, ship in ipairs(self.marauders) do if Registry:hasEntity(ship) then list[#list + 1] = ship end end
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        local isMarauder = false
        for _, m in ipairs(self.marauders) do if m.id == ship.entity.id then isMarauder = true end end
        if not isMarauder and Registry:hasEntity(ship.entity) then list[#list + 1] = ship.entity end
    end
    return list
end

---@param index integer
function FactionsTestbed:select(index)
    local ships = self:ships()
    if #ships == 0 then return end
    self.selected = (index - 1) % #ships + 1
    self.camera:setTarget(ships[self.selected])
end

---@param data EventData
function FactionsTestbed:onSim(data)
    local dt = math.min(data:deltaTime(), SIM_STEP_S)
    if dt <= 0 then return end
    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    self.weaponCombat:preUpdate(dt)
    self.world:update(dt)
    self.weaponCombat:update(dt)
end

---@param data EventData
function FactionsTestbed:onPostSim(data)
    self.weaponCombat:postUpdate(data:deltaTime())
    ExplosionEffectSystem:update(data:deltaTime())
    self.logClock = (self.logClock or 0) + data:deltaTime()
    if self.logClock >= 3 then
        self.logClock = 0
        for _, line in ipairs(self:statusLines()) do Log.Info("FactionsTestbed: %s", line) end
    end
end

---@param data EventData
function FactionsTestbed:onInput(data)
    if self:handleAppInput() then return end
    local dt = data:deltaTime()
    for _, action in pairs(Actions) do action:update(dt) end
    if Actions.Next:isPressed() then self:select(self.selected + 1) end
    if Actions.Prev:isPressed() then self:select(self.selected - 1) end
end

---Every ship's kind, activity and earnings by job, the factions' relation and the stations' threat.
---@return string[]
function FactionsTestbed:statusLines()
    local lines = {}
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        local agent = Registry:hasEntity(ship.entity) and PeopleManager.commandOf(ship.entity, Economy.EconomyAgent)
        local earned = {}
        for kind, credits in pairs(agent and agent.earnedBy or {}) do earned[#earned + 1] = string.format("%s %d", kind, credits) end
        table.sort(earned)
        lines[#lines + 1] = string.format("%s (%s%s): %s%s", ship.name, FactionManager.factionOf(ship.entity) == self.factions[2] and "marauder, " or "",
            ship.battery and "armed" or "unarmed", agent and agent:getActivity() or "gone",
            #earned > 0 and (" | " .. table.concat(earned, ", ")) or "")
    end
    local a, b = self.factions[1], self.factions[2]
    local threats = require("Modules.Factions.Systems.ThreatSystem")
    lines[#lines + 1] = string.format("%s vs %s: %.2f | threat at Power Plant %.1f, Outpost %.1f", a:get(Factions.Faction):getName(),
        b:get(Factions.Faction):getName(), FactionManager.relation(a, b), threats:threatOf(self.power), threats:threatOf(self.outpost))
    return lines
end

---@param data EventData
function FactionsTestbed:onRender(data)
    CelestialLightingSystem:update(self.star)
    RenderCoreSystem:render(data)
    local ships = self:ships()
    local followed = ships[self.selected]
    local name = followed and followed:get(NameComponent)
    local lines = { string.format("Factions testbed: following %s (Left/Right: another ship)", name and name:getName() or "-") }
    for _, line in ipairs(self:statusLines()) do lines[#lines + 1] = line end
    for _, line in ipairs(EconomyOverlay()) do lines[#lines + 1] = line end
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, 40, 20,
                0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        end
    end)
end

return FactionsTestbed
