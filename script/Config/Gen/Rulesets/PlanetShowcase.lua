local Items = require("Shared.Registries.Items")

--- PlanetShowcase: one planet to look at, different per seed (the Planet
--- Showcase scenario; was the PlanetTest state). One star, one planet of a
--- random size and type, a ring for about a third of them, and one to three
--- moons spread around it.
local Ruleset = {
    name = "PlanetShowcase",
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
            size = { type = Enums.Gen.Rule.Range, min = 0.6, max = 2.5 },  -- Earth radii
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.PlanetTypes.Rocky,  weight = 0.5 },
                    { value = Enums.Gen.PlanetTypes.Icy,    weight = 0.25 },
                    { value = Enums.Gen.PlanetTypes.Desert, weight = 0.25 },
                }
            },
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
        count = { type = Enums.Gen.Rule.Range, min = 1, max = 3 },
        aspects = {
            size = { type = Enums.Gen.Rule.Fixed, value = 0.27 },          -- (the generator sizes moons by planet type)
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.MoonTypes.Rocky },
            orbitalRadius = { type = Enums.Gen.Rule.Range, min = 2.5e5, max = 9e5 },  -- km
            inclination = { type = Enums.Gen.Rule.Range, min = 0, max = 10 },
            longitudeOfAscendingNode = { type = Enums.Gen.Rule.Range, min = 0, max = 360 },
            argumentOfPeriapsis = { type = Enums.Gen.Rule.Fixed, value = 0 },
            meanAnomaly = { type = Enums.Gen.Rule.Range, min = 0, max = 360 }
        }
    },
    asteroidRings = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },                -- (not read: the type decides)
        aspects = {
            -- A ring for about a third of the planets
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.AsteroidRingTypes.None,  weight = 0.65 },
                    { value = Enums.Gen.AsteroidRingTypes.Rocky, weight = 0.35 },  -- (the composition below only has rocky rings)
                }
            },
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
        count = { type = Enums.Gen.Rule.Fixed, value = 0 },                -- rocks come from the planet's ring band
        aspects = {
            orbitRadius = { type = Enums.Gen.Rule.Fixed, value = 1.0 },
            type = { type = Enums.Gen.Rule.Fixed, value = Enums.Gen.AsteroidRingTypes.Rocky },
            density = { type = Enums.Gen.Rule.Fixed, value = 0.5 },
            width = { type = Enums.Gen.Rule.Fixed, value = 0.05 },
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
