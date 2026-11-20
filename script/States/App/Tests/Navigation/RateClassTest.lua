-- Checks the update-rate classes of navigation: a ship inside a zone with a rate class has it (the
-- highest of several), dense traffic gives High, open space gives Low, and a body in range gives
-- Normal; the class is judged again at once when a zone is entered or left and otherwise once a
-- period, and the replan period follows the class. Logs each failed check and ends with a failure
-- exit code.

local Application       = require("States.Application")
local Registry          = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")
local SpatialComponents = require("Modules.Spatial.Components")
local SpatialEntities   = require("Modules.Spatial.Entities")
local Nav               = require("Modules.Navigation")
local NavConfig         = require("Config.Navigation.NavConfig")
local NavRateSystem     = require("Modules.Navigation.Systems.NavRateSystem")
local FlightScene       = require("States.App.Tests.Support.FlightScene")

---@class NavigationRateClassTest: Application
local RateClassTest = Subclass("NavigationRateClassTest", Application)

local Class      = Enums.NavRateClass
local Components = Nav.Components

local STEP = 1 / 60
local BODY_RADIUS = 2
-- Where in a range a body is put to be inside it, and the share of a radius a zone has beyond a ship.
local INSIDE_SHARE = 0.5
local ZONE_RADIUS  = 10
-- The longest a judging is waited for, in judging periods: a safety limit only.
local MAX_PERIODS = 3

function RateClassTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("NavigationRateClassTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---One update of the order system and of the physics index.
function RateClassTest:step()
    Nav.update(STEP)
    self.world:update(STEP)
end

---@param ship { entity: Entity }
---@return NavRateClassId|nil
local function classOf(ship)
    local rate = ship.entity:get(Components.NavRate)
    return rate and rate.class
end

---A ship with a Hold order, which has an agent in the navigation world from the first update.
---@param at Position
---@return { entity: Entity, rb: RigidBody }, Entity order
function RateClassTest:holder(at)
    local ship = FlightScene.spawnShip(self.world, at)
    self.world:update(STEP)
    local order = Nav.order(ship.entity, Nav.Orders.Hold())
    return ship, order
end

---@param ship { entity: Entity }
---@return number range How far the ship's avoidance senses
function RateClassTest:sensingRange(ship)
    for _, world in Registry:view(Components.NavWorld) do
        return world:getNav():sensingRange(ship.entity:get(Components.NavAgent):getSlot())
    end
    error("RateClassTest: no navigation world")
end

---Steps until the ship has been judged once more (its judging time moves), which is at most a period
---away. Afterwards the next judging is a whole period away.
---@param ship { entity: Entity }
function RateClassTest:untilJudged(ship)
    local rate = ship.entity:get(Components.NavRate)
    local due = rate and rate.nextJudgeAt
    local limit = math.ceil(MAX_PERIODS * NavConfig.rateJudgePeriodS / STEP)
    for _ = 1, limit do
        self:step()
        rate = ship.entity:get(Components.NavRate)
        if rate and rate.nextJudgeAt ~= due then return end
    end
    self:check(false, "a ship is judged again within a few periods")
end

---Ends a case: its orders are cancelled and released, and its things removed.
---@param orders Entity[]
---@param things { entity: Entity, rb: RigidBody }[]
function RateClassTest:finish(orders, things)
    for _, order in ipairs(orders) do Nav.cancel(order) end
    self:step()
    for _, order in ipairs(orders) do Nav.release(order) end
    FlightScene.despawn(self.world, things)
    self.world:update(STEP)
end

---@param at Position
---@param radius gameunit
---@return Entity
local function sphereZone(at, radius)
    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Sphere)
    zone:get(SpatialComponents.Shape):setRadius(radius)
    zone:get(PhysicsComponents.Transform):setPos(at)
    return zone
end

function RateClassTest:caseOpenSpace()
    local name = "open space: "
    local ship, order = self:holder(Position(0, 0, 0))
    self:step()
    self:check(classOf(ship) == Class.Low, name .. "a ship with nothing near is Low", tostring(classOf(ship)))
    self:check(NavRateSystem.replanPeriodS(ship.entity) == NavConfig.rateClasses[Class.Low].replanPeriodS,
        name .. "its replan period is Low's")
    self:finish({ order }, { ship })
    self:check(ship.entity:get(Components.NavRate) == nil, name .. "a ship that is no agent has no class")
end

---A body in range makes a ship Normal, once the ship is judged again, and not before.
function RateClassTest:caseBodyInRange()
    local name = "a body in range: "
    local ship, order = self:holder(Position(0, 0, 0))
    self:step()
    self:untilJudged(ship)
    local range = self:sensingRange(ship)
    local body = FlightScene.spawnBody(self.world, Position(range * INSIDE_SHARE, 0, 0), BODY_RADIUS)
    self.world:update(STEP)
    self:step()
    self:check(classOf(ship) == Class.Low, name .. "the class is kept until the ship is due", tostring(classOf(ship)))
    self:untilJudged(ship)
    self:check(classOf(ship) == Class.Normal, name .. "a ship with a body near is Normal", tostring(classOf(ship)))
    self:check(NavRateSystem.replanPeriodS(ship.entity) == NavConfig.rateClasses[Class.Normal].replanPeriodS,
        name .. "its replan period is Normal's")
    FlightScene.despawn(self.world, { body })
    self:finish({ order }, { ship })
end

---`count` ships around a ship, each at `distance` along an axis, and their orders.
---@param count integer
---@param distance gameunit
---@return { entity: Entity, rb: RigidBody }[] ships
---@return Entity[] orders
function RateClassTest:crowd(count, distance)
    local axes = { Position(distance, 0, 0), Position(0, distance, 0), Position(0, 0, distance),
        Position(-distance, 0, 0), Position(0, -distance, 0) }
    local ships, orders = {}, {}
    for i = 1, count do
        ships[i], orders[i] = self:holder(axes[i])
    end
    return ships, orders
end

function RateClassTest:caseTraffic()
    local name = "traffic: "
    local radius = NavConfig.trafficRadius()
    local ship, order = self:holder(Position(0, 0, 0))
    local enough, enoughOrders = self:crowd(NavConfig.trafficShips, radius * INSIDE_SHARE)
    self:step()
    self:check(classOf(ship) == Class.High, name .. "enough ships near give High", tostring(classOf(ship)))
    self:check(NavRateSystem.replanPeriodS(ship.entity) == NavConfig.rateClasses[Class.High].replanPeriodS,
        name .. "its replan period is High's")
    self:finish(enoughOrders, enough)

    local few, fewOrders = self:crowd(NavConfig.trafficShips - 1, radius * INSIDE_SHARE)
    self:untilJudged(ship)
    self:check(classOf(ship) ~= Class.High, name .. "fewer ships near do not", tostring(classOf(ship)))
    self:finish(fewOrders, few)

    -- A body that is not a ship is not traffic, however many.
    local bodies = {}
    for i = 1, NavConfig.trafficShips do
        bodies[i] = FlightScene.spawnBody(self.world, Position(0, 0, radius * INSIDE_SHARE * i / NavConfig.trafficShips),
            BODY_RADIUS)
    end
    self.world:update(STEP)
    self:untilJudged(ship)
    self:check(classOf(ship) ~= Class.High, name .. "bodies that are not ships do not", tostring(classOf(ship)))
    FlightScene.despawn(self.world, bodies)
    self:finish({ order }, { ship })
end

---A zone gives its class to a ship inside it; the highest of several wins; entering or leaving judges
---at once.
function RateClassTest:caseZone()
    local name = "a rate zone: "
    local ship, order = self:holder(Position(0, 0, 0))
    local zone = sphereZone(Position(0, 0, 0), ZONE_RADIUS)
    zone:add(Components.NavRateClass(Class.High))
    self:step()
    self:check(classOf(ship) == Class.High, name .. "a ship inside has the zone's class", tostring(classOf(ship)))

    local second = sphereZone(Position(0, 0, 0), ZONE_RADIUS)
    second:add(Components.NavRateClass(Class.Low))
    self:untilJudged(ship)
    self:check(classOf(ship) == Class.High, name .. "the highest rate of the zones wins", tostring(classOf(ship)))
    Registry:destroyEntity(second)

    -- The ship leaves: the next update judges it, though it was judged a moment ago.
    local rate = ship.entity:get(Components.NavRate)
    local due = rate.nextJudgeAt
    ship.rb:setPos(Position(ZONE_RADIUS * 4, 0, 0))
    self.world:update(STEP)
    Nav.update(STEP)
    self:check(rate.nextJudgeAt ~= due, name .. "leaving the zone judges the ship at once")
    self:check(classOf(ship) == Class.Low, name .. "a ship outside it is in open space", tostring(classOf(ship)))

    -- And enters again.
    ship.rb:setPos(Position(0, 0, 0))
    self.world:update(STEP)
    Nav.update(STEP)
    self:check(classOf(ship) == Class.High, name .. "entering the zone judges the ship at once", tostring(classOf(ship)))

    Registry:destroyEntity(zone)
    self:finish({ order }, { ship })
end

function RateClassTest:onInit()
    self.passed, self.failed = 0, 0
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)
    Nav.attach(self.world)

    self:caseOpenSpace()
    self:caseBodyInRange()
    self:caseTraffic()
    self:caseZone()

    Log.Info("NavigationRateClassTest: %d checks passed, %d failed", self.passed, self.failed)
    EventBus:subscribe(Event.PostInput, self, function()
        self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
    end)
end

return RateClassTest
