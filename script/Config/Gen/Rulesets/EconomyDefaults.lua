-- The economy of a star system, for the rulesets that don't name their own: how many trading
-- factions there are and of which kinds, how many stations and ships of each kind (haulers, miners,
-- armed fighters) each has, how many ore fields there are, whether marauders raid the system, and
-- how many independents (ships their own captains own, of no faction) work it and of which kinds.
-- The ships' AI chooses their jobs by what they carry. UniverseManager evaluates it with
-- the system's seed (EconomyPlanner); the spawner builds it, scaled by the New Game density.
local Rule = Enums.Gen.Rule
local Kind = Enums.FactionType

return {
    factions = { type = Rule.Count, min = 1, max = 3 },
    factionType = {
        type = Rule.Weighted,
        values = {
            { value = Kind.Corporation, weight = 3 }, { value = Kind.TradingGuild, weight = 3 },
            { value = Kind.Kingdom, weight = 1 }, { value = Kind.DemocraticUnion, weight = 1 },
            { value = Kind.TribalUnion, weight = 1 }, { value = Kind.Empire, weight = 1 },
        },
    },
    stationsPerFaction = { type = Rule.Count, min = 2, max = 3 },
    haulersPerFaction = { type = Rule.Count, min = 2, max = 4 },
    minersPerFaction = { type = Rule.Count, min = 1, max = 2 },
    fightersPerFaction = { type = Rule.Count, min = 1, max = 2 },
    oreFields = { type = Rule.Count, min = 1, max = 2 },
    marauders = { type = Rule.Chance, value = 0.6 },
    pirates = { type = Rule.Count, min = 2, max = 4 },
    independents = { type = Rule.Count, min = 1, max = 3 },
    independentKind = {
        type = Rule.Weighted,
        values = { { value = "Hauler", weight = 3 }, { value = "Miner", weight = 2 } },
    },
}
