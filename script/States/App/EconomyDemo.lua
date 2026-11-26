-- A star system where the economy runs on its own, for a person to watch: stations produce and
-- consume goods and keep orders on their markets, traders carry goods between them, and miners take
-- iron ore from a field of rocks and sell it. Ships fly with navigation orders and dock at the
-- stations' ports. Left and Right pick the station or ship the orbit camera follows; the corner text
-- tells what it holds and does, and the state of every station and ship.
-- Design: doc/script/economy.md

local Application             = require("States.Application")
local Registry                = require("Core.ECS.Registry")
local DrawEx                  = require("UI.DrawEx")
local Nav                     = require("Modules.Navigation")
local NameComponent           = require("Modules.Core.Components.NameComponent")
local CelestialLightingSystem = require("Modules.Rendering.Systems.CelestialLightingSystem")
local RenderCoreSystem        = require("Modules.Rendering.Systems.RenderCoreSystem")
local ShipFlightSystem        = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem       = require("Modules.Constructs.Systems.TravelDriveSystem")
local GravityWellSystem       = require("Modules.Physics.Systems.GravityWellSystem")
local EntityPosition          = require("Shared.Helpers.EntityPosition")
local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
local _                       = require("Modules.Cameras.Systems.CameraSystem")
local Skybox                  = require("Shared.Generation.Skybox")
local Items                   = require("Shared.Registries.Items")
local Economy                 = require("Modules.Economy.Components")
local PhysicsComponents       = require("Modules.Physics.Components")
local EconomySetup            = require("Modules.Economy.Managers.EconomySetup")
local InventoryManager        = require("Modules.Economy.Managers.InventoryManager")
local AccountManager          = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem       = require("Modules.Economy.Systems.MarketplaceSystem")
-- Required for their event subscriptions: they run the economy and the dockings at PreSim.
local _                       = require("Modules.Economy.Systems")
local DockingSystem           = require("Modules.Docking.Systems.DockingSystem")
local Actions                 = require("Input.ActionBindings.EconomyDemoActions")
local Scene                   = require("States.App.EconomyDemo.Scene")

---@class EconomyDemo: Application
---@field maxStepS number The most simulated time a frame advances, in seconds
---@field substepS number The longest single simulation step, in seconds
local EconomyDemo = Subclass("EconomyDemo", Application)

EconomyDemo.maxStepS = 0.05
EconomyDemo.substepS = 0.05

local SEED = 20261126
local CAMERA_NAME = "EconomyDemo"
-- The camera starts and stays within this range of what it follows, in game units, and never closer
-- than a few of its radii: a station is far larger than a ship.
local CAMERA_START, CAMERA_MIN, CAMERA_MAX = 0.06, 0.012, 5e5
local CAMERA_START_RADII, CAMERA_MIN_RADII = 4, 1.5
local CAMERA_ZOOM_RATIO = 0.15
local CAMERA_PITCH = 0.4
local CAMERA_SENSITIVITY = 0.01

-- The economic agents: how many of each role, their hold's mass capacity and their starting credits.
local AGENTS = {
    { role = Enums.EconomyAgentRole.Trader, label = "Trader", count = 4, capacity = 600, credits = 8000 },
    { role = Enums.EconomyAgentRole.Miner, label = "Miner", count = 3, capacity = 480, credits = 500 },
}
-- The agents start around the colony, this far from it and from each other, in metres.
local SPAWN_DISTANCE_M, SPAWN_SPACING_M = 30e3, 2e3

local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 22
local HUD_W, HUD_H = 40, 20
local HUD_GREY, HUD_ALPHA = 0.9, 0.9
local HUD_ALIGN_X, HUD_ALIGN_Y = 0.0, 0.5

function EconomyDemo:onInit()
    self.simSeconds = 0
    self.simSteps = 0
    self.started = false

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    self.scene = Scene.build(self.world)
    self.skybox = Skybox.Create(SEED)
    Nav.attach(self.world)
    self:spawnShips()
    self:createCamera()

    -- What Left and Right step through: the stations, then the ships.
    self.selectable = {}
    for _, station in ipairs(self.scene.stations) do self.selectable[#self.selectable + 1] = station.entity end
    for _, ship in ipairs(self.ships) do self.selectable[#self.selectable + 1] = ship.entity end
    self:select(1)
end

---The agents' ships around the colony, in the physics world. They become agents once physics has
---stepped (see `onPreSim`).
function EconomyDemo:spawnShips()
    local rng = RNG.Create(SEED)
    local colony = EntityPosition.Get(self.scene.stations[1].entity)
    self.ships = {}
    local index = 0
    for _, group in ipairs(AGENTS) do
        for n = 1, group.count do
            index = index + 1
            local angle = index * 360 / 7
            local position = Scene.around(colony, angle, SPAWN_DISTANCE_M + index * SPAWN_SPACING_M)
            local entity = Scene.spawnShip(self.world, rng:get31(), position, colony)
            self.ships[#self.ships + 1] = { entity = entity, group = group, name = string.format("%s %d", group.label, n) }
        end
    end
    Log.Info("EconomyDemo: %d stations, %d ships, %d ore rocks", #self.scene.stations, #self.ships, #self.scene.oreRocks)
end

function EconomyDemo:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, {
        initialPitch = CAMERA_PITCH, distance = CAMERA_START, minDistance = CAMERA_MIN, maxDistance = CAMERA_MAX,
        zoomRatio = CAMERA_ZOOM_RATIO, mouseSensitivity = CAMERA_SENSITIVITY,
    })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
end

---@param index integer Into `selectable`, wrapping
function EconomyDemo:select(index)
    self.selected = (index - 1) % #self.selectable + 1
    local entity = self.selectable[self.selected]
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local radius = rbCmp and rbCmp:getRigidBody() and rbCmp:getRigidBody():getBoundingRadius() or 0
    self.camera:setDistances({
        distance = math.max(CAMERA_START, radius * CAMERA_START_RADII),
        minDistance = math.max(CAMERA_MIN, radius * CAMERA_MIN_RADII),
        maxDistance = CAMERA_MAX,
        zoomRatio = CAMERA_ZOOM_RATIO,
    })
    self.camera:setTarget(entity)
end

---The frame's start: the ships become agents once physics has stepped, since a body that has just
---been made is in the query index that navigation reads only after one physics update.
---@param data EventData
function EconomyDemo:onPreSim(data)
    if self.started or self.simSteps < 1 then return end
    self.started = true
    for _, ship in ipairs(self.ships) do
        EconomySetup.equipAgent(ship.entity, {
            role = ship.group.role, capacity = ship.group.capacity, credits = ship.group.credits, name = ship.name,
        })
    end
end

---Ships are actuated from the control input the orders wrote, then physics steps.
---@param data EventData
function EconomyDemo:onSim(data)
    local advance = math.min(data:deltaTime(), self.maxStepS)
    local steps = math.max(1, math.ceil(advance / self.substepS))
    for _ = 1, steps do self:simStep(advance / steps) end
end

---@param dt number
function EconomyDemo:simStep(dt)
    self.simSeconds = self.simSeconds + dt
    self.simSteps = self.simSteps + 1
    GravityWellSystem:update(dt, self.world:getSolverSubsteps())
    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    self.world:update(dt)
end

---@param data EventData
function EconomyDemo:onInput(data)
    if self:handleAppInput() then return end
    local dt = data:deltaTime()
    for _, action in pairs(Actions) do action:update(dt) end
    if Actions.Next:isPressed() then self:select(self.selected + 1) end
    if Actions.Prev:isPressed() then self:select(self.selected - 1) end
end

---@param entity Entity
---@return string
local function nameOf(entity)
    local name = entity:get(NameComponent)
    return name and name:getName() or tostring(entity)
end

---@param itemId integer
---@return string
local function itemName(itemId)
    return Items:getDefinition(itemId).name
end

---"Item count" for each item the inventory holds, in item order.
---@param inventory InventoryComponent
---@return string
local function stockText(inventory)
    local ids = {}
    for itemId in pairs(inventory:getInventory()) do ids[#ids + 1] = itemId end
    table.sort(ids)
    local parts = {}
    for _, itemId in ipairs(ids) do
        parts[#parts + 1] = string.format("%s %d", itemName(itemId), InventoryManager:count(inventory, itemId))
    end
    return #parts > 0 and table.concat(parts, ", ") or "empty"
end

---@param station Entity
---@return string[]
function EconomyDemo:stationDetail(station)
    local lines = {}
    local production = station:get(Economy.Production)
    local inputs, outputs = production:getFlows()
    local items = {}
    for itemId in pairs(inputs) do items[#items + 1] = itemId end
    for itemId in pairs(outputs) do items[#items + 1] = itemId end
    table.sort(items)
    for _, itemId in ipairs(items) do
        local quote = MarketplaceSystem:quote(station, itemId)
        local bid = quote.bestBid and string.format("bid %d x%d", quote.bestBid, quote.demand) or "no bid"
        local ask = quote.bestAsk and string.format("ask %d x%d", quote.bestAsk, quote.supply) or "no ask"
        local market = station:get(Economy.Marketplace)
        local traded = #market:getHistory(itemId)
        lines[#lines + 1] = string.format("  %s: %s, %s, %d trades", itemName(itemId), bid, ask, traded)
    end
    for i, line in ipairs(production:getLines()) do
        local state = line.running and string.format("%d%%", math.floor(100 * line.progress / line.recipe.duration)) or "waiting for inputs"
        lines[#lines + 1] = string.format("  Line %d %s: %s, %d cycles", i, line.recipe.name, state, line.cycles)
    end
    local free, total = DockingSystem:countPorts(station)
    lines[#lines + 1] = string.format("  Ports: %d of %d free", free, total)
    return lines
end

---@return string[]
function EconomyDemo:hudLines()
    local lines = {
        string.format("Economy demo: %.0f s simulated (hold the time acceleration key to speed up)", MarketplaceSystem:now()),
    }
    local entity = self.selectable[self.selected]
    lines[#lines + 1] = "Following: " .. nameOf(entity)
    if entity:get(Economy.Production) then
        for _, line in ipairs(self:stationDetail(entity)) do lines[#lines + 1] = line end
    end
    lines[#lines + 1] = ""
    lines[#lines + 1] = "Stations"
    for _, station in ipairs(self.scene.stations) do
        local e = station.entity
        lines[#lines + 1] = string.format("  %s: %d cr | %s", station.name, AccountManager:balanceOf(e),
            stockText(e:get(Economy.Inventory)))
    end
    lines[#lines + 1] = "Ships"
    for _, ship in ipairs(self.ships) do
        local agent = ship.entity:get(Economy.EconomyAgent)
        if agent then
            local trips = agent:getStats()
            lines[#lines + 1] = string.format("  %s: %s | %d cr | %d trips | %s", ship.name, agent:getActivity(),
                AccountManager:balanceOf(ship.entity), trips, stockText(ship.entity:get(Economy.Inventory)))
        end
    end
    lines[#lines + 1] = ""
    lines[#lines + 1] = "Left/Right: station or ship   Right mouse: turn   Wheel: zoom"
    return lines
end

---@param data EventData
function EconomyDemo:onRender(data)
    CelestialLightingSystem:update(self.scene.star)
    RenderCoreSystem:render(data)
    local lines = self:hudLines()
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, HUD_W, HUD_H,
                HUD_GREY, HUD_GREY, HUD_GREY, HUD_ALPHA, HUD_ALIGN_X, HUD_ALIGN_Y)
        end
    end)
end

return EconomyDemo
