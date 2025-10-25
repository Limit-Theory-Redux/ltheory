local StandardSolarSystem = require("Config.Gen.Rulesets.StandardSolarSystem")

local Ruleset = DeepClone(StandardSolarSystem)
Ruleset.name = "StandardTrinarySolarSystem"

Ruleset.starSystems.count = { type = Enums.Gen.Rule.Fixed, value = 1 }
Ruleset.stars.count = { type = Enums.Gen.Rule.Fixed, value = 3 }
Ruleset.stars.aspects.position = {
    type = Enums.Gen.Rule.Fixed,
    condition = {
        type = Enums.Gen.Condition.StarIndex,
        ranges = {
            { min = 1, max = 1, value = { x = 0, y = 5e8, z = 0 } },
            { min = 2, max = 2, value = { x = -4.33e8, y = -2.5e8, z = 0 } },
            { min = 3, max = 3, value = { x = 4.33e8, y = -2.5e8, z = 0 } }
        }
    },
    default = { x = 0, y = 0, z = 0 }
}
Ruleset.starSystems.aspects.metallicity = {
    type = Enums.Gen.Rule.Range,
    min = 0.03,
    max = 0.06,
    default = 0.04
}
Ruleset.planets.count = {
    type = Enums.Gen.Rule.Weighted,
    values = {
        { value = 1, weight = 0.2 },
        { value = 2, weight = 0.3 },
        { value = 3, weight = 0.3 },
        { value = 4, weight = 0.15 },
        { value = 5, weight = 0.05 }
    }
}
Ruleset.planets.aspects.orbitRadius = {
    type = Enums.Gen.Rule.Range,
    min = 0.7,
    max = 4.0
}
Ruleset.planets.aspects.atmosphere = {
    type = Enums.Gen.Rule.Chance,
    value = 0.4,
    condition = {
        type = Enums.Gen.Condition.PlanetType,
        types = {
            [Enums.Gen.PlanetTypes.Rocky] = { chance = 0.5 },
            [Enums.Gen.PlanetTypes.GasGiant] = { chance = 0.7 },
            [Enums.Gen.PlanetTypes.Icy] = { chance = 0.2 },
            [Enums.Gen.PlanetTypes.Desert] = { chance = 0.0 }
        }
    },
    default = false
}
Ruleset.asteroidBelts.count = { type = Enums.Gen.Rule.Chance, value = 0.3 }

return Ruleset
