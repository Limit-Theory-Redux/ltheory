local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local DockingSystem = require("Modules.Docking.Systems.DockingSystem")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local Factions = require("Modules.Factions.Components")
local FactionManager = require("Modules.Factions.Managers.FactionManager")
local PeopleManager = require("Modules.People.Managers.PeopleManager")
local FormationSystem = require("Modules.Formations.Systems.FormationSystem")
local People = require("Modules.People.Components")

-- Design: doc/script/factions.md

---Flies the ships on fighting jobs (`CombatAgentComponent`). On a patrol, a ship circles its points
---(round the station it guards) and, when a ship of a hostile faction comes within `guardRange`,
---follows it close; its weapons do the rest. On a raid, a ship hunts the nearest trader or miner of a
---faction it is hostile to within `huntRange` (not one docked), and follows it; staying within
---`raidRange` for `raidS` forces part of the victim's cargo out into its hold, fences part of its
---value into the raider's account, raises the threat at the victim's stations, turns the victim's
---faction against the raider, and sends it off to find another. A raider that a hostile patrol comes
---close to breaks off. The decisions are made every `scanS`; the flights are navigation
---orders (Patrol, Follow). The ships that patrol one station fly it as a wing (`FormationSystem`): the
---most experienced captain leads and flies the route, the others hold their slots, and an engagement
---(a Combat order) takes a wingman out of its slot until it ends. Runs at PreSim.
---@class CombatAgentSystem
local CombatAgentSystem = Class("CombatAgentSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function CombatAgentSystem:registerVars()
    self.profiler = QuickProfiler("CombatAgentSystem", false, false)
    self.scanS = 2
    self.guardRange = ScaleConfig:toGameUnits(40e3, "starSystem")
    -- Raiders hunt across the star system (the travel drive closes the distance).
    self.huntRange = ScaleConfig:toGameUnits(1e11, "starSystem")
    self.raidRange = ScaleConfig:toGameUnits(3e3, "starSystem")
    self.raidS = 10
    -- The share of each cargo a raid takes, and how far a raid turns the victim's faction.
    self.raidShare = 0.3
    self.raidPenalty = 0.2
    -- The share of the loot's start price a raider's fence pays.
    self.fenceShare = 0.5
    -- After a raid, or when security drives it off, a pirate leaves victims alone this long.
    self.pirateCooldownS = 60
    -- Followers keep this far behind their target, in metres.
    self.followGapM = 1500
    ---@type Entity[]
    self.cancelled = {}
end

function CombatAgentSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function CombatAgentSystem:onPreSim(e)
    self:update()
end

---@param id integer|nil
---@return Entity|nil
local function alive(id)
    if not id then return nil end
    local entity = Entity(id)
    return Registry:hasEntity(entity) and entity or nil
end

---@param entity Entity
---@return Position|nil
local function positionOf(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local body = rbCmp and rbCmp:getRigidBody()
    return body and body:getPos() or nil
end

---@param a Position
---@param b Position
---@return number
local function distance(a, b)
    local dx, dy, dz = a.x - b.x, a.y - b.y, a.z - b.z
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

function CombatAgentSystem:update()
    self.profiler:start()
    local now = MarketplaceSystem:now()
    self:releaseCancelled()
    local agents = {}
    for ship, agent in Registry:view(Factions.CombatAgent) do agents[#agents + 1] = { ship, agent } end
    for _, entry in ipairs(agents) do
        local ship, agent = entry[1], entry[2]
        if ship:get(PhysicsComponents.RigidBody) then
            if now >= (agent.nextScanAt or 0) then
                agent.nextScanAt = now + self.scanS
                self:decide(ship, agent, now)
            end
            if agent.task == Enums.CombatTask.Raid then self:raid(ship, agent, now) end
        end
    end
    self.profiler:stop()
end

function CombatAgentSystem:releaseCancelled()
    local waiting = {}
    for _, order in ipairs(self.cancelled) do
        if Registry:hasEntity(order) then
            if order:get(Nav.Components.OrderStatus):hasEnded() then Nav.release(order) else waiting[#waiting + 1] = order end
        end
    end
    self.cancelled = waiting
end

---@param agent CombatAgentComponent
function CombatAgentSystem:dropOrder(agent)
    local order = agent.order
    agent.order, agent.orderKind = nil, nil
    if not order or not Registry:hasEntity(order) then return end
    if order:get(Nav.Components.OrderStatus):hasEnded() then
        Nav.release(order)
    else
        Nav.cancel(order)
        self.cancelled[#self.cancelled + 1] = order
    end
end

---Ends the ship's fighting job: its order is dropped and its `CombatAgentComponent` removed.
---@param ship Entity
function CombatAgentSystem:release(ship)
    local agent = ship:get(Factions.CombatAgent)
    if not agent then return end
    self:dropOrder(agent)
    FormationSystem:leave(ship)
    ship:remove(Factions.CombatAgent)
end

---How senior a ship's commander is, to lead a wing: the patrols and other jobs they have done.
---@param ship Entity
---@return number
local function seniority(ship)
    local captain = PeopleManager.commanderOf(ship)
    local career = captain and captain:get(People.Career)
    if not career then return 0 end
    return career:getExperience("patrol") * 2 + career:getExperience("trade") + career:getExperience("mining")
end

---Puts a patrolling ship in the wing of the station it guards: it starts the wing, or takes the
---first slot behind the leader whose captain is less senior than its own (the leader keeps the lead).
---@param ship Entity
---@param agent CombatAgentComponent
function CombatAgentSystem:joinWing(ship, agent)
    self.wings = self.wings or {}
    local wing = self.wings[agent.postId]
    if wing and not Registry:hasEntity(wing) then wing = nil end
    if wing and FormationSystem:formationOf(ship) == wing then return end
    if not wing then
        self.wings[agent.postId] = FormationSystem:create("Wedge", { ship })
        Log.Debug("[CombatAgent] %s leads the wing guarding %d", tostring(ship), agent.postId)
        return
    end
    local members = wing:get(require("Modules.Formations.Components").Formation):getMembers()
    local mine, slot = seniority(ship), #members + 1
    for i = 2, #members do
        local other = alive(members[i])
        if other and seniority(other) < mine then slot = i break end
    end
    FormationSystem:join(wing, ship, slot)
    Log.Debug("[CombatAgent] %s joins the wing guarding %d in slot %d", tostring(ship), agent.postId, slot)
end

---The nearest ship to `from` that `accept` takes, within `range`.
---@param from Position
---@param range number
---@param accept fun(ship: Entity): boolean
---@return Entity|nil, number|nil
local function nearest(from, range, accept)
    local best, bestDistance = nil, range
    for ship in Registry:view(ConstructComponents.Targetable) do
        local pos = positionOf(ship)
        if pos then
            local d = distance(from, pos)
            if d < bestDistance and accept(ship) then best, bestDistance = ship, d end
        end
    end
    return best, best and bestDistance or nil
end

---@param ship Entity
---@param agent CombatAgentComponent
---@param now number
function CombatAgentSystem:decide(ship, agent, now)
    local pos = positionOf(ship)
    local own = FactionManager.factionOf(ship)
    local target = alive(agent.target)

    -- Keep a target while it lives and stays within reach; else look for one.
    local range = agent.task == Enums.CombatTask.Patrol and self.guardRange or self.huntRange
    local targetPos = target and positionOf(target)
    if not targetPos or distance(pos, targetPos) > range * 1.5 then target = nil end

    if agent.task == Enums.CombatTask.Raid then
        -- Security close by drives a pirate off.
        local guard = nearest(pos, self.raidRange * 2, function(other)
            local combat = other:get(Factions.CombatAgent)
            return combat ~= nil and combat:getTask() == Enums.CombatTask.Patrol
                and FactionManager.isHostile(own, FactionManager.factionOf(other))
        end)
        if guard then
            agent.cooldownUntil, target = now + self.pirateCooldownS, nil
            agent.activity = "fleeing security"
        elseif not target and now >= (agent.cooldownUntil or 0) then
            -- Prey: an unarmed agent of a hostile faction, out of port; one with cargo first.
            local function prey(other, laden)
                local economic = PeopleManager.commandOf(other, Economy.EconomyAgent)
                if not economic or other:get(ConstructComponents.WeaponControl) then return false end
                if not FactionManager.isHostile(own, FactionManager.factionOf(other)) then return false end
                local phase = DockingSystem:phaseOf(other)
                if phase ~= nil and phase ~= Enums.DockingPhase.Approaching then return false end
                local cargo = other:get(Economy.Inventory)
                return not laden or (cargo ~= nil and InventoryManager:getMass(cargo) > 0)
            end
            target = nearest(pos, self.huntRange, function(other) return prey(other, true) end)
                or nearest(pos, self.huntRange, function(other) return prey(other, false) end)
        end
    else
        if not target then
            target = nearest(pos, self.guardRange, function(other)
                return other ~= ship and FactionManager.isHostile(own, FactionManager.factionOf(other))
            end)
        end
    end

    if target then
        if agent.target ~= target.id or agent.orderKind ~= "follow" then
            self:dropOrder(agent)
            agent.target, agent.closeSince = target.id, nil
            -- A Combat order: it takes over from a wing's slot-keeping, which goes on after.
            agent.order = Nav.order(ship, Nav.Orders.Follow { target = target, offset = Vec3d(0, 0, -self.followGapM),
                priority = Nav.Priority.Combat })
            agent.orderKind = "follow"
        end
        local name = target:get(require("Modules.Core.Components.NameComponent"))
        agent.activity = (agent.task == Enums.CombatTask.Raid and "hunting " or "engaging ")
            .. (name and name:getName() or tostring(target))
    else
        agent.target = nil
        if agent.task == Enums.CombatTask.Patrol and agent.postId then self:joinWing(ship, agent) end
        if FormationSystem:isWingman(ship) then
            -- The wing flies it: no route of its own.
            if agent.orderKind ~= "wing" then
                self:dropOrder(agent)
                agent.orderKind = "wing"
            end
        elseif agent.orderKind ~= "patrol" and #agent.patrol >= 2 then
            self:dropOrder(agent)
            agent.order = Nav.order(ship, Nav.Orders.Patrol { points = agent.patrol, loop = true })
            agent.orderKind = "patrol"
        end
        if agent.activity ~= "fleeing security" or now >= (agent.cooldownUntil or 0) then agent.activity = "patrolling" end
    end
end

---A pirate that has stayed close to its victim long enough takes part of its cargo.
---@param ship Entity
---@param agent CombatAgentComponent
---@param now number
function CombatAgentSystem:raid(ship, agent, now)
    local victim = alive(agent.target)
    local pos, victimPos = positionOf(ship), victim and positionOf(victim)
    if not victimPos or distance(pos, victimPos) > self.raidRange then
        agent.closeSince = nil
        return
    end
    agent.closeSince = agent.closeSince or now
    if now - agent.closeSince < self.raidS then return end

    local hold, cargo = ship:get(Economy.Inventory), victim:get(Economy.Inventory)
    local taken = 0
    if hold and cargo then
        for itemId in pairs(cargo:getInventory()) do
            local quantity = math.floor(InventoryManager:count(cargo, itemId) * self.raidShare)
            quantity = math.min(quantity, InventoryManager:getRoomFor(hold, itemId))
            if quantity > 0 and InventoryManager:remove(cargo, itemId, quantity) then
                InventoryManager:add(hold, itemId, quantity)
                taken = taken + quantity
                agent.lastLoot = agent.lastLoot or {}
                agent.lastLoot[itemId] = (agent.lastLoot[itemId] or 0) + quantity
            end
        end
    end
    agent.raids = agent.raids + taken
    local victimFaction, own = FactionManager.factionOf(victim), FactionManager.factionOf(ship)
    if victimFaction and own then FactionManager.adjust(victimFaction, own, -self.raidPenalty) end
    require("Modules.Factions.Systems.ThreatSystem"):reportRaid(victim)
    -- The loot is fenced at once for a share of its start price; the raider keeps the rest aboard.
    local fenced = 0
    for itemId, quantity in pairs(agent.lastLoot or {}) do
        fenced = fenced + quantity * MarketplaceSystem:getStartPrice(itemId) * self.fenceShare
    end
    agent.lastLoot = nil
    if fenced > 0 then
        AccountManager:deposit(AccountManager:accountOf(ship), math.floor(fenced))
        local economic = PeopleManager.commandOf(ship, Economy.EconomyAgent)
        if economic then economic:addEarnings("raid", math.floor(fenced)) end
        PeopleManager.reportEarnings(ship, "raid", math.floor(fenced))
    end
    Log.Debug("[CombatAgent] %s raided %s for %d units", tostring(ship), tostring(victim), taken)
    agent.cooldownUntil, agent.target, agent.closeSince = now + self.pirateCooldownS, nil, nil
    self:dropOrder(agent)
    agent.activity = "making off with cargo"
end

return CombatAgentSystem()
