local Items = require("Shared.Registries.Items")

local Ruleset = {
    name = "Test: 2 Asteroids + Player Ship",
    seed = 1,
    starSystems = {
        count = { type = "Fixed", value = 1 },
        aspects = {
            age = { type = "Fixed", value = 4.6e9 },         -- Default: 4.6 billion years
            metallicity = { type = "Fixed", value = 0.02 },  -- Default metallicity
            stability = { type = "Fixed", value = "Stable" } -- Default stability
        }
    },
    stars = {
        count = { type = "Fixed", value = 1 },
        aspects = {
            type = { type = "Fixed", value = "MainSequence" },
            mass = { type = "Fixed", value = 1.0 },      -- Solar mass
            luminosity = { type = "Fixed", value = 1.0 } -- Solar luminosity
        }
    },
    planets = {
        count = { type = "Fixed", value = 0 },
        aspects = {
            orbitRadius = { type = "Fixed", value = 0 },
            size = { type = "Fixed", value = 0 },
            type = { type = "Fixed", value = "None" },
            atmosphere = { type = "Fixed", value = false },
            asteroidRing = { type = "Fixed", value = "None" },
            temperature = { type = "Fixed", value = 0 },
            gravity = { type = "Fixed", value = 0 },
            rotationPeriod = { type = "Fixed", value = 0 },
            eccentricity = { type = "Fixed", value = 0 },
            magneticField = { type = "Fixed", value = false },
            inclination = { type = "Fixed", value = 0 }
        }
    },
    moons = {
        count = { type = "Fixed", value = 0 },
        aspects = {
            size = { type = "Fixed", value = 0 },
            type = { type = "Fixed", value = "None" },
            orbitalDistance = { type = "Fixed", value = 0 },
            inclination = { type = "Fixed", value = 0 }
        }
    },
    rings = {
        count = { type = "Fixed", value = 0 },
        aspects = {
            composition = { type = "Fixed", value = "None" },
            thickness = { type = "Fixed", value = 0 }
        }
    },
    asteroidBelts = {
        count = { type = "Fixed", value = 0 },
        aspects = {
            density = { type = "Fixed", value = 0 },
            composition = { type = "Fixed", value = "None" },
            width = { type = "Fixed", value = 0 }
        }
    },
    starZoneRadius = { type = "Fixed", value = 1.5e11 }, -- Default star zone radius
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
