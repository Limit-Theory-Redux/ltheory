-- Runs the navigation demo (`States/App/NavigationDemo.lua`) on its own and checks its work:
--   * every ship's order begins, and each scenario that finishes an order finishes one;
--   * ships swap scenarios, leaders stay, and a swapped ship's new order begins;
--   * the 2D and the 3D map show every ship and its route, and picking a ship in a map selects it;
--   * across the whole run, no order fails and no ship is inside a body, a rock or another ship.
-- The run ends when the work has been seen, not after a time; a simulated-time limit turns a hang into a
-- failure. Logs each failed check and ends with a failure exit code.

local Nav                = require("Modules.Navigation")
local PhysicsComponents  = require("Modules.Physics.Components")
local NavigationDemo     = require("States.App.NavigationDemo")
local Scenarios          = require("States.App.NavigationDemo.Scenarios")
local SystemMap          = require("Modules.CelestialObjects.Systems.SystemMap")
local SystemMap3D        = require("Modules.CelestialObjects.Systems.SystemMap3D")

---@class NavigationDemoTest: NavigationDemo
local DemoTest = Subclass("NavigationDemoTest", NavigationDemo)

-- A wait that never ends becomes a failure after this many seconds of simulated time.
local MAX_WAIT_S = 900
-- Simulated seconds per second of the clock: the flights take minutes of simulated time.
local TIME_SCALE = 8
-- The swaps come often, to see several in one run.
DemoTest.swapPeriodS = 6
DemoTest.swapJitterS = 2
DemoTest.maxStepS = TIME_SCALE / 20
-- The run ends once this many swaps have happened and their ships have begun their new orders.
local SWAPS_SEEN = 2
-- The scenarios whose orders end, by name: each must finish one.
local FINISHING = { "Far crossing", "Moons and the station", "Traffic" }
-- Two ships are overlapping when their centres are closer than this fraction of their radii added.
local OVERLAP_FRACTION = 0.5
-- The first few of a kind of violation are told in detail.
local DETAILS_SHOWN = 3

local Status = Enums.NavOrderStatus
local MapMode = NavigationDemo.MapMode

function DemoTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("NavigationDemoTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---Waits, a frame at a time, until `done()` holds.
---@param done fun(): boolean
---@param what string Named in the failure when the wait runs out
---@return boolean reached
function DemoTest:waitUntil(done, what)
    local start = self.simSeconds
    while not done() do
        if self.simSeconds - start > MAX_WAIT_S then
            self:check(false, what, "ran out of time")
            return false
        end
        coroutine.yield()
    end
    return true
end

---@param name string
---@return integer The scenario's index
local function scenarioIndex(name)
    for index, scenario in ipairs(Scenarios) do
        if scenario.name == name then return index end
    end
    error("NavigationDemoTest: no scenario named " .. name)
end

function DemoTest:onInit()
    self.passed, self.failed = 0, 0
    EventBus:setTimeScale(TIME_SCALE)
    NavigationDemo.onInit(self)
    -- The swaps wait for their part of the run.
    self.nextSwapAt = math.huge
    self.finished = {}
    self.awaiting = {}
    self.mapFrames = { [MapMode.Flat] = 0, [MapMode.Space] = 0 }
    self.violations = { solid = 0, ship = 0 }
    self.leaders = {}
    for index, scenario in ipairs(Scenarios) do
        if scenario.leader then self.leaders[index] = self.groups[index][scenario.leader].entity end
    end
    self:collectSolids()
    self.cases = coroutine.create(function() self:runCases() end)
end

---The bodies and rocks no ship may be inside.
function DemoTest:collectSolids()
    local scene = self.scene
    local bodies = { scene.star, scene.station }
    for _, planet in pairs(scene.planets) do table.insert(bodies, planet) end
    for _, moon in pairs(scene.moons) do table.insert(bodies, moon) end
    self.solids = {}
    for _, body in ipairs(bodies) do
        local rb = body:get(PhysicsComponents.RigidBody):getRigidBody()
        table.insert(self.solids, { rb = rb, radius = rb:getBoundingRadius() })
    end
    for _, rock in ipairs(scene.rocks) do table.insert(self.solids, { rb = rock.rb, radius = rock.radius }) end
    self.bodies = {}
    for i, ship in ipairs(self.ships) do
        local rb = ship.entity:get(PhysicsComponents.RigidBody):getRigidBody()
        self.bodies[i] = { rb = rb, radius = rb:getBoundingRadius() }
    end
end

---Counts the ships that are inside a body or a rock, or overlapping another ship, now.
function DemoTest:sampleSeparation()
    local bodies, violations = self.bodies, self.violations
    for i, ship in ipairs(bodies) do
        local at = ship.rb:getPos()
        for _, solid in ipairs(self.solids) do
            if at:distance(solid.rb:getPos()) < solid.radius then
                violations.solid = violations.solid + 1
                if violations.solid <= DETAILS_SHOWN then
                    Log.Warn("NavigationDemoTest: ship %d is inside a body at %.1f s", i, self.simSeconds)
                end
            end
        end
        for j = i + 1, #bodies do
            local other = bodies[j]
            if at:distance(other.rb:getPos()) < (ship.radius + other.radius) * OVERLAP_FRACTION then
                violations.ship = violations.ship + 1
                if violations.ship <= DETAILS_SHOWN then
                    Log.Warn("NavigationDemoTest: ships %d and %d overlap at %.1f s", i, j, self.simSeconds)
                end
            end
        end
    end
end

---A leader never moves between scenarios: the ship in its slot is the one that started there.
function DemoTest:leadersStay()
    for index, entity in pairs(self.leaders) do
        if self.groups[index][Scenarios[index].leader].entity ~= entity then return false end
    end
    return true
end

---An order that ends is told; a failure fails the run.
---@param name string
---@param id integer
function DemoTest:onOrderEnded(name, id)
    local ship = self.orderShip[id]
    if ship and name == "NavOrderDone" then
        self.finished[ship.scenario] = true
        -- An order that starts within reach of its goal is done in its first update: it began, though
        -- no frame saw it `Active`.
        ship.began = true
        self.awaiting[ship] = nil
    end
    if ship and name == "NavOrderFailed" then
        self:check(false, "no order fails", string.format("ship %d of '%s'", ship.slot, Scenarios[ship.scenario].name))
    end
    NavigationDemo.onOrderEnded(self, name, id)
end

---@return NavigationDemoShip a
---@return NavigationDemoShip b
function DemoTest:swapShips()
    local a, b = NavigationDemo.swapShips(self)
    self.awaiting[a], self.awaiting[b] = true, true
    return a, b
end

---Every ship whose order was given since a swap, and every ship once, has begun it: the order is `Active`.
function DemoTest:noteBegun()
    for _, ship in ipairs(self.ships) do
        local order = ship.order
        if order and Nav.status(order) == Status.Active then
            ship.began = true
            self.awaiting[ship] = nil
        end
    end
end

---@return boolean
function DemoTest:everyShipBegan()
    for _, ship in ipairs(self.ships) do
        if not ship.began then return false end
    end
    return true
end

---@return boolean
function DemoTest:everyFinishingScenarioDone()
    for _, name in ipairs(FINISHING) do
        if not self.finished[scenarioIndex(name)] then return false end
    end
    return true
end

---The ships that are shown in the open map: how many of its entries are ships.
---@return integer
function DemoTest:shipsOnMap()
    local count = 0
    for _, entry in ipairs(self:openMap().entities) do
        if self.shipByEntity[entry.entity.id] then count = count + 1 end
    end
    return count
end

---Opens a map, waits for it to draw, and checks that it shows every ship and route, and that a ship
---picked in it becomes the selected one.
---@param mode integer
---@param name string
---@param map table SystemMap or SystemMap3D
function DemoTest:caseMap(mode, name, map)
    self:setMapMode(mode)
    self:waitUntil(function() return self.mapFrames[mode] >= 2 end, name .. "the map draws")
    local state = self:openMap()
    self:check(self:shipsOnMap() == #self.ships, name .. "every ship is on the map", tostring(self:shipsOnMap()))
    self:check(#state.extraRoutes == #self.ships - 1 and state.routeOverlay == self:selectedShip().overlay,
        name .. "every ship has a route, the selected one with its goal")
    local withRoute = 0
    for _, ship in ipairs(self.ships) do
        if #ship.overlay.route > 0 then withRoute = withRoute + 1 end
    end
    self:check(withRoute > 0, name .. "ships are flying routes", tostring(withRoute))

    local selected = self:selectedShip()
    local other = self.ships[1] == selected and self.ships[2] or self.ships[1]
    map:selectEntity(state, other.entity)
    self:waitUntil(function() return self:selectedShip() == other end, name .. "a ship picked in the map is selected")
    self:setMapMode(MapMode.Off)
end

---The swaps start, and the run waits until `SWAPS_SEEN` have happened and their ships have begun.
function DemoTest:caseSwaps()
    local name = "swaps: "
    local leadersMoved = false
    self:scheduleSwap()
    self:waitUntil(function()
        leadersMoved = leadersMoved or not self:leadersStay()
        return self.swapCount >= SWAPS_SEEN and next(self.awaiting) == nil
    end, name .. "ships swap and begin their new orders")
    self:check(not leadersMoved, name .. "a leader stays in its scenario")
    self:check(self.swapCount >= SWAPS_SEEN, name .. "ships swapped", tostring(self.swapCount))
end

---Runs the cases one after the other; the frames of the app resume it.
function DemoTest:runCases()
    local function reached(what) Log.Info("NavigationDemoTest: %s at %.0f s", what, self.simSeconds) end
    self:waitUntil(function() return self:everyShipBegan() end, "every ship's order begins")
    reached("every ship's order has begun")
    self:waitUntil(function() return self:everyFinishingScenarioDone() end,
        "every scenario that ends an order finishes one")
    reached("every scenario that ends an order has finished one")
    self:caseSwaps()
    reached("the swaps are seen")
    self:caseMap(MapMode.Flat, "2D map: ", SystemMap)
    self:caseMap(MapMode.Space, "3D map: ", SystemMap3D)
    self:check(self.violations.solid == 0, "no ship is inside a body or a rock", tostring(self.violations.solid))
    self:check(self.violations.ship == 0, "no two ships overlap", tostring(self.violations.ship))
    Log.Info("NavigationDemoTest: %.0f s of simulated time, %d swaps, %d checks passed, %d failed",
        self.simSeconds, self.swapCount, self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

---The frame's start: the demo's, then the case code, after the events of the last frame have arrived.
---@param data EventData
function DemoTest:onPreSim(data)
    NavigationDemo.onPreSim(self, data)
    self:noteBegun()
    local ok, err = coroutine.resume(self.cases)
    if not ok then Log.Error("NavigationDemoTest: %s", tostring(err)) end
end

---The app's input handling sets the clock to real time each frame: the run takes it back after.
---@param data EventData
function DemoTest:onInput(data)
    NavigationDemo.onInput(self, data)
    EventBus:setTimeScale(TIME_SCALE)
end

---@param data EventData
function DemoTest:onSim(data)
    NavigationDemo.onSim(self, data)
    self:sampleSeparation()
end

---@param data EventData
function DemoTest:onRender(data)
    NavigationDemo.onRender(self, data)
    if self.mapFrames[self.mapMode] then self.mapFrames[self.mapMode] = self.mapFrames[self.mapMode] + 1 end
end

return DemoTest
