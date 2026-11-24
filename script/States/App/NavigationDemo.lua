-- A star system where ships fly the navigation orders on their own, for a person to watch: each
-- scenario is a group of ships on one order or behaviour (`NavigationDemo/Scenarios.lua`), in an
-- authored system (`NavigationDemo/Scene.lua`). Left and Right pick the scenario, Up and Down the
-- ship the orbit camera follows; the corner label tells what that ship is doing. From time to time
-- two ships swap scenarios. M opens the 2D and the 3D system map, which show every ship's route.
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
local SystemMap               = require("Modules.CelestialObjects.Systems.SystemMap")
local SystemMap3D             = require("Modules.CelestialObjects.Systems.SystemMap3D")
local EntityPosition          = require("Shared.Helpers.EntityPosition")
local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
local _                       = require("Modules.Cameras.Systems.CameraSystem")
local Skybox                  = require("Shared.Generation.Skybox")
local ScaleConfig             = require("Config.Gen.UniverseScaleConfig")
local Actions                 = require("Input.ActionBindings.NavigationDemoActions")
local GeneralActions          = require("Input.ActionBindings.GeneralActions")
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
---@field joined boolean|nil Whether the ship has been swapped into its scenario (it did not start there)
---@field mapSpec table The ship's entry for the maps: `{ entity, label, color }`, kept current
---@field overlay table The ship's route for the maps: `{ shipPos, route, flown, color, ... }`, reused

---@class NavigationDemo: Application
---@field swapPeriodS number Simulated seconds between two swaps, on average
---@field swapJitterS number A swap comes this much sooner or later than the period, uniformly
---@field maxStepS number The most simulated time a frame advances, in seconds
---@field substepS number The longest single simulation step, in seconds
---@field mapMode integer One of `MapMode`
local NavigationDemo = Subclass("NavigationDemo", Application)

NavigationDemo.swapPeriodS = 30
NavigationDemo.swapJitterS = 10
-- The most a frame advances: a hitch of a frame would otherwise be one huge advance.
NavigationDemo.maxStepS = 0.05
-- A frame that advances more than this runs several equal steps, so that a sped-up run steps the
-- ships as finely as a real-time one.
NavigationDemo.substepS = 0.05

local SEED = 20261122
-- The swaps pick their ships and times from a stream of their own, so the scenarios' random points
-- don't depend on how many swaps have happened.
local SWAP_SEED = 20261123
-- A ship whose order failed is given the next one after this many simulated seconds.
local RETRY_DELAY_S = 5
local CAMERA_NAME = "NavigationDemo"
-- The camera starts and stays within this range of the ship, in game units; each wheel step is this
-- fraction of the distance.
local CAMERA_START, CAMERA_MIN, CAMERA_MAX = 0.06, 0.012, 5e5
local CAMERA_ZOOM_RATIO = 0.15
local CAMERA_PITCH = 0.4
local CAMERA_SENSITIVITY = 0.01
local EVENT_NAMES = { "NavOrderDone", "NavOrderFailed", "NavOrderCancelled" }

-- What the M key cycles through, in this order.
local MapMode = { Off = 0, Flat = 1, Space = 2 }
NavigationDemo.MapMode = MapMode
local MAP_MODE_COUNT = 3
-- A scenario's ships, and their routes on the maps, have a colour of its own: { r, g, b }.
local SCENARIO_COLORS = {
    { 1.0, 0.5, 0.2 }, { 0.3, 0.9, 0.4 }, { 1.0, 0.3, 0.3 }, { 0.8, 0.5, 1.0 }, { 0.3, 0.8, 1.0 },
    { 1.0, 0.9, 0.3 }, { 0.5, 1.0, 0.9 }, { 1.0, 0.5, 0.8 }, { 0.7, 0.7, 0.7 },
}
local ROUTE_ALPHA = 0.6

local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 25
local HUD_W, HUD_H = 40, 20
local HUD_GREY, HUD_ALPHA = 0.9, 0.9
local HUD_ALIGN_X, HUD_ALIGN_Y = 0.0, 0.5

---A uniformly chosen element of a non-empty list.
---@generic T
---@param rng RNG
---@param list T[]
---@return T
local function pick(rng, list)
    return list[math.min(#list, math.floor(rng:getUniformRange(0, #list)) + 1)]
end

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
    -- Orders cancelled by a swap, which are released when their event arrives.
    self.cancelled = {}
    self.swapCount = 0
    self.swapRng = RNG.Create(SWAP_SEED)
    self.nextSwapAt = 0
    self:scheduleSwap()
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
    self:createMaps()
    self:selectShip(1, 1)
end

---The two maps, closed, with an entry and a route for each ship.
function NavigationDemo:createMaps()
    assert(#SCENARIO_COLORS >= #Scenarios, "NavigationDemo: a colour for each scenario")
    self.mapMode = MapMode.Off
    self.mapState = SystemMap:create()
    self.map3DState = SystemMap3D:create()
    SystemMap3D:setSkybox(self.map3DState, self.skybox)
    self.routeColors = {}
    for index, c in ipairs(SCENARIO_COLORS) do self.routeColors[index] = Color(c[1], c[2], c[3], ROUTE_ALPHA) end
    self.shipByEntity = {}
    self.mapShips = {}
    self.otherRoutes = {}
    -- What the maps have selected, to tell a click on a ship from a selection the demo made.
    self.mapPicked = nil
    for _, ship in ipairs(self.ships) do
        ship.mapSpec = { entity = ship.entity }
        ship.overlay = { route = {}, flown = {} }
        self.shipByEntity[ship.entity.id] = ship
        table.insert(self.mapShips, ship.mapSpec)
        self:describeShip(ship)
    end
    self.mapState.ships = self.mapShips
    self.map3DState.ships = self.mapShips
end

---Sets the name and the colour a ship has on the maps from its scenario and slot.
---@param ship NavigationDemoShip
function NavigationDemo:describeShip(ship)
    ship.mapSpec.label = string.format("%s %d", Scenarios[ship.scenario].name, ship.slot)
    ship.mapSpec.color = SCENARIO_COLORS[ship.scenario]
    ship.overlay.color = self.routeColors[ship.scenario]
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

---An order has ended: a cancelled one is a swap's, and is only released; the ship of any other goes on
---to its next step.
---@param name string The event
---@param id integer The order's entity id
function NavigationDemo:onOrderEnded(name, id)
    if self.cancelled[id] then
        self.cancelled[id] = nil
        Nav.release(Registry:getEntityById(id))
        return
    end
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

---Sets the time of the next swap: the period, and or minus the jitter.
function NavigationDemo:scheduleSwap()
    local jitter = self.swapRng:getUniformRange(-self.swapJitterS, self.swapJitterS)
    self.nextSwapAt = self.simSeconds + self.swapPeriodS + jitter
end

---Takes the ship out of its scenario: its order is cancelled (its event releases it), or released at
---once when it has ended already.
---@param ship NavigationDemoShip
function NavigationDemo:leave(ship)
    local order = ship.order
    if order then
        self.orderShip[order.id] = nil
        if order:get(NavigationComponents.OrderStatus):hasEnded() then
            Nav.release(order)
        else
            self.cancelled[order.id] = true
            Nav.cancel(order)
        end
        ship.order = nil
    end
    ship.failure, ship.retryAt = nil, nil
end

---Two ships that are not leaders, from different scenarios, change scenarios: each leaves its orders,
---takes the other's slot and flies that scenario's first order from where it is. A leader stays, so
---the ships that follow or chase it keep a target. The ship the camera follows stays selected.
---@return NavigationDemoShip a
---@return NavigationDemoShip b
function NavigationDemo:swapShips()
    local movers = {}
    for _, ship in ipairs(self.ships) do
        if Scenarios[ship.scenario].leader ~= ship.slot then table.insert(movers, ship) end
    end
    local a = pick(self.swapRng, movers)
    local others = {}
    for _, ship in ipairs(movers) do
        if ship.scenario ~= a.scenario then table.insert(others, ship) end
    end
    local b = pick(self.swapRng, others)

    local selected = self:selectedShip()
    Log.Info("NavigationDemo: swap ship %d of '%s' with ship %d of '%s'", a.slot, Scenarios[a.scenario].name,
        b.slot, Scenarios[b.scenario].name)
    self:leave(a)
    self:leave(b)
    a.scenario, b.scenario = b.scenario, a.scenario
    a.slot, b.slot = b.slot, a.slot
    self.groups[a.scenario][a.slot] = a
    self.groups[b.scenario][b.slot] = b
    for _, ship in ipairs({ a, b }) do
        ship.step, ship.joined = 1, true
        self:describeShip(ship)
        self:giveOrder(ship)
    end
    self.swapCount = self.swapCount + 1
    -- The selection is a place in a scenario: it follows the ship when that one moved.
    if selected == a or selected == b then self:selectShip(selected.scenario, selected.slot) end
    return a, b
end

---@param scenarioIndex integer
---@param slot integer
function NavigationDemo:selectShip(scenarioIndex, slot)
    local count = #Scenarios
    self.scenarioIndex = (scenarioIndex - 1) % count + 1
    self.shipSlot = (slot - 1) % Scenarios[self.scenarioIndex].count + 1
    local entity = self:selectedShip().entity
    self.camera:setTarget(entity)
    self:showOnMap(entity)
end

---Selects `entity` in the open map.
---@param entity Entity
function NavigationDemo:showOnMap(entity)
    self.mapPicked = entity
    if self.mapMode == MapMode.Flat then
        SystemMap:selectEntity(self.mapState, entity)
    elseif self.mapMode == MapMode.Space then
        SystemMap3D:selectEntity(self.map3DState, entity)
    end
end

---Switches the maps: the flight view, the 2D map, the 3D map. A map has the mouse, so the orbit camera
---gives it up while one is open.
---@param mode integer One of `MapMode`
function NavigationDemo:setMapMode(mode)
    if self.mapMode == MapMode.Space then SystemMap3D:deactivate(self.map3DState) end
    self.mapMode = mode
    self.mapState.enabled = mode == MapMode.Flat
    self.map3DState.enabled = mode == MapMode.Space
    if mode == MapMode.Off then self.camera:enable() else self.camera:disable() end
    if mode == MapMode.Flat then
        -- The list of what the map shows is built once: ask for a new one.
        self.mapState._entityListBuilt = false
        self.mapState._dotCache = nil
        SystemMap:collectEntities(self.mapState)
    elseif mode == MapMode.Space then
        SystemMap3D:collectEntities(self.map3DState)
        SystemMap3D:activate(self.map3DState)
    end
    self:showOnMap(self:selectedShip().entity)
end

---The open map and its state, or nil.
---@return table|nil
function NavigationDemo:openMap()
    if self.mapMode == MapMode.Flat then return self.mapState end
    if self.mapMode == MapMode.Space then return self.map3DState end
    return nil
end

---A click on a ship in the open map points the camera at it.
function NavigationDemo:followMapSelection()
    local state = self:openMap()
    local entity = state and state.selected and state.selected.entity
    if entity == self.mapPicked then return end
    self.mapPicked = entity
    local ship = entity and self.shipByEntity[entity.id]
    if ship then self:selectShip(ship.scenario, ship.slot) end
end

---@return NavigationDemoShip
function NavigationDemo:selectedShip()
    return self.groups[self.scenarioIndex][self.shipSlot]
end

---The frame's start: the first orders go out once physics has stepped, and ships that failed an
---order retry it after a delay.
---@param data EventData
function NavigationDemo:onPreSim(data)
    SystemMap3D:updateTrails(self.map3DState, data:deltaTime())
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
    if self.simSeconds >= self.nextSwapAt then
        self:swapShips()
        self:scheduleSwap()
    end
end

---Bodies move, then ships are actuated from the control input the orders wrote, then physics steps.
---@param data EventData
function NavigationDemo:onSim(data)
    local advance = math.min(data:deltaTime(), self.maxStepS)
    local steps = math.max(1, math.ceil(advance / self.substepS))
    for _ = 1, steps do self:simStep(advance / steps) end
end

---One simulation step of `dt` seconds.
---@param dt number
function NavigationDemo:simStep(dt)
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

    GeneralActions.CycleMapMode:update(dt)
    if GeneralActions.CycleMapMode:isPressed() then self:setMapMode((self.mapMode + 1) % MAP_MODE_COUNT) end
    if self.mapMode == MapMode.Flat then
        SystemMap:updateInput(self.mapState, dt)
    elseif self.mapMode == MapMode.Space then
        SystemMap3D:updateInput(self.map3DState, dt)
    end
    self:followMapSelection()
end

---@param list any[]
local function clear(list)
    for i = #list, 1, -1 do list[i] = nil end
end

---Fills every ship's route for the maps from its order. The selected ship's has the goal and the
---progress too, and goes to the maps' own overlay; the others' are drawn as lines.
function NavigationDemo:updateRoutes()
    local selected = self:selectedShip()
    clear(self.otherRoutes)
    for _, ship in ipairs(self.ships) do
        local overlay, order = ship.overlay, ship.order
        overlay.shipPos = EntityPosition.Get(ship.entity)
        if order then
            overlay.route = Nav.route(order, overlay.route)
            overlay.flown = Nav.routeFlown(order, overlay.flown)
        else
            clear(overlay.route)
            clear(overlay.flown)
        end
        if ship ~= selected then table.insert(self.otherRoutes, overlay) end
    end

    local overlay, order = selected.overlay, selected.order
    local route = overlay.route
    overlay.targetPos = #route > 0 and route[#route] or nil
    local progress = order and Nav.progress(order)
    overlay.progress = progress and progress.distanceLeft and progress.eta and progress or nil
    overlay.planning = order ~= nil and Nav.status(order) == Enums.NavOrderStatus.Planning
    for _, state in ipairs({ self.mapState, self.map3DState }) do
        state.routeOverlay = overlay
        state.extraRoutes = self.otherRoutes
    end
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
    if self.started then
        lines[#lines + 1] = string.format("Next swap in %d s", math.max(0, math.ceil(self.nextSwapAt - self.simSeconds)))
    end
    lines[#lines + 1] = "Left/Right: scenario   Up/Down: ship   M: map   Right mouse: turn   Wheel: zoom"
    return lines
end

---@param data EventData
function NavigationDemo:onRender(data)
    CelestialLightingSystem:update(self.scene.star)
    if self.mapMode == MapMode.Space then
        SystemMap3D:collectEntities(self.map3DState)
        self:updateRoutes()
        SystemMap3D:render(self.map3DState, data)
    else
        RenderCoreSystem:render(data)
        if self.mapMode == MapMode.Flat then
            SystemMap:collectEntities(self.mapState)
            self:updateRoutes()
        end
    end
    local lines = self:labelLines()
    self:immediateUI(function()
        if self.mapMode == MapMode.Space then
            SystemMap3D:renderOverlay(self.map3DState, 0, 0, Window:width(), Window:height())
        elseif self.mapMode == MapMode.Flat then
            SystemMap:draw(self.mapState, 0, 0, Window:width(), Window:height())
        end
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, HUD_W, HUD_H,
                HUD_GREY, HUD_GREY, HUD_GREY, HUD_ALPHA, HUD_ALIGN_X, HUD_ALIGN_Y)
        end
    end)
end

return NavigationDemo
