local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Items = require("Shared.Registries.Items")
local ApproachSpec = require("Shared.Types.ApproachSpec")
local EntityPosition = require("Shared.Helpers.EntityPosition")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local FlightProfile = require("Modules.Constructs.Components.FlightProfileComponent")
local DockingSystem = require("Modules.Docking.Systems.DockingSystem")
local MiningSystem = require("Modules.Economy.Systems.MiningSystem")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")
local NameComponent = require("Modules.Core.Components.NameComponent")
local DockingComponents = require("Modules.Docking.Components")

-- Design: doc/script/economy.md

local State = Enums.EconomyAgentState
local Phase = Enums.DockingPhase

-- The states of a leg: flying, or docking at the end of a flight.
local LEG_STATES = { [State.DockingToBuy] = true, [State.DockingToSell] = true, [State.FlyingToMine] = true }

-- The economy clock's time this update, for the states' start times.
local currentNow = 0

-- The claims of this update: a job's claims count at once, so that agents planning in the same
-- update don't take the same lot or crowd one rock.
local liveClaims = {}

---Runs the economic agents (`EconomyAgentComponent`): ships that pick a job from the markets and
---fly it. A trader buys an item at the station that asks least for it and sells it to one that
---bids more; a miner flies to an asteroid that yields an item some station bids for, fills its hold
---and sells there. A ship with cargo first sells it where it is bid for best. Jobs are scored by
---profit over the time they take, from navigation estimates of the legs; the quantities a job
---plans to buy, sell or mine are held against the other agents, so two agents don't chase one lot.
---The flights are navigation orders and dockings (`DockingSystem`); trades happen docked, at once
---(`MarketplaceSystem:buyNow`/`sellNow`), after a service time.
---@class EconomyAgentSystem
local EconomyAgentSystem = Class("EconomyAgentSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function EconomyAgentSystem:registerVars()
    self.profiler = QuickProfiler("EconomyAgentSystem", false, false)
    self.rng = RNG.FromTime()
    -- Seconds docked before a trade is done, and before an agent without a job looks again.
    self.serviceS = 6
    self.idleRetryS = 8
    -- The least profit a job is taken for, in credits.
    self.minProfit = 100
    -- How many of the most profitable jobs are timed with navigation estimates.
    self.estimateCandidates = 4
    -- Seconds added to a job's time for each docking.
    self.dockOverheadS = 40
    -- A miner's run lasts at most `maxMiningS`; it flies to `mineGapM` off the rock's hull first,
    -- and the mining run's glide takes it onto its circle from there.
    self.maxMiningS = 600
    self.mineGapM = 1500
    self.maxMinersPerAsteroid = 2
    -- A trade may pay this much over the planned buy price, or take this much under the planned sell price.
    self.priceSlack = 0.1
    -- A flight or docking leg that hasn't ended after this many seconds is given up, with its job.
    self.maxLegS = 900
    -- A miner whose run took nothing leaves that rock alone for this many seconds.
    self.badRockS = 300
    ---@type Entity[] Navigation orders cancelled and not yet ended, released once they have
    self.cancelled = {}
end

function EconomyAgentSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function EconomyAgentSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---@param id integer|nil
---@return Entity|nil
local function alive(id)
    if not id then return nil end
    local entity = Entity(id)
    return Registry:hasEntity(entity) and entity or nil
end

---@param dt number Seconds
function EconomyAgentSystem:update(dt)
    self.profiler:start()
    self:releaseCancelled()
    local agents = {}
    for ship, agent in Registry:view(Economy.EconomyAgent) do agents[#agents + 1] = { ship, agent } end
    self:collectClaims(agents)
    local now = MarketplaceSystem:now()
    currentNow = now
    for _, entry in ipairs(agents) do
        local ship, agent = entry[1], entry[2]
        if ship:get(PhysicsComponents.RigidBody) and ship:get(Economy.Inventory) then
            self:step(ship, agent, now, dt)
        end
    end
    self.profiler:stop()
end

-- Claims -----------------------------------------------------------------------------------------

---@param kind string "ask", "bid" or "rock"
---@param entityId integer
---@param itemId integer
---@return string
local function claimKey(kind, entityId, itemId)
    return kind .. ":" .. entityId .. ":" .. itemId
end

---Sums what the agents' jobs hold, so that a job is planned on what the others leave.
---@param agents table[]
function EconomyAgentSystem:collectClaims(agents)
    self.claims = {}
    liveClaims = self.claims
    for _, entry in ipairs(agents) do
        local job = entry[2].job
        for _, claim in ipairs(job and job.claims or {}) do
            self.claims[claim.key] = (self.claims[claim.key] or 0) + claim.quantity
        end
    end
end

---@param key string
---@return integer
function EconomyAgentSystem:claimed(key)
    return self.claims and self.claims[key] or 0
end

---@param job EconomyJob
---@param key string
---@param quantity integer
local function claim(job, key, quantity)
    job.claims[#job.claims + 1] = { key = key, quantity = quantity }
    liveClaims[key] = (liveClaims[key] or 0) + quantity
end

---@param job EconomyJob
---@param prefix string Drops the claims whose key starts with it
local function dropClaims(job, prefix)
    local kept = {}
    for _, c in ipairs(job.claims) do
        if c.key:sub(1, #prefix) ~= prefix then kept[#kept + 1] = c end
    end
    job.claims = kept
end

-- Orders -----------------------------------------------------------------------------------------

function EconomyAgentSystem:releaseCancelled()
    local waiting = {}
    for _, order in ipairs(self.cancelled) do
        if Registry:hasEntity(order) then
            if order:get(Nav.Components.OrderStatus):hasEnded() then Nav.release(order) else waiting[#waiting + 1] = order end
        end
    end
    self.cancelled = waiting
end

---Ends the agent's navigation order: released when it has ended, cancelled (and released later) when not.
---@param agent EconomyAgentComponent
function EconomyAgentSystem:dropOrder(agent)
    local order = agent.order
    agent.order = nil
    if not order or not Registry:hasEntity(order) then return end
    if order:get(Nav.Components.OrderStatus):hasEnded() then
        Nav.release(order)
    else
        Nav.cancel(order)
        self.cancelled[#self.cancelled + 1] = order
    end
end

-- Estimates --------------------------------------------------------------------------------------

---A point just outside an entity, from which a flight onward can be estimated (its centre is
---inside its hull).
---@param entity Entity
---@return Position
local function departurePoint(entity)
    local pos = EntityPosition.Get(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local radius = rbCmp and rbCmp:getRigidBody() and rbCmp:getRigidBody():getBoundingRadius() or 0
    return Position(pos.x, pos.y + radius * 2, pos.z)
end

---Seconds the flight takes, from the navigation estimate, or from the straight distance at top
---speed when the estimate finds no way.
---@param ship Entity
---@param from Entity|Position
---@param to Entity
---@return number
function EconomyAgentSystem:flightTime(ship, from, to)
    local profile = FlightProfile.ProfileOf(ship)
    local estimate = Nav.estimate(from, to, profile)
    if estimate and estimate.reachable and estimate.time then return estimate.time end
    local a = rawtype(from) == "table" and EntityPosition.Get(from) or from
    local b = EntityPosition.Get(to)
    local dx, dy, dz = b.x - a.x, b.y - a.y, b.z - a.z
    return math.sqrt(dx * dx + dy * dy + dz * dz) / math.max(profile.maxSpeed, 1e-6)
end

-- Planning ---------------------------------------------------------------------------------------

---@return Entity[] stations The entities with a market and a body
local function markets()
    local stations = {}
    for entity in Registry:view(Economy.Marketplace) do
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then stations[#stations + 1] = entity end
    end
    return stations
end

---@param ship Entity
---@param agent EconomyAgentComponent
---@param inventory InventoryComponent
---@return EconomyJob|nil
function EconomyAgentSystem:planSale(ship, agent, inventory)
    local best, bestScore = nil, 0
    for itemId in pairs(inventory:getInventory()) do
        local held = InventoryManager:count(inventory, itemId)
        if held > 0 then
            for _, station in ipairs(markets()) do
                local quote = MarketplaceSystem:quote(station, itemId, ship)
                local open = quote.demand - self:claimed(claimKey("bid", station.id, itemId))
                if quote.bestBid and open > 0 then
                    local quantity = math.min(held, open)
                    local score = quote.bestBid * quantity / (self:flightTime(ship, ship, station) + self.dockOverheadS)
                    if score > bestScore then
                        best, bestScore = {
                            itemId = itemId, quantity = quantity, destination = station.id,
                            sellPrice = quote.bestBid, claims = {},
                        }, score
                    end
                end
            end
        end
    end
    if best then claim(best, claimKey("bid", best.destination, best.itemId), best.quantity) end
    return best
end

---Times the most profitable candidates and returns the best by profit per second.
---@param candidates table[] `{ profit, job, time = fun(): number }`
---@return EconomyJob|nil
function EconomyAgentSystem:bestCandidate(candidates)
    table.sort(candidates, function(a, b) return a.profit > b.profit end)
    local best, bestScore = nil, 0
    for i = 1, math.min(#candidates, self.estimateCandidates) do
        local candidate = candidates[i]
        local score = candidate.profit / math.max(1, candidate.time())
        if score > bestScore then best, bestScore = candidate.job, score end
    end
    return best
end

---@param ship Entity
---@param inventory InventoryComponent
---@return EconomyJob|nil
function EconomyAgentSystem:planTrade(ship, inventory)
    local stations = markets()
    local balance = AccountManager:balanceOf(ship)
    local candidates = {}
    for _, source in ipairs(stations) do
        local asked = {}
        for _, ask in ipairs(source:get(Economy.Marketplace):getAsks()) do
            if Registry:hasEntity(ask) then asked[ask:get(Economy.Item):getItem()] = true end
        end
        for itemId in pairs(asked) do
            local sourceQuote = MarketplaceSystem:quote(source, itemId, ship)
            local supply = (sourceQuote.bestAsk and sourceQuote.askQuantity or 0)
                - self:claimed(claimKey("ask", source.id, itemId))
            if supply > 0 then
                for _, destination in ipairs(stations) do
                    if destination.id ~= source.id then
                        local quote = MarketplaceSystem:quote(destination, itemId, ship)
                        local demand = quote.demand - self:claimed(claimKey("bid", destination.id, itemId))
                        if quote.bestBid and quote.bestBid > sourceQuote.bestAsk and demand > 0 then
                            local quantity = math.min(supply, demand, InventoryManager:getRoomFor(inventory, itemId),
                                math.floor(balance / sourceQuote.bestAsk))
                            local profit = (quote.bestBid - sourceQuote.bestAsk) * quantity
                            if quantity > 0 and profit >= self.minProfit then
                                candidates[#candidates + 1] = {
                                    profit = profit,
                                    job = {
                                        itemId = itemId, quantity = quantity, source = source.id,
                                        destination = destination.id, buyPrice = sourceQuote.bestAsk,
                                        sellPrice = quote.bestBid, claims = {},
                                    },
                                    time = function()
                                        return self:flightTime(ship, ship, source)
                                            + self:flightTime(ship, departurePoint(source), destination)
                                            + 2 * self.dockOverheadS
                                    end,
                                }
                            end
                        end
                    end
                end
            end
        end
    end
    local job = self:bestCandidate(candidates)
    if job then
        claim(job, claimKey("ask", job.source, job.itemId), job.quantity)
        claim(job, claimKey("bid", job.destination, job.itemId), job.quantity)
    end
    return job
end

---@param ship Entity
---@param inventory InventoryComponent
---@return EconomyJob|nil
function EconomyAgentSystem:planMining(ship, agent, inventory)
    local stations = markets()
    local candidates = {}
    for asteroid, yield, item, quantity in Registry:iterEntities(Economy.Yield, Economy.Item, Economy.Quantity) do
        local itemId = item:getItem()
        local rockKey = claimKey("rock", asteroid.id, itemId)
        local left = quantity:getQuantity() - self:claimed(rockKey)
        local miners = self.claims and self.claims["miners:" .. asteroid.id] or 0
        local avoidUntil = agent.badRocks and agent.badRocks[asteroid.id]
        if left > 0 and miners < self.maxMinersPerAsteroid and not (avoidUntil and currentNow < avoidUntil) then
            for _, station in ipairs(stations) do
                local quote = MarketplaceSystem:quote(station, itemId, ship)
                local demand = quote.demand - self:claimed(claimKey("bid", station.id, itemId))
                if quote.bestBid and demand > 0 then
                    local amount = math.min(left, demand, InventoryManager:getRoomFor(inventory, itemId))
                    local value = quote.bestBid * amount
                    if amount > 0 and value >= self.minProfit then
                        candidates[#candidates + 1] = {
                            profit = value,
                            job = {
                                itemId = itemId, quantity = amount, source = asteroid.id,
                                destination = station.id, sellPrice = quote.bestBid, claims = {},
                            },
                            time = function()
                                return self:flightTime(ship, ship, asteroid)
                                    + self:flightTime(ship, departurePoint(asteroid), station)
                                    + amount / MiningSystem.rate + self.dockOverheadS
                            end,
                        }
                    end
                end
            end
        end
    end
    local job = self:bestCandidate(candidates)
    if job then
        claim(job, claimKey("rock", job.source, job.itemId), job.quantity)
        claim(job, "miners:" .. job.source, 1)
        claim(job, claimKey("bid", job.destination, job.itemId), job.quantity)
    end
    return job
end

---A job without trade for an agent that has none: dock at the nearest market with a free port, to
---wait there.
---@param ship Entity
---@return EconomyJob|nil
function EconomyAgentSystem:planPark(ship)
    local best, bestTime = nil, math.huge
    for _, station in ipairs(markets()) do
        if DockingSystem:countPorts(station) > 0 then
            local time = self:flightTime(ship, ship, station)
            if time < bestTime then best, bestTime = station, time end
        end
    end
    return best and { stage = "park", destination = best.id, claims = {} } or nil
end

-- Steps ------------------------------------------------------------------------------------------

---@param agent EconomyAgentComponent
---@param state EconomyAgentState
---@param activity string
local function enter(agent, state, activity)
    if agent.state ~= state then agent.stateSince = currentNow end
    agent.state = state
    agent.activity = activity
end

---@param itemId integer
---@return string
local function itemName(itemId)
    local definition = Items:getDefinition(itemId)
    return definition and definition.name or tostring(itemId)
end

---@param entityId integer|nil
---@return string
local function nameOf(entityId)
    local entity = alive(entityId)
    if not entity then return "?" end
    local name = entity:get(NameComponent)
    return name and name:getName() or tostring(entity)
end

---@param agent EconomyAgentComponent
---@param now number
---@param seconds number
---@param resume boolean Whether the job goes on after the wait
function EconomyAgentSystem:wait(agent, now, seconds, resume)
    agent.waitUntil = now + seconds * self.rng:getUniformRange(0.75, 1.25)
    agent.resume = resume
    enter(agent, State.Waiting, resume and agent.activity or "looking for work")
end

---Ends the job, whatever is left of it; the agent looks for the next.
---@param agent EconomyAgentComponent
function EconomyAgentSystem:endJob(agent)
    self:dropOrder(agent)
    agent.job = nil
    agent.resume = false
    enter(agent, State.Idle, "idle")
end

---Starts the leg the job is at: docking at the station to buy or sell at, or the flight to the
---asteroid. A ship docked elsewhere leaves its port first.
---@param ship Entity
---@param agent EconomyAgentComponent
---@param now number
function EconomyAgentSystem:startStage(ship, agent, now)
    local job = agent.job
    if not job then return self:endJob(agent) end
    local stage = job.stage
    local target = (stage == "sell" or stage == "park") and job.destination or job.source
    local targetEntity = alive(target)
    if not targetEntity then return self:endJob(agent) end

    local phase = DockingSystem:phaseOf(ship)
    if phase == Phase.Failed then
        DockingSystem:release(ship)
        phase = nil
    end
    if phase then
        local docking = ship:get(DockingComponents.Docking)
        if stage ~= "mine" and phase == Phase.Docked and docking:getStation() == target then
            agent.waitUntil = now + self.serviceS
            if stage == "park" then
                agent.job = nil
                return self:wait(agent, now, self.idleRetryS, false)
            elseif stage == "buy" then
                enter(agent, State.Buying, "buying " .. itemName(job.itemId) .. " at " .. nameOf(target))
            else
                enter(agent, State.Selling, "selling " .. itemName(job.itemId) .. " at " .. nameOf(target))
            end
            return
        end
        DockingSystem:undock(ship)
        agent.resume = true
        enter(agent, State.Undocking, "undocking")
        return
    end

    if stage == "mine" then
        agent.order = Nav.order(ship, Nav.Orders.MoveTo {
            target = targetEntity,
            approach = ApproachSpec.Of { kind = Enums.ApproachKind.StandOff, gapM = self.mineGapM },
        })
        enter(agent, State.FlyingToMine, "flying to mine " .. itemName(job.itemId))
        return
    end

    if not DockingSystem:requestDock(ship, targetEntity) then
        -- Every port is taken: try again shortly, keeping the job.
        return self:wait(agent, now, self.idleRetryS * 0.5, true)
    end
    if stage == "buy" then
        enter(agent, State.DockingToBuy, "flying to buy " .. itemName(job.itemId) .. " at " .. nameOf(target))
    elseif stage == "park" then
        enter(agent, State.DockingToSell, "docking at " .. nameOf(target) .. " to wait")
    else
        enter(agent, State.DockingToSell, "flying to sell " .. itemName(job.itemId) .. " at " .. nameOf(target))
    end
end

---The agent's next job: selling its cargo, else what its role does.
---@param ship Entity
---@param agent EconomyAgentComponent
---@param inventory InventoryComponent
---@return EconomyJob|nil
function EconomyAgentSystem:planWork(ship, agent, inventory)
    local job = self:planSale(ship, agent, inventory)
    if job then
        job.stage = "sell"
    elseif agent.role == Enums.EconomyAgentRole.Miner then
        job = self:planMining(ship, agent, inventory)
        if job then job.stage = "mine" end
    else
        job = self:planTrade(ship, inventory)
        if job then job.stage = "buy" end
    end
    return job
end

---@param ship Entity
---@param agent EconomyAgentComponent
---@param now number
---@param dt number
function EconomyAgentSystem:step(ship, agent, now, dt)
    local state = agent.state
    local inventory = ship:get(Economy.Inventory)

    if LEG_STATES[state] and now - (agent.stateSince or now) > self.maxLegS then
        Log.Debug("[EconomyAgent] %s gave up %s after %.0f s", tostring(ship), agent.activity, now - agent.stateSince)
        if DockingSystem:phaseOf(ship) then DockingSystem:release(ship) end
        self:endJob(agent)
        return self:wait(agent, now, self.idleRetryS, false)
    end

    if state == State.Idle then
        local job = self:planWork(ship, agent, inventory)
        if not job then
            -- Nothing pays: a ship out in space docks to wait; a docked one waits where it is.
            job = DockingSystem:phaseOf(ship) == nil and self:planPark(ship) or nil
            if not job then return self:wait(agent, now, self.idleRetryS, false) end
        end
        agent.job = job
        self:startStage(ship, agent, now)
    elseif state == State.Waiting then
        if now >= agent.waitUntil then
            if agent.resume and agent.job then self:startStage(ship, agent, now) else self:endJob(agent) end
        end
    elseif state == State.Undocking then
        if DockingSystem:phaseOf(ship) == nil then
            if agent.resume and agent.job then self:startStage(ship, agent, now) else self:endJob(agent) end
        end
    elseif state == State.DockingToBuy or state == State.DockingToSell then
        local phase = DockingSystem:phaseOf(ship)
        -- A ship on its way to park keeps looking for work, and takes it as soon as there is some.
        if agent.job and agent.job.stage == "park" and phase == Phase.Approaching
            and now >= (agent.nextLookAt or 0) then
            agent.nextLookAt = now + self.idleRetryS
            local job = self:planWork(ship, agent, inventory)
            if job then
                DockingSystem:release(ship)
                agent.job = job
                return self:startStage(ship, agent, now)
            end
        end
        if phase == Phase.Docked then
            self:startStage(ship, agent, now)
        elseif phase == Phase.Failed or phase == nil then
            DockingSystem:release(ship)
            self:wait(agent, now, self.idleRetryS * 0.5, true)
        end
    elseif state == State.Buying then
        if now >= agent.waitUntil then self:buy(ship, agent, now) end
    elseif state == State.Selling then
        if now >= agent.waitUntil then self:sell(ship, agent) end
    elseif state == State.FlyingToMine then
        self:flyToMine(ship, agent, now)
    elseif state == State.Mining then
        self:mine(ship, agent, now)
    end
end

---@param ship Entity
---@param agent EconomyAgentComponent
---@param now number
function EconomyAgentSystem:buy(ship, agent, now)
    local job = agent.job
    local station = alive(job.source)
    if not station then return self:endJob(agent) end
    local bought, spent = MarketplaceSystem:buyNow(station, ship, job.itemId, job.quantity,
        math.ceil(job.buyPrice * (1 + self.priceSlack)))
    agent.spent = agent.spent + spent
    dropClaims(job, "ask:")
    if bought <= 0 then return self:endJob(agent) end
    Log.Debug("[EconomyAgent] %s bought %d %s at %s for %d", tostring(ship), bought, itemName(job.itemId),
        nameOf(job.source), spent)
    job.quantity = bought
    job.stage = "sell"
    self:startStage(ship, agent, now)
end

---@param ship Entity
---@param agent EconomyAgentComponent
function EconomyAgentSystem:sell(ship, agent)
    local job = agent.job
    local station = alive(job.destination)
    if station then
        local held = InventoryManager:count(ship:get(Economy.Inventory), job.itemId)
        local sold, earned = MarketplaceSystem:sellNow(station, ship, job.itemId, held,
            math.max(1, math.floor(job.sellPrice * (1 - self.priceSlack))))
        agent.earned = agent.earned + earned
        if sold > 0 then
            agent.trips = agent.trips + 1
            Log.Debug("[EconomyAgent] %s sold %d %s at %s for %d", tostring(ship), sold, itemName(job.itemId),
                nameOf(job.destination), earned)
        end
    end
    -- Whatever didn't sell is sold elsewhere by the next job; the agent stays docked till then.
    self:endJob(agent)
end

---@param ship Entity
---@param agent EconomyAgentComponent
---@param now number
function EconomyAgentSystem:flyToMine(ship, agent, now)
    local rock = alive(agent.job.source)
    if not rock then return self:endJob(agent) end
    local status = agent.order and Nav.status(agent.order)
    if status == Enums.NavOrderStatus.Done then
        self:dropOrder(agent)
        local room = InventoryManager:getRoomFor(ship:get(Economy.Inventory), agent.job.itemId)
        if not MiningSystem:begin(ship, rock, math.min(agent.job.quantity, room), self.maxMiningS, now) then
            return self:endJob(agent)
        end
        enter(agent, State.Mining, "mining " .. itemName(agent.job.itemId))
    elseif not status or status == Enums.NavOrderStatus.Failed or status == Enums.NavOrderStatus.Cancelled then
        self:endJob(agent)
    end
end

---Waits for the mining run to end, then sells what it took.
---@param ship Entity
---@param agent EconomyAgentComponent
---@param now number
function EconomyAgentSystem:mine(ship, agent, now)
    local run = MiningSystem:runOf(ship)
    if run and not run:isDone() then return end
    local taken = MiningSystem:finish(ship)
    local job = agent.job
    if taken <= 0 then
        agent.badRocks = agent.badRocks or {}
        agent.badRocks[job.source] = now + self.badRockS
    end
    dropClaims(job, "rock:")
    dropClaims(job, "miners:")
    local held = InventoryManager:count(ship:get(Economy.Inventory), job.itemId)
    if held <= 0 then return self:endJob(agent) end
    job.quantity = held
    job.stage = "sell"
    self:startStage(ship, agent, now)
end

return EconomyAgentSystem()
