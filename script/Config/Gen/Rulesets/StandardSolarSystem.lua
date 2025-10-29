local Items = require("Shared.Registries.Items")

---@type Ruleset
local Ruleset = {
    name = "StandardSolarSystem",
    starSystems = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            type = {
                type = Enums.Gen.Rule.Fixed,
                condition = {
                    type = Enums.Gen.Condition.StarCount,
                    ranges = {
                        { min = 1, max = 1, value = Enums.Gen.StarSystemTypes.Single },
                        { min = 2, max = 2, value = Enums.Gen.StarSystemTypes.Binary },
                        { min = 3, max = 3, value = Enums.Gen.StarSystemTypes.Trinary }
                    }
                },
                default = Enums.Gen.StarSystemTypes.Single
            },
            age = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = 1e9,   weight = 0.2 },  -- 1 billion years
                    { value = 3e9,   weight = 0.3 },  -- 3 billion years
                    { value = 4.6e9, weight = 0.3 },  -- Solar-like
                    { value = 7e9,   weight = 0.15 }, -- Older
                    { value = 10e9,  weight = 0.05 }  -- Very old
                },
                default = 4.6e9
            },
            metallicity = {
                type = Enums.Gen.Rule.Range,
                min = 0.01,
                max = 0.04,
                default = 0.02
            },
            stability = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.StarSystem.Stability.Stable },
                    { value = Enums.Gen.StarSystem.Stability.Unstable },
                    { value = Enums.Gen.StarSystem.Stability.Chaotic }
                },
                condition = {
                    type = Enums.Gen.Condition.Combined,
                    criteria = {
                        -- Young systems (0–5e9 years)
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge,         min = 0,   max = 5e9 },
                                { type = Enums.Gen.Condition.StarCount,         min = 1,   max = 1 },
                                { type = Enums.Gen.Condition.SystemMetallicity, min = 0.0, max = 0.1 }
                            },
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.9,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.08,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 0.02
                            }
                        },
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge,         min = 0,   max = 5e9 },
                                { type = Enums.Gen.Condition.StarCount,         min = 2,   max = 2 },
                                { type = Enums.Gen.Condition.SystemMetallicity, min = 0.0, max = 0.1 }
                            },
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.6,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.3,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 0.1
                            }
                        },
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge,         min = 0,   max = 5e9 },
                                { type = Enums.Gen.Condition.StarCount,         min = 3,   max = 3 },
                                { type = Enums.Gen.Condition.SystemMetallicity, min = 0.0, max = 0.1 }
                            },
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.0,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.0,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 1.0
                            }
                        },
                        -- Older systems (5e9–13.8e9 years)
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge,         min = 5e9, max = 13.8e9 },
                                { type = Enums.Gen.Condition.StarCount,         min = 1,   max = 1 },
                                { type = Enums.Gen.Condition.SystemMetallicity, min = 0.0, max = 0.1 }
                            },
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.95,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.04,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 0.01
                            }
                        },
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge,         min = 5e9, max = 13.8e9 },
                                { type = Enums.Gen.Condition.StarCount,         min = 2,   max = 2 },
                                { type = Enums.Gen.Condition.SystemMetallicity, min = 0.0, max = 0.1 }
                            },
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.7,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.2,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 0.1
                            }
                        },
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge,         min = 5e9, max = 13.8e9 },
                                { type = Enums.Gen.Condition.StarCount,         min = 3,   max = 3 },
                                { type = Enums.Gen.Condition.SystemMetallicity, min = 0.0, max = 0.1 }
                            },
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.0,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.0,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 1.0
                            }
                        }
                    }
                }
            }
        }
    },
    stars = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
            position = {
                type = Enums.Gen.Rule.Fixed,
                default = Position(0, 0, 0)
            },
            type = {
                type = Enums.Gen.Rule.Fixed,
                value = Enums.Gen.StarTypes.MainSequence,
                condition = {
                    type = Enums.Gen.Condition.SystemAge,
                    ranges = {
                        { min = 1e9, max = 5e9,  value = Enums.Gen.StarTypes.MainSequence },
                        { min = 5e9, max = 8e9,  value = Enums.Gen.StarTypes.RedGiant },
                        { min = 8e9, max = 10e9, value = Enums.Gen.StarTypes.WhiteDwarf }
                    }
                }
            },
            mass = {
                type = Enums.Gen.Rule.Range,
                min = 0.8,
                max = 2.0,
                condition = {
                    type = Enums.Gen.Condition.StarType,
                    types = {
                        [Enums.Gen.StarTypes.MainSequence] = { min = 0.8, max = 1.5 },
                        [Enums.Gen.StarTypes.RedGiant] = { min = 1.0, max = 2.0 },
                        [Enums.Gen.StarTypes.WhiteDwarf] = { min = 0.8, max = 1.2 }
                    }
                }
            },
            luminosity = {
                type = Enums.Gen.Rule.Range,
                min = 0.1,
                max = 100,
                condition = {
                    type = Enums.Gen.Condition.StarMass,
                    ranges = {
                        { min = 0.8, max = 1.2, minLuminosity = 0.1,  maxLuminosity = 2.0 },
                        { min = 1.2, max = 1.6, minLuminosity = 2.0,  maxLuminosity = 20.0 },
                        { min = 1.6, max = 2.0, minLuminosity = 20.0, maxLuminosity = 100.0 }
                    }
                }
            }
        }
    },
    planets = {
        count = {
            type = Enums.Gen.Rule.Weighted,
            values = {
                { value = 1,  weight = 0.05 },
                { value = 2,  weight = 0.05 },
                { value = 3,  weight = 0.05 },
                { value = 4,  weight = 0.1 },
                { value = 5,  weight = 0.15 },
                { value = 6,  weight = 0.2 },
                { value = 7,  weight = 0.2 },
                { value = 8,  weight = 0.15 },
                { value = 9,  weight = 0.1 },
                { value = 10, weight = 0.05 },
                { value = 11, weight = 0.03 },
                { value = 12, weight = 0.02 }
            }
        },
        aspects = {
            orbitRadius = { type = Enums.Gen.Rule.Range, min = 0.1, max = 10.0 },
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.PlanetTypes.Rocky,    weight = 0.5 },
                    { value = Enums.Gen.PlanetTypes.GasGiant, weight = 0.3 },
                    { value = Enums.Gen.PlanetTypes.Icy,      weight = 0.15 },
                    { value = Enums.Gen.PlanetTypes.Desert,   weight = 0.05 }
                },
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        {
                            min = 0.1,
                            max = 0.7,
                            weights = {
                                [Enums.Gen.PlanetTypes.Rocky] = 0.8,
                                [Enums.Gen.PlanetTypes.Desert] = 0.2
                            }
                        },
                        {
                            min = 0.7,
                            max = 2.0,
                            weights = {
                                [Enums.Gen.PlanetTypes.Rocky] = 0.5,
                                [Enums.Gen.PlanetTypes.GasGiant] = 0.3,
                                [Enums.Gen.PlanetTypes.Icy] = 0.2
                            }
                        },
                        {
                            min = 2.0,
                            max = 10.0,
                            weights = {
                                [Enums.Gen.PlanetTypes.GasGiant] = 0.6,
                                [Enums.Gen.PlanetTypes.Icy] = 0.4
                            }
                        }
                    }
                }
            },
            atmosphere = {
                type = Enums.Gen.Rule.Chance,
                value = 0.6,
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        [Enums.Gen.PlanetTypes.Rocky] = { chance = 0.7 },
                        [Enums.Gen.PlanetTypes.GasGiant] = { chance = 0.9 },
                        [Enums.Gen.PlanetTypes.Icy] = {
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.None] = 0.7,
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.1,
                                [Enums.Gen.AsteroidRingTypes.Icy] = 0.2
                            }
                        },
                        [Enums.Gen.PlanetTypes.Desert] = {
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.None] = 0.95,
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.05
                            }
                        }
                    }
                }
            },
            -- New orbital parameters for 3D positioning
            longitudeOfAscendingNode = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 360,
                default = 0
            },
            argumentOfPeriapsis = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 360,
                default = 0
            },
            meanAnomaly = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 360,
                default = 0
            }
        }
    },
    moons = {
        count = {
            type = Enums.Gen.Rule.Range,
            min = 0,
            max = 3,
            condition = { type = Enums.Gen.Condition.PlanetCount, ranges = { { min = 0, max = 4 }, { min = 5, max = 8 }, { min = 9, max = 12 } } }
        },
        aspects = {
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.MoonTypes.Rocky, weight = 0.6 },
                    { value = Enums.Gen.MoonTypes.Icy,   weight = 0.4 }
                },
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        [Enums.Gen.PlanetTypes.GasGiant] = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.5,
                                [Enums.Gen.MoonTypes.Icy] = 0.5
                            }
                        },
                        [Enums.Gen.PlanetTypes.Rocky] = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.8,
                                [Enums.Gen.MoonTypes.Icy] = 0.2
                            }
                        },
                        [Enums.Gen.PlanetTypes.Icy] = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.3,
                                [Enums.Gen.MoonTypes.Icy] = 0.7
                            }
                        },
                        [Enums.Gen.PlanetTypes.Desert] = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.9,
                                [Enums.Gen.MoonTypes.Icy] = 0.1
                            }
                        }
                    }
                }
            },
            orbitalRadius = {
                type = Enums.Gen.Rule.Range,
                min = 1e5,
                max = 1e6,
            },
            inclination = { type = Enums.Gen.Rule.Range, min = 0.0, max = 5.0, default = 0.0 },
            longitudeOfAscendingNode = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 360,
                default = 0
            },
            argumentOfPeriapsis = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 360,
                default = 0
            },
            meanAnomaly = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 360,
                default = 0
            }
        }
    },
    asteroidRings = {
        count = { type = Enums.Gen.Rule.Chance, value = 0.3 },
        aspects = {
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.AsteroidRingTypes.Rocky, weight = 0.5 },
                    { value = Enums.Gen.AsteroidRingTypes.Icy,   weight = 0.5 }
                },
                condition = {
                    type = Enums.Gen.Condition.Combined,
                    criteria = {
                        -- Close-in orbit: mostly Rocky
                        {
                            conditions = { { type = Enums.Gen.Condition.OrbitRadius, min = 0.1, max = 2.0 } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.8,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.2
                            }
                        },
                        -- Far orbit: mostly Icy
                        {
                            conditions = { { type = Enums.Gen.Condition.OrbitRadius, min = 2.0, max = 10.0 } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.2,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.8
                            }
                        },
                        -- Young systems: slightly more Rocky
                        {
                            conditions = { { type = Enums.Gen.Condition.SystemAge, min = 0, max = 3e9 } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.6,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.4
                            }
                        },
                        -- Gas giant host planets: more icy debris
                        {
                            conditions = { { type = Enums.Gen.Condition.PlanetType, types = { [Enums.Gen.PlanetTypes.GasGiant] = true } } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.3,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.7
                            }
                        }
                    }
                }
            },
            density = {
                type = Enums.Gen.Rule.Range,
                min = 0.2,
                max = 1.0,
                default = 0.5
            },
            width = {
                type = Enums.Gen.Rule.Range,
                min = 0.05,
                max = 0.5,
                default = 0.1
            },
            composition = {
                type = Enums.Gen.Rule.ByType,
                types = {
                    [Enums.Gen.AsteroidRingTypes.Rocky] = {
                        { id = Items.RawMaterials.SilicateOre.id,  weight = 0.5 },
                        { id = Items.RawMaterials.IronOre.id,      weight = 0.3 },
                        { id = Items.RawMaterials.AluminumOre.id,  weight = 0.15 },
                        { id = Items.RawMaterials.BerylliumOre.id, weight = 0.05 }
                    },
                    [Enums.Gen.AsteroidRingTypes.Icy] = {
                        { id = Items.Consumables.WaterIce.id,     weight = 0.6 },
                        { id = Items.RawMaterials.SilicateOre.id, weight = 0.2 },
                        { id = Items.Gases.Hydrogen.id,           weight = 0.1 },
                        { id = Items.Gases.Helium.id,             weight = 0.05 },
                        { id = Items.Gases.Nitrogen.id,           weight = 0.03 },
                        { id = Items.Gases.Oxygen.id,             weight = 0.02 }
                    }
                }
            }
        }
    },
    asteroidBelts = {
        count = { type = Enums.Gen.Rule.Chance, value = 0.6 },
        aspects = {
            orbitRadius = {
                type = Enums.Gen.Rule.Range,
                condition = {
                    type = Enums.Gen.Condition.Combined,
                    criteria = {
                        -- Inner belts around Sun-like stars
                        {
                            conditions = {
                                {
                                    type = Enums.Gen.Condition.StarType,
                                    types = {
                                        [Enums.Gen.StarTypes.MainSequence] = true
                                    }
                                }
                            },
                            min = 1.0,
                            max = 3.5
                        },
                        -- Outer belts around Sun-like stars
                        {
                            conditions = {
                                {
                                    type = Enums.Gen.Condition.StarType,
                                    types = {
                                        [Enums.Gen.StarTypes.MainSequence] = true }
                                }
                            },
                            min = 3.5,
                            max = 15.0
                        },
                        -- For very young systems, belts are more compact
                        {
                            conditions = {
                                { type = Enums.Gen.Condition.SystemAge, min = 0, max = 1e9
                                }
                            },
                            min = 0.5,
                            max = 5.0
                        },
                        -- Around giant stars, belts are pushed outward
                        {
                            conditions = {
                                {
                                    type = Enums.Gen.Condition.StarType,
                                    types = {
                                        [Enums.Gen.StarTypes.RedGiant] = true }
                                }
                            },
                            min = 5.0,
                            max = 15.0
                        }
                    }
                }
            },
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.AsteroidRingTypes.Rocky, weight = 0.4 },
                    { value = Enums.Gen.AsteroidRingTypes.Icy,   weight = 0.6 }
                },
                condition = {
                    type = Enums.Gen.Condition.Combined,
                    criteria = {
                        -- Close orbit: rocky
                        {
                            conditions = { { type = Enums.Gen.Condition.OrbitRadius, min = 0.5, max = 3.0 } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.7,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.3
                            }
                        },
                        -- Outer orbit: icy
                        {
                            conditions = { { type = Enums.Gen.Condition.OrbitRadius, min = 3.0, max = 15.0 } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.2,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.8
                            }
                        },
                        -- Young systems: more rocky
                        {
                            conditions = { { type = Enums.Gen.Condition.SystemAge, min = 0, max = 2e9 } },
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.6,
                                [Enums.Gen.AsteroidRingTypes.Icy]   = 0.4
                            }
                        }
                    }
                }
            },
            density = {
                type = Enums.Gen.Rule.Range,
                min = 0.1,
                max = 1.0,
                default = 0.5
            },
            width = {
                type = Enums.Gen.Rule.Range,
                min = 0.5,
                max = 5.0,
                default = 1.0
            },
            inclination = {
                type = Enums.Gen.Rule.Range,
                min = 0,
                max = 15,
                default = 5
            },
            composition = {
                type = Enums.Gen.Rule.ByType,
                types = {
                    [Enums.Gen.AsteroidRingTypes.Rocky] = {
                        { id = Items.RawMaterials.SilicateOre.id, weight = 0.5 },
                        { id = Items.RawMaterials.IronOre.id,     weight = 0.4 },
                        --{ id = Items.RawMaterials.NickelOre.id,   weight = 0.1 }
                    },
                    [Enums.Gen.AsteroidRingTypes.Icy] = {
                        { id = Items.Consumables.WaterIce.id,     weight = 0.7 },
                        { id = Items.Gases.Hydrogen.id,           weight = 0.15 },
                        { id = Items.Gases.Helium.id,             weight = 0.1 },
                        { id = Items.RawMaterials.SilicateOre.id, weight = 0.05 }
                    }
                }
            }
        }
    },
    starZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 1.5e11 }
}

return Ruleset
