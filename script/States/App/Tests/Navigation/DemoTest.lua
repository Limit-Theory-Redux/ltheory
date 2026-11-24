-- Runs the navigation demo (`States/App/NavigationDemo.lua`) on its own and checks its work:
--   * every ship's order begins; every ship of the scenarios that finish orders finishes one, and the
--     bodies the "Bodies" scenario flies to (moons, station, planet, rock) are each reached;
--   * ships swap scenarios, leaders stay, and a swapped ship's new order begins;
--   * the 2D and the 3D map show every ship and its route, and picking a ship in a map selects it;
--   * across the whole run, no order fails and no ship's bounding sphere touches a body, a rock or another
--     ship's (the navigation sandbox's collision rule).
-- The run ends when the work has been seen, not after a time; a simulated-time limit turns a hang into a
-- failure. Logs each failed check and ends with a failure exit code.

local Registry           = require("Core.ECS.Registry")
local Nav                = require("Modules.Navigation")
local PhysicsComponents  = require("Modules.Physics.Components")
local NavigationDemo     = require("States.App.NavigationDemo")
local Scenarios          = require("States.App.NavigationDemo.Scenarios")
local Scene              = require("States.App.NavigationDemo.Scene")
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
local FINISHING = { "Far crossing", "Bodies", "Traffic" }
-- Two bounding spheres may overlap by this share of the ship's radius before it is a contact: the
-- avoidance steers by the spheres and a ship can graze a hull inside its clearance.
local OVERLAP_TOLERANCE = 0.75
-- The first few of a kind of violation are told in detail.
local DETAILS_SHOWN = 20

---@param gameUnits gameunit
---@return number
local function metres(gameUnits) return gameUnits / Scene.toGame(1) end

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
    -- The ships that have finished an order, and the ids of the entities a done MoveTo reached.
    self.finishedShips = {}
    self.reached = {}
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

---The bodies and rocks no ship may touch, and the ships.
function DemoTest:collectSolids()
    local scene = self.scene
    local bodies = { { name = "the star", entity = scene.star }, { name = "the station", entity = scene.station } }
    for name, planet in pairs(scene.planets) do table.insert(bodies, { name = name, entity = planet }) end
    for name, moon in pairs(scene.moons) do table.insert(bodies, { name = name, entity = moon }) end
    self.solids = {}
    for _, body in ipairs(bodies) do
        local rb = body.entity:get(PhysicsComponents.RigidBody):getRigidBody()
        table.insert(self.solids, { name = body.name, rb = rb, radius = rb:getBoundingRadius() })
    end
    for i, rock in ipairs(scene.rocks) do
        table.insert(self.solids, { name = "rock " .. i, rb = rock.rb, radius = rock.radius })
    end
    self.bodies = {}
    for i, ship in ipairs(self.ships) do
        local rb = ship.entity:get(PhysicsComponents.RigidBody):getRigidBody()
        self.bodies[i] = { rb = rb, radius = rb:getBoundingRadius() }
    end
    -- The pairs in contact, each counted once while it lasts: a key per (ship, solid) and per (ship, ship).
    self.touching, self.grazing, self.grazes = {}, {}, 0
    self.shipPairBase = #self.bodies * #self.solids
end

---Counts a new contact when a pair's bounding spheres overlap by more than `OVERLAP_TOLERANCE` of the
---ship's radius; a shallower overlap is a graze, counted and not failed. A pair is counted once while
---it stays in contact.
---@param key integer The pair's key
---@param depth gameunit How far the spheres overlap; negative when apart
---@param shipRadius gameunit
---@param kind string "solid" or "ship"
---@param describe fun(): string What the log says of the pair
function DemoTest:noteContact(key, depth, shipRadius, kind, describe)
    local touching, grazing = self.touching, self.grazing
    if depth <= 0 then
        grazing[key] = nil
    elseif not grazing[key] then
        grazing[key] = true
        self.grazes = self.grazes + 1
    end
    if depth <= OVERLAP_TOLERANCE * shipRadius then
        touching[key] = nil
    elseif not touching[key] then
        touching[key] = true
        self.violations[kind] = self.violations[kind] + 1
        if self.violations[kind] <= DETAILS_SHOWN then
            Log.Warn("NavigationDemoTest: %s at %.1f s", describe(), self.simSeconds)
        end
    end
end

---Counts the ships whose bounding sphere starts to touch a body, a rock or another ship, now.
function DemoTest:sampleSeparation()
    local bodies, solids = self.bodies, self.solids
    for i, ship in ipairs(bodies) do
        local at = ship.rb:getPos()
        for k, solid in ipairs(solids) do
            local distance, reach = at:distance(solid.rb:getPos()), solid.radius + ship.radius
            self:noteContact((i - 1) * #solids + k, reach - distance, ship.radius, "solid", function()
                return string.format("%s touches %s: %.0f m apart, spheres reach %.0f m, ship at %.0f m/s",
                    self:shipLabel(i), solid.name, metres(distance), metres(reach), metres(ship.rb:getVelocity():length()))
            end)
        end
        for j = i + 1, #bodies do
            local other = bodies[j]
            local distance, reach = at:distance(other.rb:getPos()), ship.radius + other.radius
            self:noteContact(self.shipPairBase + (i - 1) * #bodies + j, reach - distance, ship.radius, "ship", function()
                return string.format("%s and %s touch: %.0f m apart, spheres reach %.0f m", self:shipLabel(i),
                    self:shipLabel(j), metres(distance), metres(reach))
            end)
        end
    end
end

---@param index integer Index in `self.ships`
---@return string The ship's scenario and slot now
function DemoTest:shipLabel(index)
    local ship = self.ships[index]
    return string.format("ship %d (%s %d)", index, Scenarios[ship.scenario].name, ship.slot)
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
        self.finishedShips[ship] = true
        local moveTo = Registry:getEntityById(id):get(Nav.Components.MoveToOrder)
        if moveTo and moveTo:getTargetEntityId() then self.reached[moveTo:getTargetEntityId()] = true end
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

---What the scenarios that end an order still have to show: a ship of each that has not finished one,
---and a body of "Bodies" that no ship has reached. Meaningful before the first swap, while a slot
---holds its first ship.
---@return string[]
function DemoTest:missingFinishes()
    local missing = {}
    for _, name in ipairs(FINISHING) do
        local index = scenarioIndex(name)
        for slot, ship in ipairs(self.groups[index]) do
            if not self.finishedShips[ship] then missing[#missing + 1] = string.format("%s ship %d", name, slot) end
        end
        local targets = Scenarios[index].targets
        for _, target in ipairs(targets and targets(self.context) or {}) do
            if not self.reached[target.id] then missing[#missing + 1] = string.format("%s target %d", name, target.id) end
        end
    end
    return missing
end

---@return boolean
function DemoTest:everyFinishingScenarioDone()
    return #self:missingFinishes() == 0
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
    if not self:waitUntil(function() return self:everyFinishingScenarioDone() end,
        "every ship of the scenarios that end an order finishes one, and every body is reached") then
        Log.Warn("NavigationDemoTest: still missing: %s", table.concat(self:missingFinishes(), ", "))
    end
    reached("every ship of the scenarios that end an order has finished one, and every body is reached")
    self:caseSwaps()
    reached("the swaps are seen")
    self:caseMap(MapMode.Flat, "2D map: ", SystemMap)
    self:caseMap(MapMode.Space, "3D map: ", SystemMap3D)
    self:check(self.violations.solid == 0, "no ship overlaps a body or a rock", tostring(self.violations.solid))
    self:check(self.violations.ship == 0, "no two ships overlap", tostring(self.violations.ship))
    Log.Info("NavigationDemoTest: %.0f s of simulated time, %d swaps, %d grazes, %d checks passed, %d failed",
        self.simSeconds, self.swapCount, self.grazes, self.passed, self.failed)
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

---Every step is checked, so that a touch inside a frame's several steps is seen.
---@param dt number
function DemoTest:simStep(dt)
    NavigationDemo.simStep(self, dt)
    self:sampleSeparation()
end

---@param data EventData
function DemoTest:onRender(data)
    NavigationDemo.onRender(self, data)
    if self.mapFrames[self.mapMode] then self.mapFrames[self.mapMode] = self.mapFrames[self.mapMode] + 1 end
end

return DemoTest
