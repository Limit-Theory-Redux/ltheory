local Items = require("Shared.Registries.Items")

local Ruleset = {
    name = "Test: 2 Asteroids + Player Ship",
    seed = 1,
    starSystems = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            age = { type = Enums.Gen.Rule.Fixed, value = 4.6e9 },         -- Default: 4.6 billion years
            metallicity = { type = Enums.Gen.Rule.Fixed, value = 0.02 },  -- Default metallicity
            stability = { type = Enums.Gen.Rule.Fixed, value = "Stable" } -- Default stability
        }
    },
    stars = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            type = { type = Enums.Gen.Rule.Fixed, value = "MainSequence" },
            mass = { type = Enums.Gen.Rule.Fixed, value = 1.0 },      -- Solar mass
            luminosity = { type = Enums.Gen.Rule.Fixed, value = 1.0 } -- Solar luminosity
        }
    },
    planets = {
        count = { type = Enums.Gen.Rule.Fixed, value = 0 },
        aspects = {
            orbitRadius = { type = Enums.Gen.Rule.Fixed, value = 0 },
            size = { type = Enums.Gen.Rule.Fixed, value = 0 },
            type = { type = Enums.Gen.Rule.Fixed, value = "None" },
            atmosphere = { type = Enums.Gen.Rule.Fixed, value = false },
            asteroidRing = { type = Enums.Gen.Rule.Fixed, value = "None" },
            temperature = { type = Enums.Gen.Rule.Fixed, value = 0 },
            gravity = { type = Enums.Gen.Rule.Fixed, value = 0 },
            rotationPeriod = { type = Enums.Gen.Rule.Fixed, value = 0 },
            eccentricity = { type = Enums.Gen.Rule.Fixed, value = 0 },
            magneticField = { type = Enums.Gen.Rule.Fixed, value = false },
            inclination = { type = Enums.Gen.Rule.Fixed, value = 0 }
        }
    },
    moons = {
        count = { type = Enums.Gen.Rule.Fixed, value = 0 },
        aspects = {
            size = { type = Enums.Gen.Rule.Fixed, value = 0 },
            type = { type = Enums.Gen.Rule.Fixed, value = "None" },
            orbitalDistance = { type = Enums.Gen.Rule.Fixed, value = 0 },
            inclination = { type = Enums.Gen.Rule.Fixed, value = 0 }
        }
    },
    rings = {
        count = { type = Enums.Gen.Rule.Fixed, value = 0 },
        aspects = {
            composition = { type = Enums.Gen.Rule.Fixed, value = "None" },
            thickness = { type = Enums.Gen.Rule.Fixed, value = 0 }
        }
    },
    asteroidBelts = {
        count = { type = Enums.Gen.Rule.Fixed, value = 0 },
        aspects = {
            density = { type = Enums.Gen.Rule.Fixed, value = 0 },
            composition = { type = Enums.Gen.Rule.Fixed, value = "None" },
            width = { type = Enums.Gen.Rule.Fixed, value = 0 }
        }
    },
    starZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 1.5e11 }, -- Default star zone radius
    overrides = {
        starSystem = {
            {
                type = "Asteroid",
                position = Position(10000, -5000, 0),
                components = {
                    ItemComponent = Items.RawMaterials.SilicateOre,
                    QuantityComponent = 50000
                }
            },
            {
                type = "Asteroid",
                position = Position(-8000, 3000, 0),
                components = {
                    ItemComponent = Items.RawMaterials.IronOre,
                    QuantityComponent = 50000
                }
            },
            {
                type = "Spaceship",
                position = Position(1000, 2000, 0),
                components = {}
            }
        }
    }
}

return Ruleset
