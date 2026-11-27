-- Runs the economy demo (`States/App/EconomyDemo.lua`) on its own and checks its work:
--   * every ship docks at a station (a ship without a job docks to wait), and ships leave their ports;
--   * a trade run buys at one station and sells at another; ore that was mined is sold;
--   * every station's market trades, and every station's production finishes cycles;
--   * credits are kept: the stations' and ships' balances and the credits held in open bids add up
--     to the start's plus the income the colonies were paid.
-- The run ends when the work has been seen, not after a time; a simulated-time limit turns a hang into
-- a failure. Logs each failed check and ends with a failure exit code.

local Registry          = require("Core.ECS.Registry")
local EconomyDemo       = require("States.App.EconomyDemo")
local Economy           = require("Modules.Economy.Components")
local AccountManager    = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")
local DockingSystem     = require("Modules.Docking.Systems.DockingSystem")
local PeopleManager     = require("Modules.People.Managers.PeopleManager")
local People            = require("Modules.People.Components")

---@class EconomyDemoTest: EconomyDemo
local DemoTest = Subclass("EconomyDemoTest", EconomyDemo)

-- A wait that never ends becomes a failure after this many seconds of simulated time.
local MAX_WAIT_S = 1500
-- Simulated seconds per second of the clock.
local TIME_SCALE = 8
DemoTest.maxStepS = TIME_SCALE / 20
-- Credits may be off by this much in all, for the rounding of trade prices.
local CREDIT_TOLERANCE = 0

function DemoTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        -- Warn, not Error: Error raises, and a run should list every failed check.
        Log.Warn("EconomyDemoTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---Waits, a frame at a time, until `done()` holds.
---@param done fun(): boolean
---@param what string Named in the failure when the wait runs out
---@param detail (fun(): string)|nil Tells, when the wait runs out, what it was waiting on
---@return boolean reached
function DemoTest:waitUntil(done, what, detail)
    local start = MarketplaceSystem:now()
    while not done() do
        if MarketplaceSystem:now() - start > MAX_WAIT_S then
            self:check(false, what, "ran out of time" .. (detail and ("; " .. detail()) or ""))
            return false
        end
        coroutine.yield()
    end
    Log.Info("EconomyDemoTest: %s at %.0f s", what, MarketplaceSystem:now())
    return true
end

---Every credit in the economy: the balances of the stations and of every person (a ship's account is
---its owner-captain's wallet; its crew's wallets hold their pay), and the credits open bids hold.
---@return number
function DemoTest:totalCredits()
    local total = 0
    for _, station in ipairs(self.scene.stations) do total = total + AccountManager:balanceOf(station.entity) end
    for person in Registry:view(People.Person) do total = total + AccountManager:balanceOf(person) end
    for _, market in Registry:view(Economy.Marketplace) do
        for _, bid in ipairs(market:getBids()) do
            if Registry:hasEntity(bid) then
                total = total + bid:get(Economy.Quantity):getQuantity() * bid:get(Economy.Price):getPrice()
            end
        end
    end
    return total
end

---The credits the stations' production has paid them, all told.
---@return number
function DemoTest:incomePaid()
    local paid = 0
    for _, station in ipairs(self.scene.stations) do
        for _, line in ipairs(station.entity:get(Economy.Production):getLines()) do
            paid = paid + line.cycles * (line.recipe.income or 0)
        end
    end
    return paid
end

function DemoTest:onInit()
    self.passed, self.failed = 0, 0
    EventBus:setTimeScale(TIME_SCALE)
    EconomyDemo.onInit(self)
    self.docked, self.undocked = {}, {}
    self.cases = coroutine.create(function() self:runCases() end)
end

---Notes the ships that have docked, and those that have left a port since.
function DemoTest:noteDockings()
    for _, ship in ipairs(self.ships) do
        local id = ship.entity.id
        local docked = DockingSystem:isDocked(ship.entity)
        if docked then self.docked[id] = true end
        if self.docked[id] and not docked and DockingSystem:phaseOf(ship.entity) == nil then self.undocked[id] = true end
    end
end

---@param jobKind string "trade" or "mining"
---@return boolean
function DemoTest:someoneEarned(jobKind)
    for _, ship in ipairs(self.ships) do
        local agent = PeopleManager.commandOf(ship.entity, Economy.EconomyAgent)
        if agent and (agent.earnedBy[jobKind] or 0) > 0 then return true end
    end
    return false
end

---The ships not in `set`, with what each is doing.
---@param set table<integer, boolean>
---@return string
function DemoTest:shipsMissing(set)
    local parts = {}
    for _, ship in ipairs(self.ships) do
        if not set[ship.entity.id] then
            local agent = PeopleManager.commandOf(ship.entity, Economy.EconomyAgent)
            local phase = DockingSystem:phaseOf(ship.entity)
            parts[#parts + 1] = string.format("%s (%s, state %s, docking phase %s)", ship.name,
                agent and agent:getActivity() or "no agent", agent and tostring(agent:getState()) or "-", tostring(phase))
        end
    end
    return table.concat(parts, "; ")
end

---@return boolean
function DemoTest:everyShip(set)
    for _, ship in ipairs(self.ships) do
        if not set[ship.entity.id] then return false end
    end
    return true
end

---@return string|nil missing The first station whose market hasn't traded or whose lines haven't cycled
function DemoTest:idleStation()
    for _, station in ipairs(self.scene.stations) do
        local market = station.entity:get(Economy.Marketplace)
        local traded = false
        for _ in pairs(market.history) do traded = true end
        local cycled = false
        for _, line in ipairs(station.entity:get(Economy.Production):getLines()) do
            if line.cycles > 0 then cycled = true end
        end
        if not traded or not cycled then return station.name end
    end
    return nil
end

function DemoTest:runCases()
    self:waitUntil(function() return self.started end, "the ships become agents")
    -- The income paid before now (the long first frames run whole cycles) is in the start's credits.
    self.startCredits, self.startIncome = self:totalCredits(), self:incomePaid()
    self:waitUntil(function() return next(self.docked) ~= nil end, "a ship docks")
    self:waitUntil(function() return self:someoneEarned("trade") end, "a trade run sells")
    self:waitUntil(function() return self:someoneEarned("mining") end, "mined ore sells")
    self:waitUntil(function() return self:idleStation() == nil end, "every station trades and produces")
    self:waitUntil(function() return self:everyShip(self.docked) end, "every ship docks",
        function() return self:shipsMissing(self.docked) end)
    self:waitUntil(function() return next(self.undocked) ~= nil end, "a ship leaves a port")

    local expected = self.startCredits + self:incomePaid() - self.startIncome
    local total = self:totalCredits()
    self:check(math.abs(total - expected) <= CREDIT_TOLERANCE, "credits are kept",
        string.format("%.0f in the economy, %.0f expected", total, expected))
    local idle = self:idleStation()
    self:check(idle == nil, "every station trades and produces", idle)

    for _, ship in ipairs(self.ships) do
        local agent = PeopleManager.commandOf(ship.entity, Economy.EconomyAgent)
        local trips, earned, spent = agent:getStats()
        Log.Info("EconomyDemoTest: %s: %d trips, earned %.0f, spent %.0f, balance %.0f", ship.name, trips, earned,
            spent, AccountManager:balanceOf(ship.entity))
    end
    Log.Info("EconomyDemoTest: %.0f s of simulated time, %d checks passed, %d failed", MarketplaceSystem:now(),
        self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

---@param data EventData
function DemoTest:onPreSim(data)
    EconomyDemo.onPreSim(self, data)
    self:noteDockings()
    local ok, err = coroutine.resume(self.cases)
    if not ok then
        Log.Warn("EconomyDemoTest: %s", tostring(err))
        self:quit(Core.ExitCode.Failure)
    end
end

---The app's input handling sets the clock to real time each frame: the run takes it back after.
---@param data EventData
function DemoTest:onInput(data)
    EconomyDemo.onInput(self, data)
    EventBus:setTimeScale(TIME_SCALE)
end

return DemoTest
