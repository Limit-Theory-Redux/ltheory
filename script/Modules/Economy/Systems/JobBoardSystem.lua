local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local PhysicsComponents = require("Modules.Physics.Components")
local Economy = require("Modules.Economy.Components")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")
local AccountManager = require("Modules.Economy.Managers.AccountManager")

-- Design: doc/script/economy.md

---@class TradeOffer
---@field source Entity The station that asks for the item
---@field destination Entity The station that bids for it
---@field itemId integer
---@field buyPrice integer The source's best ask
---@field sellPrice integer The destination's best bid
---@field supply integer What the source asks at that price
---@field demand integer What the destination bids for in all

---@class MiningOffer
---@field rock Entity
---@field station Entity A station that bids for the rock's item
---@field itemId integer
---@field sellPrice integer
---@field left integer What the rock holds
---@field demand integer What the station bids for in all
---@field useRate number What the station's production uses of the item per simulated second

---@class PatrolOffer
---@field station Entity A threatened station, which pays for its security
---@field threat number
---@field payout number What a patrol of `duration` earns
---@field duration number Seconds
---@field slots integer How many ships its threat calls for

---@class SaleBid
---@field station Entity
---@field itemId integer
---@field price integer
---@field demand integer

---The work a star system offers, gathered every `refreshS` seconds instead of by every agent on every
---decision: trade routes (an item asked for at one station for less than another bids), mining runs
---(a rock's item bid for at a station), the bids an agent with cargo can sell to, and security: a
---patrol at each station whose threat (`ThreatSystem`) is at least `minThreat`, paid by the station
---by its threat. Agents take work from it and weigh it by their own place, hold and
---credits; what they plan to take they claim with the agent system, so `unfilledValue` is what is
---left. Runs at PreSim.
---@class JobBoardSystem
local JobBoardSystem = Class("JobBoardSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function JobBoardSystem:registerVars()
    self.profiler = QuickProfiler("JobBoardSystem", false, false)
    self.refreshS = 2
    self.nextRefreshAt = 0
    ---@type TradeOffer[]
    self.trades = {}
    ---@type MiningOffer[]
    self.mining = {}
    ---@type table<integer, SaleBid[]> Item id -> the stations that bid for it
    self.bids = {}
    ---@type PatrolOffer[]
    self.patrols = {}
    -- Security: a station posts a patrol from this threat, paying `payPerThreat` a threat (at most
    -- `maxPayout`) for `patrolS` seconds, and wants a ship for each `threatPerSlot` of its threat.
    self.minThreat = 1
    self.payPerThreat = 250
    self.maxPayout = 3000
    self.patrolS = 150
    self.threatPerSlot = 3
    -- A station spends at most this share of what it earns (`AccountManager:earningRate`) on
    -- security: a patrol pays no more than the share of its earnings over the patrol, split over the
    -- patrols it posts. A station that earns little can afford little protection.
    self.securityShare = 0.25
end

function JobBoardSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function JobBoardSystem:onPreSim(e)
    local now = MarketplaceSystem:now()
    if now >= self.nextRefreshAt then
        self:refresh()
        self.nextRefreshAt = now + self.refreshS
    end
end

---@return Entity[] stations The entities with a market and a body
local function markets()
    local stations = {}
    for entity in Registry:view(Economy.Marketplace) do
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then stations[#stations + 1] = entity end
    end
    table.sort(stations, function(a, b) return a.id < b.id end)
    return stations
end

---Gathers the offers again from the markets' books.
function JobBoardSystem:refresh()
    self.profiler:start()
    local stations = markets()
    local asks, bids = {}, {}
    for _, station in ipairs(stations) do
        local market = station:get(Economy.Marketplace)
        local items = {}
        for _, set in ipairs({ market:getAsks(), market:getBids() }) do
            for _, order in ipairs(set) do
                if Registry:hasEntity(order) then items[order:get(Economy.Item):getItem()] = true end
            end
        end
        for itemId in pairs(items) do
            local quote = MarketplaceSystem:quote(station, itemId)
            if quote.bestAsk then
                asks[#asks + 1] = { station = station, itemId = itemId, price = quote.bestAsk, supply = quote.askQuantity }
            end
            if quote.bestBid then
                bids[itemId] = bids[itemId] or {}
                table.insert(bids[itemId], { station = station, itemId = itemId, price = quote.bestBid, demand = quote.demand })
            end
        end
    end

    local trades = {}
    for _, ask in ipairs(asks) do
        for _, bid in ipairs(bids[ask.itemId] or {}) do
            if bid.station.id ~= ask.station.id and bid.price > ask.price then
                trades[#trades + 1] = {
                    source = ask.station, destination = bid.station, itemId = ask.itemId,
                    buyPrice = ask.price, sellPrice = bid.price, supply = ask.supply, demand = bid.demand,
                }
            end
        end
    end

    local mining = {}
    for rock, _, item, quantity in Registry:iterEntities(Economy.Yield, Economy.Item, Economy.Quantity) do
        local left = quantity:getQuantity()
        if left > 0 then
            for _, bid in ipairs(bids[item:getItem()] or {}) do
                local production = bid.station:get(Economy.Production)
                local useRate = production and production:getRates()[item:getItem()] or 0
                mining[#mining + 1] = {
                    rock = rock, station = bid.station, itemId = item:getItem(), sellPrice = bid.price,
                    left = left, demand = bid.demand, useRate = useRate,
                }
            end
        end
    end

    local patrols = {}
    local threats = require("Modules.Factions.Systems.ThreatSystem")
    for _, station in ipairs(stations) do
        local threat = threats:threatOf(station)
        if threat >= self.minThreat then
            local slots = math.max(1, math.ceil(threat / self.threatPerSlot))
            local budget = self.securityShare * AccountManager:earningRate(station) * self.patrolS / slots
            local payout = math.floor(math.min(self.maxPayout, threat * self.payPerThreat, budget))
            if payout > 0 then
                patrols[#patrols + 1] = {
                    station = station, threat = threat, payout = payout, duration = self.patrolS, slots = slots,
                }
            end
        end
    end

    self.trades, self.mining, self.bids, self.patrols = trades, mining, bids, patrols
    self.profiler:stop()
end

---@return TradeOffer[]
function JobBoardSystem:getTrades()
    return self.trades
end

---@return MiningOffer[]
function JobBoardSystem:getMining()
    return self.mining
end

---@return PatrolOffer[]
function JobBoardSystem:getPatrols()
    return self.patrols
end

---How many unarmed agents (haulers, miners) of factions hostile to `faction` there are: a raider's prey.
---@param faction Entity
---@return integer
function JobBoardSystem:countRaidTargets(faction)
    local FactionManager = require("Modules.Factions.Managers.FactionManager")
    local WeaponControl = require("Modules.Constructs.Components").WeaponControl
    local count = 0
    local PeopleManager = require("Modules.People.Managers.PeopleManager")
    for commander in Registry:view(Economy.EconomyAgent) do
        local ship = PeopleManager:postOf(commander)
        if ship and not ship:get(WeaponControl) and FactionManager:isHostile(faction, FactionManager:factionOf(ship)) then
            count = count + 1
        end
    end
    return count
end

---@param itemId integer
---@return SaleBid[]
function JobBoardSystem:getBids(itemId)
    return self.bids[itemId] or {}
end

---The value of the work nobody has claimed: a trade route's margin, a mining run's sale price, times
---the quantity left of it after the claims. A route counts its best lot once per source and item.
---@param kind "trade"|"mining"|"patrol"
---@param claimed fun(key: string): integer The agent system's claims
---@return number credits
function JobBoardSystem:unfilledValue(kind, claimed)
    local total, seen = 0, {}
    if kind == "trade" then
        for _, offer in ipairs(self.trades) do
            local key = offer.source.id .. ":" .. offer.itemId
            local open = math.min(offer.supply - claimed("ask:" .. key), offer.demand - claimed("bid:" .. offer.destination.id .. ":" .. offer.itemId))
            local value = math.max(0, open) * (offer.sellPrice - offer.buyPrice)
            seen[key] = math.max(seen[key] or 0, value)
        end
    elseif kind == "patrol" then
        for _, offer in ipairs(self.patrols) do
            local open = offer.slots - claimed("patrol:" .. offer.station.id)
            seen[offer.station.id] = math.max(0, open) * offer.payout
        end
    else
        for _, offer in ipairs(self.mining) do
            local key = offer.station.id .. ":" .. offer.itemId
            local open = math.min(offer.left - claimed("rock:" .. offer.rock.id .. ":" .. offer.itemId), offer.demand - claimed("bid:" .. key))
            seen[key] = math.max(seen[key] or 0, math.max(0, open) * offer.sellPrice)
        end
    end
    for _, value in pairs(seen) do total = total + value end
    return total
end

return JobBoardSystem()
