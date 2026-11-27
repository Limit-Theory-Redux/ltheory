local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Items = require("Shared.Registries.Items")
local ApproachSpec = require("Shared.Types.ApproachSpec")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local EntityPosition = require("Shared.Helpers.EntityPosition")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local DockingSystem = require("Modules.Docking.Systems.DockingSystem")
local MiningSystem = require("Modules.Economy.Systems.MiningSystem")
local JobBoardSystem = require("Modules.Economy.Systems.JobBoardSystem")
local FactionManager = require("Modules.Factions.Managers.FactionManager")
local ConstructComponents = require("Modules.Constructs.Components")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")
local NameComponent = require("Modules.Core.Components.NameComponent")
local DockingComponents = require("Modules.Docking.Components")
local PeopleManager = require("Modules.People.Managers.PeopleManager")
local JobPlanner = require("Shared.Economy.JobPlanner")
local JobSnapshot = require("Modules.Economy.Managers.JobSnapshot")
local FrameScheduler = require("Shared.Tools.FrameScheduler")

-- The agent is the decision-making of a ship's commander: it lives on the person and moves with command.
PeopleManager:registerCommandComponent(Economy.EconomyAgent)

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

---Runs the economic agents (`EconomyAgentComponent`): the commanders of ships, people who pick a job
---for the ship they command from the markets and fly it (the agent lives on the person and passes
---with command, `PeopleManager`). A trader buys an item at the station that asks least for it and sells it to one that
---bids more; a miner flies to an asteroid that yields an item some station bids for, fills its hold
---and sells there. A ship with cargo first sells it where it is bid for best. Jobs are scored by
---profit over the time they take, from navigation estimates of the legs; the quantities a job
---plans to buy, sell or mine are held against the other agents, so two agents don't chase one lot.
---The choosing is the captains' thinking: the captains that need work think together, off the main
---thread (`JobPlanner` on a worker, from a `JobSnapshot`), and their answers are applied when they come.
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
    self.minProfit = 30
    -- The captains that need work think together every `thinkPeriodS`: one snapshot of the job board
    -- goes to the thinking worker (`JobPlannerWorker.lua`), or with `thinkInWorker` off, to the
    -- planner on the main thread, spread over frames (`FrameScheduler`; LTHEORY_THINK_INLINE sets it).
    self.thinkPeriodS = 0.5
    self.thinkInWorker = os.getenv("LTHEORY_THINK_INLINE") == nil
    -- Seconds added to a job's time for each docking.
    self.dockOverheadS = 40
    -- A miner's run lasts at most `maxMiningS`; it flies to `mineGapM` off the rock's hull first,
    -- and the mining run's glide takes it onto its circle from there.
    self.maxMiningS = 600
    -- A run is planned to take about `miningRunS` at the rate miners make (`MiningSystem:expectedRate`),
    -- so a station's demand goes to several miners at once rather than one hold that takes many minutes.
    self.miningRunS = 180
    self.mineGapM = 1500
    self.maxMinersPerAsteroid = 2
    -- A trade may pay this much over the planned buy price, or take this much under the planned sell price.
    self.priceSlack = 0.1
    -- A flight or docking leg that hasn't ended after this many seconds is given up, with its job.
    self.maxLegS = 900
    -- A miner whose run took nothing leaves that rock alone for this many seconds.
    self.badRockS = 300
    -- A raid lasts `raidS` and is reckoned worth `raidValue` while there are victims about.
    self.raidS = 240
    self.raidValue = 1500
    self.raidPatrolRadius = ScaleConfig:toGameUnits(40e3, "starSystem")
    ---@type Entity[] Navigation orders cancelled and not yet ended, released once they have
    self.cancelled = {}
    ---@type table<integer, { ship: Entity, agent: EconomyAgentComponent }> Captains waiting for a thought, by ship id
    self.awaiting = {}
    self.nextThinkAt = 0
    self.thinkTaskId = nil
    ---@type FrameTask|nil The thinking on the main thread, while it runs
    self.thinkTask = nil
    self.thinkWorker = nil
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
    -- The agents are people who command a ship; the ship is what they decide for.
    for commander, agent in Registry:view(Economy.EconomyAgent) do
        local ship = PeopleManager:postOf(commander)
        if ship then agents[#agents + 1] = { ship, agent } end
    end
    self:collectClaims(agents)
    local now = MarketplaceSystem:now()
    currentNow = now
    for _, entry in ipairs(agents) do
        local ship, agent = entry[1], entry[2]
        if ship:get(PhysicsComponents.RigidBody) and ship:get(Economy.Inventory) then
            self:step(ship, agent, now, dt)
        end
    end
    self:think(now)
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

---Notes a share the job will hold if it is chosen (see `commitClaims`).
---@param job EconomyJob
---@param key string
---@param quantity integer
local function plan(job, key, quantity)
    job.planned = job.planned or {}
    job.planned[#job.planned + 1] = { key = key, quantity = quantity }
end

---Holds the shares of the job that was chosen.
---@param job EconomyJob
local function commitClaims(job)
    for _, c in ipairs(job.planned or {}) do claim(job, c.key, c.quantity) end
    job.planned = nil
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

-- Thinking ---------------------------------------------------------------------------------------

---A captain needs work: its ship waits for the next thought (`think`).
---@param ship Entity
---@param agent EconomyAgentComponent
function EconomyAgentSystem:awaitThought(ship, agent)
    self.awaiting[ship.id] = { ship = ship, agent = agent }
end

---Gives out the answers that have come, and sends the captains waiting for one off to think when
---none is under way and `thinkPeriodS` has passed.
---@param now number
function EconomyAgentSystem:think(now)
    self:collectThoughts()
    if self.thinkTaskId or self.thinkTask or now < self.nextThinkAt or not next(self.awaiting) then return end
    self.nextThinkAt = now + self.thinkPeriodS

    local ships = {}
    for id, entry in pairs(self.awaiting) do
        if Registry:hasEntity(entry.ship) and entry.ship:get(Economy.Inventory) then
            ships[#ships + 1] = entry
        else
            self.awaiting[id] = nil
        end
    end
    if #ships == 0 then return end
    table.sort(ships, function(a, b) return a.ship.id < b.ship.id end)
    local snapshot = JobSnapshot.build(ships, JobBoardSystem, self.claims or {}, {
        now = now, dockOverheadS = self.dockOverheadS, minProfit = self.minProfit,
        miningRate = MiningSystem:expectedRate(), miningRunS = self.miningRunS,
        maxMinersPerAsteroid = self.maxMinersPerAsteroid, raidS = self.raidS, raidValue = self.raidValue,
    })
    if self.thinkInWorker then
        if not self.thinkWorker then
            self.thinkWorker = TaskQueue:startWorker("JobPlanner", "script/Shared/Economy/JobPlannerWorker.lua", 1)
            assert(self.thinkWorker, "EconomyAgentSystem: the thinking worker did not start")
        end
        self.thinkTaskId = TaskQueue:sendTask(self.thinkWorker, snapshot)
    else
        self.thinkTask = FrameScheduler:spawn("captains think", JobPlanner.planAll, snapshot,
            function() FrameScheduler:checkpoint() end)
    end
end

---Takes the answers of the thinking under way, when they have come.
function EconomyAgentSystem:collectThoughts()
    local values = nil
    if self.thinkTaskId then
        while true do
            local taskId, payload, err = TaskQueue:nextTaskResult(self.thinkWorker)
            if not taskId then break end
            if taskId == self.thinkTaskId then
                assert(not err, "EconomyAgentSystem: the thinking worker failed: " .. tostring(err))
                values = payload.values or {}
                self.thinkTaskId = nil
            end
        end
    elseif self.thinkTask and self.thinkTask:isDone() then
        values = self.thinkTask:getResult() or {}
        self.thinkTask = nil
    end
    if values then self:applyThoughts(values) end
end

---Turns each answer into its captain's job: its shares are claimed, and its ship sets off. An
---answer whose places are gone since is dropped, and its captain thinks again.
---@param values number[] `JobPlanner.Layout.Result` records
function EconomyAgentSystem:applyThoughts(values)
    local R, Kind = JobPlanner.Layout.Result, JobPlanner.Kind
    local now = currentNow
    for base = 1, #values, R.Stride do
        local shipId = values[base + R.Agent]
        local entry = self.awaiting[shipId]
        self.awaiting[shipId] = nil
        local ship, agent = entry and entry.ship, entry and entry.agent
        if ship and Registry:hasEntity(ship) and PeopleManager:commandOf(ship, Economy.EconomyAgent) == agent then
            local job = self:jobOf(values, base)
            if job and (job.kind ~= "park" or agent.state == State.Idle) then
                commitClaims(job)
                if agent.state == State.Idle then
                    agent.job = job
                    self:startStage(ship, agent, now)
                elseif job.kind ~= "park" and agent.job and agent.job.stage == "park" then
                    -- Work for a ship on its way to wait: it takes it at once.
                    DockingSystem:release(ship)
                    agent.job = job
                    self:startStage(ship, agent, now)
                end
            elseif agent.state == State.Idle then
                self:wait(agent, now, self.idleRetryS, false)
            end
        end
    end
end

---The job an answer describes, with the shares it plans to hold; nil for no job or one whose places
---are gone.
---@param values number[]
---@param base integer
---@return EconomyJob|nil
function EconomyAgentSystem:jobOf(values, base)
    local R, Kind = JobPlanner.Layout.Result, JobPlanner.Kind
    local kind = values[base + R.Kind]
    local item, quantity = values[base + R.Item], values[base + R.Quantity]
    local source, destination = values[base + R.Source], values[base + R.Destination]
    local job = { claims = {}, score = values[base + R.Score] }
    if kind == Kind.Sale then
        job.kind, job.stage = "sale", "sell"
        job.itemId, job.quantity, job.destination, job.sellPrice = item, quantity, destination, values[base + R.Sell]
        plan(job, claimKey("bid", destination, item), quantity)
    elseif kind == Kind.Trade then
        job.kind, job.stage = "trade", "buy"
        job.itemId, job.quantity, job.source, job.destination = item, quantity, source, destination
        job.buyPrice, job.sellPrice = values[base + R.Buy], values[base + R.Sell]
        plan(job, claimKey("ask", source, item), quantity)
        plan(job, claimKey("bid", destination, item), quantity)
    elseif kind == Kind.Mining then
        job.kind, job.stage = "mining", "mine"
        job.itemId, job.quantity, job.source, job.destination, job.sellPrice = item, quantity, source, destination, values[base + R.Sell]
        plan(job, claimKey("rock", source, item), quantity)
        plan(job, "miners:" .. source, 1)
        plan(job, claimKey("bid", destination, item), quantity)
    elseif kind == Kind.Patrol then
        job.kind, job.stage = "patrol", "fight"
        job.destination, job.payout, job.duration = destination, values[base + R.Payout], values[base + R.Duration]
        plan(job, "patrol:" .. destination, 1)
    elseif kind == Kind.Raid then
        job.kind, job.stage, job.duration = "raid", "fight", values[base + R.Duration]
    elseif kind == Kind.Park then
        job.kind, job.stage, job.destination, job.score = "park", "park", destination, 0
    else
        return nil
    end
    if (job.source and not alive(job.source)) or (job.destination and not alive(job.destination)) then return nil end
    return job
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
    local target = (stage == "sell" or stage == "park" or stage == "fight") and job.destination or job.source
    local targetEntity = alive(target)
    if not targetEntity and stage ~= "fight" then return self:endJob(agent) end

    local phase = DockingSystem:phaseOf(ship)
    if phase == Phase.Failed then
        DockingSystem:release(ship)
        phase = nil
    end
    if phase then
        local docking = ship:get(DockingComponents.Docking)
        if stage ~= "mine" and stage ~= "fight" and phase == Phase.Docked and docking:getStation() == target then
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

    if stage == "fight" then return self:beginFight(ship, agent, targetEntity, now) end

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
        -- Its captain thinks with the others (`think`); the answer sets the ship off, or, when
        -- nothing pays, a ship out in space docks to wait and a docked one waits where it is.
        if not self.awaiting[ship.id] then self:awaitThought(ship, agent) end
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
            and now >= (agent.nextLookAt or 0) and not self.awaiting[ship.id] then
            agent.nextLookAt = now + self.idleRetryS
            self:awaitThought(ship, agent)
        end
        if phase == Phase.Docked then
            self:startStage(ship, agent, now)
        elseif phase == Phase.Failed or phase == nil then
            DockingSystem:release(ship)
            self:wait(agent, now, self.idleRetryS * 0.5, true)
        end
    elseif state == State.Fighting then
        local job = agent.job
        local guarded = job and job.kind == "patrol" and alive(job.destination)
        if now >= agent.waitUntil or (job and job.kind == "patrol" and not guarded) then self:endFight(ship, agent) end
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

-- Points round a centre for a patrol: `count` of them on a circle of `radius`, tilted a little.
---@param centre Position
---@param radius number
---@param count integer
---@return Position[]
local function circleRound(centre, radius, count)
    local points = {}
    for i = 1, count do
        local angle = (i - 1) * 2 * math.pi / count
        points[i] = Position(centre.x + math.cos(angle) * radius, centre.y + math.sin(angle * 2) * radius * 0.2,
            centre.z + math.sin(angle) * radius)
    end
    return points
end

---Hands the ship to `CombatAgentSystem` for its fighting job: a patrol round the station it guards,
---or a raid from where it is.
---@param ship Entity
---@param agent EconomyAgentComponent
---@param station Entity|nil The station a patrol guards
---@param now number
function EconomyAgentSystem:beginFight(ship, agent, station, now)
    local job = agent.job
    local CombatAgent = require("Modules.Factions.Components").CombatAgent
    local centre, radius, task, postId, points = nil, nil, nil, nil, 4
    if job.kind == "patrol" then
        if not station then return self:endJob(agent) end
        local body = station:get(PhysicsComponents.RigidBody):getRigidBody()
        centre, radius, task = body:getPos(), body:getBoundingRadius() * 8, Enums.CombatTask.Patrol
        -- The guards of a station fly its route as a wing: many points, so its turns are gentle.
        postId, points = station.id, 16
        agent.activity = "patrolling at " .. nameOf(station.id)
    else
        centre, radius, task = EntityPosition.Get(ship), self.raidPatrolRadius, Enums.CombatTask.Raid
        agent.activity = "raiding"
    end
    if not ship:get(CombatAgent) then ship:add(CombatAgent(task, circleRound(centre, radius, points), postId)) end
    agent.waitUntil = now + job.duration
    enter(agent, State.Fighting, agent.activity)
end

---Ends the fighting job: the ship leaves `CombatAgentSystem`, and a patrol is paid by its station.
---@param ship Entity
---@param agent EconomyAgentComponent
function EconomyAgentSystem:endFight(ship, agent)
    require("Modules.Factions.Systems.CombatAgentSystem"):release(ship)
    local job = agent.job
    local station = job and job.kind == "patrol" and alive(job.destination)
    if station then
        local paid = math.min(job.payout, AccountManager:balanceOf(station))
        if paid > 0 and AccountManager:transfer(AccountManager:accountOf(station), AccountManager:accountOf(ship), paid) then
            agent:addEarnings("patrol", paid)
            PeopleManager:reportEarnings(ship, "patrol", paid)
        end
    end
    agent.trips = agent.trips + 1
    self:endJob(agent)
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
        local activity = job.kind == "mining" and "mining" or (job.kind == "sale" and "sale" or "trade")
        agent:addEarnings(activity, earned)
        PeopleManager:reportEarnings(ship, activity, earned)
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
