-- A star system where ships fly the navigation orders on their own, for a person to watch: each
-- scenario is a group of ships on one order or behaviour (`NavigationDemo/Scenarios.lua`), in an
-- authored system (`NavigationDemo/Scene.lua`). Left and Right pick the scenario, Up and Down the
-- ship the orbit camera follows; the corner label tells what that ship is doing.
-- Design: doc/script/nav/guide.md

local Application             = require("States.Application")
local Registry                = require("Core.ECS.Registry")
local DrawEx                  = require("UI.DrawEx")
local Nav                     = require("Modules.Navigation")
local NavigationComponents    = require("Modules.Navigation.Components")
local PhysicsComponents       = require("Modules.Physics.Components")
local CelestialLightingSystem = require("Modules.Rendering.Systems.CelestialLightingSystem")
local RenderCoreSystem        = require("Modules.Rendering.Systems.RenderCoreSystem")
local ShipFlightSystem        = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem       = require("Modules.Constructs.Systems.TravelDriveSystem")
local GravityWellSystem       = require("Modules.Physics.Systems.GravityWellSystem")
local OrbitalSystem           = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
local _                       = require("Modules.Cameras.Systems.CameraSystem")
local Skybox                  = require("Shared.Generation.Skybox")
local ScaleConfig             = require("Config.Gen.UniverseScaleConfig")
local Actions                 = require("Input.ActionBindings.NavigationDemoActions")
local Scene                   = require("States.App.NavigationDemo.Scene")
local Scenarios               = require("States.App.NavigationDemo.Scenarios")

---@class NavigationDemoShip
---@field entity Entity
---@field scenario integer Index in `Scenarios`
---@field slot integer Index in the scenario's group
---@field step integer The ship's current step of the scenario: it counts the orders the ship has finished
---@field order Entity|nil The ship's current order
---@field failure string|nil Why the last order that failed did
---@field retryAt number|nil Simulated time the next order is given at, after a failure

---@class NavigationDemo: Application
local NavigationDemo = Subclass("NavigationDemo", Application)

local SEED = 20261122
-- The longest simulated step: a hitch of a frame would otherwise be one huge physics step.
local MAX_STEP_S = 0.05
-- A ship whose order failed is given the next one after this many simulated seconds.
local RETRY_DELAY_S = 5
local CAMERA_NAME = "NavigationDemo"
-- The camera starts and stays within this range of the ship, in game units; each wheel step is this
-- fraction of the distance.
local CAMERA_START, CAMERA_MIN, CAMERA_MAX = 0.06, 0.012, 5e5
local CAMERA_ZOOM_RATIO = 0.15
local CAMERA_PITCH = 0.4
local CAMERA_SENSITIVITY = 0.01
local EVENT_NAMES = { "NavOrderDone", "NavOrderFailed" }

local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 25
local HUD_W, HUD_H = 40, 20
local HUD_GREY, HUD_ALPHA = 0.9, 0.9
local HUD_ALIGN_X, HUD_ALIGN_Y = 0.0, 0.5

---The name of an enum's value.
---@param enum table<string, integer>
---@param value integer|nil
---@return string
local function nameOf(enum, value)
    for name, v in pairs(enum) do
        if v == value then return name end
    end
    return "-"
end

function NavigationDemo:onInit()
    self.simSeconds = 0
    self.simSteps = 0
    self.started = false
    self.orderShip = {}
    self.scenarioIndex = 1
    self.shipSlot = 1

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    self.scene = Scene.build(self.world)
    self.skybox = Skybox.Create(SEED)
    Nav.attach(self.world)
    -- After the world: the events exist once the navigation system does.
    for _, name in ipairs(EVENT_NAMES) do
        EventBus:subscribe(Event[name], nil, function(_, _, id) self:onOrderEnded(name, id) end)
    end

    self.context = {
        scene = self.scene,
        made = {},
        leaderOf = function(ship) return self:leaderOf(ship) end,
    }
    self:spawnShips()
    self:createCamera()
    self:selectShip(1, 1)
end

---Every scenario's ships, at their starts, in the physics world.
function NavigationDemo:spawnShips()
    local rng = RNG.Create(SEED)
    self.ships = {}
    self.groups = {}
    for index, scenario in ipairs(Scenarios) do
        local group = {}
        for slot = 1, scenario.count do
            local position = scenario.spawn(self.context, slot)
            local lookAt = Position(position.x + 1, position.y, position.z)
            local ship = {
                entity = Scene.spawnShip(self.world, rng:get31(), position, lookAt),
                scenario = index,
                slot = slot,
                step = 1,
            }
            group[slot] = ship
            table.insert(self.ships, ship)
        end
        self.groups[index] = group
    end
    Log.Info("NavigationDemo: %d ships in %d scenarios", #self.ships, #Scenarios)
end

function NavigationDemo:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, {
        initialPitch = CAMERA_PITCH, distance = CAMERA_START, minDistance = CAMERA_MIN, maxDistance = CAMERA_MAX,
        zoomRatio = CAMERA_ZOOM_RATIO, mouseSensitivity = CAMERA_SENSITIVITY,
    })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
end

---@param ship NavigationDemoShip
---@return NavigationDemoShip
function NavigationDemo:leaderOf(ship)
    local leaderSlot = assert(Scenarios[ship.scenario].leader, "NavigationDemo: the scenario has no leader")
    return self.groups[ship.scenario][leaderSlot]
end

---Gives the ship the order of its current step.
---@param ship NavigationDemoShip
function NavigationDemo:giveOrder(ship)
    local order = Nav.order(ship.entity, Scenarios[ship.scenario].order(self.context, ship))
    ship.order = order
    ship.retryAt = nil
    self.orderShip[order.id] = ship
end

---The ship's order has ended: it is released, and the ship goes on to its next step.
---@param name string The event
---@param id integer The order's entity id
function NavigationDemo:onOrderEnded(name, id)
    local ship = self.orderShip[id]
    if not ship then return end
    local order = Registry:getEntityById(id)
    self.orderShip[id] = nil
    ship.order = nil
    ship.step = ship.step + 1
    if name == "NavOrderFailed" then
        ship.failure = nameOf(Enums.NavOrderFailure, Nav.failure(order))
        Log.Warn("NavigationDemo: %s ship %d of '%s' failed an order: %s", name, ship.slot,
            Scenarios[ship.scenario].name, ship.failure)
        ship.retryAt = self.simSeconds + RETRY_DELAY_S
    else
        self:giveOrder(ship)
    end
    Nav.release(order)
end

---@param scenarioIndex integer
---@param slot integer
function NavigationDemo:selectShip(scenarioIndex, slot)
    local count = #Scenarios
    self.scenarioIndex = (scenarioIndex - 1) % count + 1
    self.shipSlot = (slot - 1) % Scenarios[self.scenarioIndex].count + 1
    self.camera:setTarget(self:selectedShip().entity)
end

---@return NavigationDemoShip
function NavigationDemo:selectedShip()
    return self.groups[self.scenarioIndex][self.shipSlot]
end

---The frame's start: the first orders go out once physics has stepped, and ships that failed an
---order retry it after a delay.
---@param data EventData
function NavigationDemo:onPreSim(data)
    if not self.started then
        -- A body that has just been made is in the query index that sensing and the planner read only
        -- after one physics update.
        if self.simSteps < 1 then return end
        self.started = true
        for _, ship in ipairs(self.ships) do self:giveOrder(ship) end
        return
    end
    for _, ship in ipairs(self.ships) do
        if ship.retryAt and self.simSeconds >= ship.retryAt then self:giveOrder(ship) end
    end
end

---Bodies move, then ships are actuated from the control input the orders wrote, then physics steps.
---@param data EventData
function NavigationDemo:onSim(data)
    local dt = math.min(data:deltaTime(), MAX_STEP_S)
    self.simSeconds = self.simSeconds + dt
    self.simSteps = self.simSteps + 1
    OrbitalSystem:update(self.scene.orbiters, dt, self.scene.followers)
    GravityWellSystem:update(dt, self.world:getSolverSubsteps())
    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    self.world:update(dt)
end

---@param data EventData
function NavigationDemo:onInput(data)
    if self:handleAppInput() then return end
    local dt = data:deltaTime()
    for _, action in pairs(Actions) do action:update(dt) end
    if Actions.NextScenario:isPressed() then self:selectShip(self.scenarioIndex + 1, 1) end
    if Actions.PrevScenario:isPressed() then self:selectShip(self.scenarioIndex - 1, 1) end
    if Actions.NextShip:isPressed() then self:selectShip(self.scenarioIndex, self.shipSlot + 1) end
    if Actions.PrevShip:isPressed() then self:selectShip(self.scenarioIndex, self.shipSlot - 1) end
end

---The label's lines: the scenario, the ship, its order and how far it has to go.
---@return string[]
function NavigationDemo:labelLines()
    local scenario = Scenarios[self.scenarioIndex]
    local ship = self:selectedShip()
    local role = scenario.leader == ship.slot and ", leader" or ""
    local lines = {
        string.format("Scenario %d/%d: %s", self.scenarioIndex, #Scenarios, scenario.name),
        scenario.orders,
        string.format("Ship %d/%d%s, order %d", ship.slot, scenario.count, role, ship.step),
    }
    local order = ship.order
    if order then
        local kind = nameOf(Enums.NavOrderKind, order:get(NavigationComponents.NavOrder):getKind())
        lines[#lines + 1] = string.format("%s: %s", kind, nameOf(Enums.NavOrderStatus, Nav.status(order)))
        local progress = Nav.progress(order)
        if progress.distanceLeft then
            -- A tracking order (Orbit, Follow, Hold) has no arrival: its progress is the distance from its
            -- circle or slot, and its ETA is 0.
            local eta = progress.eta and progress.eta > 0 and ", ETA " .. ScaleConfig:formatDuration(progress.eta) or ""
            lines[#lines + 1] = string.format("Distance %s%s", ScaleConfig:formatDistance(progress.distanceLeft), eta)
        end
    end
    if ship.failure then lines[#lines + 1] = "Last failure: " .. ship.failure end
    lines[#lines + 1] = "Left/Right: scenario   Up/Down: ship   Right mouse: turn   Wheel: zoom"
    return lines
end

---@param data EventData
function NavigationDemo:onRender(data)
    CelestialLightingSystem:update(self.scene.star)
    RenderCoreSystem:render(data)
    local lines = self:labelLines()
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, HUD_W, HUD_H,
                HUD_GREY, HUD_GREY, HUD_GREY, HUD_ALPHA, HUD_ALIGN_X, HUD_ALIGN_Y)
        end
    end)
end

return NavigationDemo
