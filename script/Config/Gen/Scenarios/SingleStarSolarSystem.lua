return {
    name = "Single Star Solar System",

    seed = 1,

    ---@type table<string, GenerationRule>
    rules = {
        starSystems    = { type = Enums.Gen.Rule.Fixed, value = 1 },
        stars          = { type = Enums.Gen.Rule.Fixed, value = 1 },

        planets        = { type = Enums.Gen.Rule.Fixed, value = 8 },

        moons          = { type = Enums.Gen.Rule.Count, min = 0, max = 3 },

        rings          = { type = Enums.Gen.Rule.Chance, value = 0.3 },

        asteroidBelts  = { type = Enums.Gen.Rule.Chance, value = 0.7 },

        stations       = { type = Enums.Gen.Rule.Count, min = 0, max = 2 },
        ships          = { type = Enums.Gen.Rule.Count, min = 0, max = 5 },

        starZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 1.7952e13 },
    },
}
