local StandardSolarSystem = require("Config.Gen.Rulesets.StandardSolarSystem")

local Ruleset = DeepClone(StandardSolarSystem)
Ruleset.name = "StandardBinarySolarSystem"

Ruleset.starSystems.count = { type = Enums.Gen.Rule.Fixed, value = 1 }
Ruleset.stars.count = { type = Enums.Gen.Rule.Fixed, value = 2 }
Ruleset.stars.aspects.position = {
    type = Enums.Gen.Rule.Fixed,
    condition = {
        type = Enums.Gen.Condition.StarIndex,
        ranges = {
            { min = 1, max = 1, value = Position(-5e8, 0, 0) },
            { min = 2, max = 2, value = Position(5e8, 0, 0) }
        }
    },
    default = Position(0, 0, 0)
}
Ruleset.starSystems.aspects.metallicity = {
    type = Enums.Gen.Rule.Range,
    min = 0.02,
    max = 0.05,
    default = 0.03
}
Ruleset.planets.count = {
    type = Enums.Gen.Rule.Weighted,
    values = {
        { value = 1, weight = 0.1 },
        { value = 2, weight = 0.2 },
        { value = 3, weight = 0.3 },
        { value = 4, weight = 0.2 },
        { value = 5, weight = 0.15 },
        { value = 6, weight = 0.05 }
    }
}
Ruleset.planets.aspects.orbitRadius = {
    type = Enums.Gen.Rule.Range,
    min = 0.5,
    max = 5.0
}
Ruleset.planets.aspects.atmosphere = {
    type = Enums.Gen.Rule.Chance,
    value = 0.5,
    condition = {
        type = Enums.Gen.Condition.PlanetType,
        types = {
            [Enums.Gen.PlanetTypes.Rocky] = { chance = 0.6 },
            [Enums.Gen.PlanetTypes.GasGiant] = { chance = 0.8 },
            [Enums.Gen.PlanetTypes.Icy] = { chance = 0.3 },
            [Enums.Gen.PlanetTypes.Desert] = { chance = 0.0 }
        }
    },
    default = false
}
Ruleset.asteroidBelts.count = { type = Enums.Gen.Rule.Chance, value = 0.5 }

return Ruleset
