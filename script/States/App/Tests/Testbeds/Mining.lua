-- Mining testbed: a light scene to watch mining in, with no player ship. A small field of iron ore
-- rocks, a refinery that buys the ore, and miners that fly to the rocks, circle them with their
-- lasers on the ore spots, and sell to the refinery. The orbit camera follows a miner: the first
-- one that starts mining, then Left and Right pick another. The corner text tells what each does.
--
-- Launch: ./bin/ltr.exe Testbeds/Mining
--         (or: cargo run -- Testbeds/Mining)
-- Camera: right mouse turns, the wheel zooms. Hold the time acceleration key to speed it up.
-- Design: doc/script/economy.md

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
local ConstructManager        = require("Modules.Constructs.Managers.ConstructManager")
local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
local _                       = require("Modules.Cameras.Systems.CameraSystem")
local Skybox                  = require("Shared.Generation.Skybox")
local ScaleConfig             = require("Config.Gen.UniverseScaleConfig")
local Items                   = require("Shared.Registries.Items")
local Economy                 = require("Modules.Economy.Components")
local PeopleManager           = require("Modules.People.Managers.PeopleManager")
local InventoryManager        = require("Modules.Economy.Managers.InventoryManager")
local MiningSystem            = require("Modules.Economy.Systems.MiningSystem")
local DockingSystem           = require("Modules.Docking.Systems.DockingSystem")
local SpawnerSystem           = require("Modules.Spawning.Systems.SpawnerSystem")
local EconomyOverlay          = require("Config.Scenarios.EconomyOverlay")
local Actions                 = require("Input.ActionBindings.EconomyDemoActions")

---@class MiningTestbed: Application
local MiningTestbed = Subclass("MiningTestbed", Application)

local SEED = 20261127
local CAMERA_NAME = "MiningTestbed"
-- The star only lights the scene: far off, in the direction it lights from.
local STAR_RADIUS_M, STAR_DISTANCE_M = 7e8, 1.5e11
local MINERS = 3
-- In game units (a unit is 10 km): the ore field at the origin, the refinery off to one side.
local ORE = { halfExtent = 1.2, count = 5, minRadius = 0.06, maxRadius = 0.1, gap = 0.25, quantity = 300, regrowPerSecond = 0.5 }
local REFINERY_AT = Position(4, 0.5, 0)
-- A frame advances at most one step: navigation writes the controls once a frame, and a frame that
-- ran several steps on one input would fly them blind (a slow frame slows the scene instead).
local SIM_STEP_S, MAX_ADVANCE_S = 0.05, 0.05

local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 22

function MiningTestbed:onInit()
    self.simSteps = 0
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
    SpawnerSystem:attach(self.world, self.constructs)
    local ore = { centre = Position(0, 0, 0), seed = SEED }
    for key, value in pairs(ORE) do ore[key] = value end
    SpawnerSystem:spawnOreField(ore)
    local refinery = SpawnerSystem:spawnStation({ kind = "Refinery", name = "Refinery", position = REFINERY_AT, seed = SEED })
    -- The refinery starts without ore, so that it bids for a hold of it from every miner.
    local inventory = refinery:get(Economy.Inventory)
    local ironOre = Items.RawMaterials.IronOre.id
    InventoryManager:remove(inventory, ironOre, InventoryManager:count(inventory, ironOre))
    self.miners = SpawnerSystem:spawnAgentsAround(refinery, { Miner = MINERS })

    self:createCamera()
    self.selected = 1
    self.autoSelect = true
    self:follow(self.miners[1])
end

function MiningTestbed:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, {
        initialPitch = 0.35, distance = 0.08, minDistance = 0.01, maxDistance = 50, zoomRatio = 0.15, mouseSensitivity = 0.01,
    })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
    -- A controller starts disabled: it takes the mouse and moves the camera once enabled.
    self.camera:enable()
end

---@param ship Entity
function MiningTestbed:follow(ship)
    self.camera:setTarget(ship)
end

---@param index integer Into `miners`, wrapping
function MiningTestbed:select(index)
    self.selected = (index - 1) % #self.miners + 1
    self.autoSelect = false
    self:follow(self.miners[self.selected])
end

---Until a miner is picked by hand, the camera goes to the first miner that starts mining.
function MiningTestbed:followWork()
    if not self.autoSelect then return end
    for index, ship in ipairs(self.miners) do
        local run = Registry:hasEntity(ship) and MiningSystem:runOf(ship)
        if run and not run:isDone() then
            if index ~= self.selected then
                self.selected = index
                self:follow(ship)
            end
            self.autoSelect = false
            return
        end
    end
end

---@param data EventData
function MiningTestbed:onPreSim(data)
    self:followWork()
    self.debugClock = (self.debugClock or 0) + data:deltaTime()
    if self.debugClock >= 2 then
        self.debugClock = 0
        Log.Info("MiningTestbed: %s", self:debugLine())
    end
end

---What the followed miner is doing, for tuning: its agent state and activity, its navigation
---order, its docking phase, and its speed and turn rate.
---@return string
function MiningTestbed:debugLine()
    local ship = self.miners[self.selected]
    if not Registry:hasEntity(ship) then return "gone" end
    local agent = PeopleManager:commandOf(ship, Economy.EconomyAgent)
    local queue = ship:get(Nav.Components.OrderQueue)
    local head = queue and queue:head()
    local order = head and Registry:getEntityById(head)
    local status = order and Nav.status(order)
    local body = ship:get(PhysicsComponents.RigidBody):getRigidBody()
    local v, w = body:getVelocity(), body:getVelocityA()
    local mps = 1 / ScaleConfig:toGameUnits(1, "starSystem")
    local progress = order and Nav.progress(order)
    local left = progress and progress.distanceLeft
    local p = body:getPos()
    return string.format("Miner %d: %s (state %s) | order %s, %s km left | docking %s | owner %s | %.0f m/s, %.2f rad/s | at (%.1f, %.1f, %.1f) km",
        self.selected, agent and agent:getActivity() or "-", agent and tostring(agent:getState()) or "-",
        tostring(status), left and string.format("%.1f", left * mps / 1000) or "-", tostring(DockingSystem:phaseOf(ship)),
        tostring(Nav.controlOwner(ship)), v:length() * mps, w:length(), p.x * mps / 1000, p.y * mps / 1000, p.z * mps / 1000)
end

---Ships are actuated from their control input, then physics steps.
---@param data EventData
function MiningTestbed:onSim(data)
    local advance = math.min(data:deltaTime(), MAX_ADVANCE_S)
    local steps = math.max(1, math.ceil(advance / SIM_STEP_S))
    for _ = 1, steps do
        local dt = advance / steps
        self.simSteps = self.simSteps + 1
        TravelDriveSystem:update(dt)
        ShipFlightSystem:update()
        self.world:update(dt)
    end
end

---@param data EventData
function MiningTestbed:onInput(data)
    if self:handleAppInput() then return end
    local dt = data:deltaTime()
    for _, action in pairs(Actions) do action:update(dt) end
    if Actions.Next:isPressed() then self:select(self.selected + 1) end
    if Actions.Prev:isPressed() then self:select(self.selected - 1) end
end

---@param data EventData
function MiningTestbed:onRender(data)
    CelestialLightingSystem:update(self.star)
    RenderCoreSystem:render(data)
    local lines = { string.format("Mining testbed: following Miner %d (Left/Right: another miner)", self.selected), self:debugLine() }
    for _, line in ipairs(EconomyOverlay()) do lines[#lines + 1] = line end
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, 40, 20,
                0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        end
    end)
end

return MiningTestbed
