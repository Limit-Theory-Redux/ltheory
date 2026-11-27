local RuleEvaluator = require("Modules.CelestialObjects.Managers.UniverseManager.RuleEvaluator")
local EconomyDefaults = require("Config.Gen.Rulesets.EconomyDefaults")
local StationEconomies = require("Shared.Definitions.StationEconomyDefs")
local Items = require("Shared.Registries.Items")

-- Design: doc/script/spawning.md

---Plans a star system's economy from its ruleset's `economy` section (`EconomyDefaults` when it has
---none) and a seed: the trading factions with their stations and ships (haulers, miners, fighters),
---the ore fields, and the marauders. The plan is data (`EconomyLayout`); the spawner builds it.
---
---The stations of all factions together always make up a chain that trades: with four or more, a
---power plant, a refinery, a foundry and a colony first; each further station then makes what the
---stations so far run shortest of, by their recipes' rates, or when nothing runs short is another
---consumer (a colony, most often) that the next ones supply. With fewer, a power plant and outposts
---that live on its energy. They are
---dealt round the factions in turn, and placed round the planets, clear of the rings above their
---equatorial planes.
---@class EconomyPlanner
local EconomyPlanner = {}

-- The chain every system with four stations or more has; its kinds also make up the producers a
-- further station is chosen from.
local CHAIN = { "PowerPlant", "Refinery", "Foundry", "Colony" }
-- A further station when nothing runs short, by weight.
local CONSUMERS = { { value = "Colony", weight = 3 }, { value = "Foundry", weight = 1 }, { value = "Refinery", weight = 1 } }
-- Items no station makes (the miners bring the ore), so no station is added for them.
local MINED = { [Items.RawMaterials.IronOre.id] = true }
-- Stations are placed this many planet radii from the planet's centre, give or take, at this height.
local STATION_RADII, STATION_RADII_SPREAD, HEIGHT_RADII = 3.0, 0.6, 1.2

---Adds a station kind's rates to `net`: per item, what it makes less what it uses, per second.
---@param net table<integer, number>
---@param kind string
local function addRates(net, kind)
    for _, recipe in ipairs(StationEconomies[kind].recipes) do
        for _, stack in ipairs(recipe.inputs) do
            net[stack.item.id] = (net[stack.item.id] or 0) - stack.count / recipe.duration
        end
        for _, stack in ipairs(recipe.outputs) do
            net[stack.item.id] = (net[stack.item.id] or 0) + stack.count / recipe.duration
        end
    end
end

---The chain's kind that makes this item.
---@param itemId integer
---@return string|nil
local function producerOf(itemId)
    for _, kind in ipairs(CHAIN) do
        for _, recipe in ipairs(StationEconomies[kind].recipes) do
            for _, stack in ipairs(recipe.outputs) do
                if stack.item.id == itemId then return kind end
            end
        end
    end
end

---The next station's kind: the maker of what `net` runs shortest of, else a consumer.
---@param rng RNG
---@param net table<integer, number>
---@return string
local function nextKind(rng, net)
    local shortest, shortfall = nil, 0
    for itemId, rate in pairs(net) do
        if not MINED[itemId] and rate < shortfall then shortest, shortfall = itemId, rate end
    end
    return shortest and producerOf(shortest) or RuleEvaluator.evaluateWeighted(rng, CONSUMERS)
end

---@param rng RNG
---@param rule table|nil
---@param context table
---@param fallback any
local function evaluate(rng, rule, context, fallback)
    if not rule then return fallback end
    local value = RuleEvaluator.evaluate(rng, rule, context)
    if value == nil then return fallback end
    return value
end

---@param economy table|nil The ruleset's `economy` section
---@param seed integer
---@param planetCount integer
---@param context table|nil The generation context, for conditional rules
---@return EconomyLayout|nil layout Nil for a system without planets or with no factions
function EconomyPlanner.plan(economy, seed, planetCount, context)
    if planetCount < 1 then return nil end
    local rules = economy or EconomyDefaults
    local rng = RNG.Create(seed)
    context = context or {}

    local layout = { factions = {}, oreFields = {} }
    local factionCount = evaluate(rng, rules.factions, context, 0)
    if factionCount < 1 then return nil end

    local stationCount = 0
    for i = 1, factionCount do
        local faction = {
            type = evaluate(rng, rules.factionType, context, Enums.FactionType.Corporation),
            seed = rng:get31(),
            stations = {},
            haulers = evaluate(rng, rules.haulersPerFaction, context, 2),
            miners = evaluate(rng, rules.minersPerFaction, context, 1),
            fighters = evaluate(rng, rules.fightersPerFaction, context, 0),
            stationCount = evaluate(rng, rules.stationsPerFaction, context, 2),
        }
        stationCount = stationCount + faction.stationCount
        layout.factions[i] = faction
    end

    -- The kinds, so that together they trade.
    local kinds = {}
    if stationCount >= #CHAIN then
        local net = {}
        for _, kind in ipairs(CHAIN) do
            kinds[#kinds + 1] = kind
            addRates(net, kind)
        end
        while #kinds < stationCount do
            local kind = nextKind(rng, net)
            kinds[#kinds + 1] = kind
            addRates(net, kind)
        end
    else
        kinds[1] = "PowerPlant"
        while #kinds < stationCount do kinds[#kinds + 1] = "Outpost" end
    end

    -- Dealt round the factions, placed round the planets.
    local index = 0
    for slot = 1, stationCount do
        local faction
        repeat
            index = index % factionCount + 1
            faction = layout.factions[index]
        until #faction.stations < faction.stationCount
        faction.stations[#faction.stations + 1] = {
            kind = kinds[slot],
            planet = (slot - 1) % planetCount + 1,
            angleDeg = (slot - 1) * 137.5 + rng:getUniformRange(-15, 15),
            radii = STATION_RADII + rng:getUniformRange(-STATION_RADII_SPREAD, STATION_RADII_SPREAD),
            heightRadii = HEIGHT_RADII,
        }
    end
    for _, faction in ipairs(layout.factions) do faction.stationCount = nil end

    for i = 1, evaluate(rng, rules.oreFields, context, 1) do
        layout.oreFields[i] = {
            planet = (i - 1) % planetCount + 1,
            angleDeg = rng:getUniformRange(0, 360),
            radii = STATION_RADII + 0.8,
            heightRadii = HEIGHT_RADII,
            count = 6 + rng:getInt(0, 4),
        }
    end

    if evaluate(rng, rules.marauders, context, false) then
        layout.marauders = {
            pirates = evaluate(rng, rules.pirates, context, 2),
            place = { planet = rng:getInt(1, planetCount), angleDeg = rng:getUniformRange(0, 360), radii = 6, heightRadii = 2 },
        }
    end
    return layout
end

return EconomyPlanner
