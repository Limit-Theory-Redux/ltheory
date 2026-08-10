local Items = require("Shared.Registries.Items")

--- SinglePlanetWithBelt: a focused single-planet scene. One star, one
--- rocky planet with an asteroid RING + BELT forced ON (Earth-sized body,
--- rocks in a ring band), one moon. Generated through the standard
--- UniverseManager pipeline, so the Visualizer materializes planets/
--- rings/belts with the instanced texture-fetch renderer automatically.
local Ruleset = {
    name = "SinglePlanetWithBelt",
    seed = 12345,
    starSystems = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.StarSystemTypes.Single },
            age = { type = Enums.Gen.Rule.Fixed, value = 4.6e9 },          -- Solar-like
            metallicity = { type = Enums.Gen.Rule.Fixed, value = 0.02 },   -- Solar-like
            stability = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.StarSystem.Stability.Stable }
        }
    },
    stars = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            position = {
                type = Enums.Gen.Rule.Fixed,
                default = Position(0, 0, 0)
            },
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.StarTypes.MainSequence },
            mass = { type = Enums.Gen.Rule.Fixed, value = 1.0 },
            luminosity = { type = Enums.Gen.Rule.Fixed, value = 1.0 }
        }
    },
    planets = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            orbitRadius = { type = Enums.Gen.Rule.Fixed, value = 1.0 },    -- habitable-zone close-in
            size = { type = Enums.Gen.Rule.Fixed, value = 1.0 },           -- Earth-sized
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.PlanetTypes.Rocky },
            atmosphere = { type = Enums.Gen.Rule.Fixed, value = true },
            asteroidRing = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.AsteroidRingTypes.Rocky },
            temperature = { type = Enums.Gen.Rule.Fixed, value = 288 },    -- 15C, Earth-like
            gravity = { type = Enums.Gen.Rule.Fixed, value = 9.81 },
            rotationPeriod = { type = Enums.Gen.Rule.Fixed, value = 24 },  -- hours
            eccentricity = { type = Enums.Gen.Rule.Fixed, value = 0.0167 },-- Earth-like
            magneticField = { type = Enums.Gen.Rule.Fixed, value = true },
            inclination = { type = Enums.Gen.Rule.Fixed, value = 23.4 },    -- Earth's axial tilt
            longitudeOfAscendingNode = { type = Enums.Gen.Rule.Fixed, value = 0 },
            argumentOfPeriapsis = { type = Enums.Gen.Rule.Fixed, value = 0 },
            meanAnomaly = { type = Enums.Gen.Rule.Fixed, value = 0 }
        }
    },
    moons = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            size = { type = Enums.Gen.Rule.Fixed, value = 0.27 },          -- Moon-like
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.MoonTypes.Rocky },
            orbitalRadius = { type = Enums.Gen.Rule.Fixed, value = 3.8e5 },
            inclination = { type = Enums.Gen.Rule.Fixed, value = 5.1 },
            longitudeOfAscendingNode = { type = Enums.Gen.Rule.Fixed, value = 0 },
            argumentOfPeriapsis = { type = Enums.Gen.Rule.Fixed, value = 0 },
            meanAnomaly = { type = Enums.Gen.Rule.Fixed, value = 0 }
        }
    },
    asteroidRings = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },                -- ring around the planet
        aspects = {
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.AsteroidRingTypes.Rocky },
            density = { type = Enums.Gen.Rule.Fixed, value = 0.5 },
            width = { type = Enums.Gen.Rule.Fixed, value = 0.1 },
            composition = {
                type = Enums.Gen.Rule.ByType,
                types = {
                    [Enums.Gen.AsteroidRingTypes.Rocky] = {
                        { id = Items.RawMaterials.SilicateOre.id, weight = 0.5 },
                        { id = Items.RawMaterials.IronOre.id,     weight = 0.4 },
                        { id = Items.RawMaterials.AluminumOre.id, weight = 0.1 }
                    }
                }
            }
        }
    },
    asteroidBelts = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },                -- belt between planets
        aspects = {
            orbitRadius = { type = Enums.Gen.Rule.Fixed, value = 2.5 },    -- just outside the planet
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.AsteroidRingTypes.Rocky },
            density = { type = Enums.Gen.Rule.Fixed, value = 0.5 },
            width = { type = Enums.Gen.Rule.Fixed, value = 1.0 },
            inclination = { type = Enums.Gen.Rule.Fixed, value = 5.0 },
            composition = {
                type = Enums.Gen.Rule.ByType,
                types = {
                    [Enums.Gen.AsteroidRingTypes.Rocky] = {
                        { id = Items.RawMaterials.SilicateOre.id, weight = 0.5 },
                        { id = Items.RawMaterials.IronOre.id,     weight = 0.4 },
                        { id = Items.RawMaterials.AluminumOre.id, weight = 0.1 }
                    }
                }
            }
        }
    },
    starZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 5e9 },
    planetZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 1e8 },
    moonZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 1e6 },
}

return Ruleset
