---@type Ruleset
local Ruleset = {
    name = "StandardSolarSystem",
    starSystems = {
        count = { type = Enums.Gen.Rule.Fixed, value = 1 },
        aspects = {
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
                    { value = Enums.Gen.StarSystem.Stability.Stable,   weight = 0.7 },
                    { value = Enums.Gen.StarSystem.Stability.Unstable, weight = 0.2 },
                    { value = Enums.Gen.StarSystem.Stability.Chaotic,  weight = 0.1 }
                },
                condition = {
                    type = Enums.Gen.Condition.SystemAge,
                    ranges = {
                        {
                            min = 1e9,
                            max = 3e9,
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.5,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.3,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 0.2
                            }
                        },
                        {
                            min = 3e9,
                            max = 10e9,
                            weights = {
                                [Enums.Gen.StarSystem.Stability.Stable] = 0.8,
                                [Enums.Gen.StarSystem.Stability.Unstable] = 0.15,
                                [Enums.Gen.StarSystem.Stability.Chaotic] = 0.05
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
                { value = 1,  weight = 0.05 }, -- Minimum 1 planet
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
            size = {
                type = Enums.Gen.Rule.Range,
                min = 0.5,
                max = 10.0,
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        { min = 0.1, max = 0.7,  minSize = 0.5, maxSize = 2.0 },
                        { min = 0.7, max = 2.0,  minSize = 1.0, maxSize = 5.0 },
                        { min = 2.0, max = 10.0, minSize = 5.0, maxSize = 10.0 }
                    }
                }
            },
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
                        Enums.Gen.PlanetTypes.Rocky,
                        Enums.Gen.PlanetTypes.GasGiant,
                        Enums.Gen.PlanetTypes.Icy
                    }
                }
            },
            asteroidRing = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.AsteroidRingTypes.None,  weight = 0.7 },
                    { value = Enums.Gen.AsteroidRingTypes.Rocky, weight = 0.2 },
                    { value = Enums.Gen.AsteroidRingTypes.Icy,   weight = 0.1 }
                },
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        [Enums.Gen.PlanetTypes.GasGiant] = {
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.None] = 0.4,
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.3,
                                [Enums.Gen.AsteroidRingTypes.Icy] = 0.3
                            }
                        },
                        [Enums.Gen.PlanetTypes.Rocky] = {
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.None] = 0.9,
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.08,
                                [Enums.Gen.AsteroidRingTypes.Icy] = 0.02
                            }
                        },
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
            temperature = {
                type = Enums.Gen.Rule.Range,
                min = 100,
                max = 500,
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        { min = 0.1, max = 0.7,  minTemp = 300, maxTemp = 500 },
                        { min = 0.7, max = 2.0,  minTemp = 200, maxTemp = 350 },
                        { min = 2.0, max = 10.0, minTemp = 100, maxTemp = 200 }
                    }
                }
            },
            gravity = {
                type = Enums.Gen.Rule.Range,
                min = 0.1,
                max = 2.5,
                condition = {
                    type = Enums.Gen.Condition.PlanetSize,
                    ranges = {
                        { min = 0.5, max = 2.0,  minGravity = 0.1, maxGravity = 0.8 },
                        { min = 2.0, max = 5.0,  minGravity = 0.8, maxGravity = 1.5 },
                        { min = 5.0, max = 10.0, minGravity = 1.5, maxGravity = 2.5 }
                    }
                }
            },
            rotationPeriod = {
                type = Enums.Gen.Rule.Range,
                min = 8,
                max = 100,
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        [Enums.Gen.PlanetTypes.Rocky] = { min = 8, max = 30 },
                        [Enums.Gen.PlanetTypes.GasGiant] = { min = 10, max = 20 },
                        [Enums.Gen.PlanetTypes.Icy] = { min = 20, max = 50 },
                        [Enums.Gen.PlanetTypes.Desert] = { min = 15, max = 40 }
                    }
                }
            },
            eccentricity = { type = Enums.Gen.Rule.Range, min = 0.0, max = 0.2, default = 0.0167 },
            magneticField = {
                type = Enums.Gen.Rule.Chance,
                value = 0.5,
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        Enums.Gen.PlanetTypes.Rocky,
                        Enums.Gen.PlanetTypes.GasGiant
                    }
                }
            },
            inclination = { type = Enums.Gen.Rule.Range, min = 0.0, max = 10.0, default = 0.0 }
        }
    },
    moons = {
        count = {
            type = Enums.Gen.Rule.Count,
            min = 0,
            max = 3,
            condition = { type = Enums.Gen.Condition.PlanetCount, ranges = { { min = 0, max = 4 }, { min = 5, max = 8 }, { min = 9, max = 12 } } }
        },
        aspects = {
            size = {
                type = Enums.Gen.Rule.Range,
                min = 0.05,
                max = 0.5,
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        [Enums.Gen.PlanetTypes.GasGiant] = { min = 0.1, max = 0.5 },
                        [Enums.Gen.PlanetTypes.Rocky] = { min = 0.05, max = 0.2 },
                        [Enums.Gen.PlanetTypes.Icy] = { min = 0.05, max = 0.3 },
                        [Enums.Gen.PlanetTypes.Desert] = { min = 0.05, max = 0.15 }
                    }
                }
            },
            type = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.MoonTypes.Rocky, weight = 0.6 },
                    { value = Enums.Gen.MoonTypes.Icy,   weight = 0.4 }
                },
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        GasGiant = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.5,
                                [Enums.Gen.MoonTypes.Icy] = 0.5
                            }
                        },
                        Rocky = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.8,
                                [Enums.Gen.MoonTypes.Icy] = 0.2
                            }
                        },
                        Icy = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.3,
                                [Enums.Gen.MoonTypes.Icy] = 0.7
                            }
                        },
                        Desert = {
                            weights = {
                                [Enums.Gen.MoonTypes.Rocky] = 0.9,
                                [Enums.Gen.MoonTypes.Icy] = 0.1
                            }
                        }
                    }
                }
            },
            orbitalDistance = {
                type = Enums.Gen.Rule.Range,
                min = 1e5,
                max = 1e6,
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        { min = 0.5, max = 2.0,  minDistance = 1e5, maxDistance = 3e5 },
                        { min = 2.0, max = 5.0,  minDistance = 2e5, maxDistance = 6e5 },
                        { min = 5.0, max = 10.0, minDistance = 4e5, maxDistance = 1e6 }
                    }
                }
            },
            inclination = { type = Enums.Gen.Rule.Range, min = 0.0, max = 5.0, default = 0.0 }
        }
    },
    rings = {
        count = { type = Enums.Gen.Rule.Chance, value = 0.3 },
        aspects = {
            composition = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.AsteroidRingTypes.Icy,   weight = 0.6 },
                    { value = Enums.Gen.AsteroidRingTypes.Rocky, weight = 0.4 }
                },
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        {
                            min = 0.1,
                            max = 2.0,
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Icy] = 0.4,
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.6
                            }
                        },
                        {
                            min = 2.0,
                            max = 10.0,
                            weights = {
                                [Enums.Gen.AsteroidRingTypes.Icy] = 0.7,
                                [Enums.Gen.AsteroidRingTypes.Rocky] = 0.3
                            }
                        }
                    }
                }
            },
            thickness = {
                type = Enums.Gen.Rule.Range,
                min = 10,
                max = 1000,
                condition = {
                    type = Enums.Gen.Condition.PlanetType,
                    types = {
                        [Enums.Gen.PlanetTypes.GasGiant] = { min = 100, max = 1000 },
                        [Enums.Gen.PlanetTypes.Rocky] = { min = 10, max = 200 },
                        [Enums.Gen.PlanetTypes.Icy] = { min = 50, max = 500 },
                        [Enums.Gen.PlanetTypes.Desert] = { min = 10, max = 100 }
                    }
                }
            }
        }
    },
    asteroidBelts = {
        count = { type = Enums.Gen.Rule.Chance, value = 0.7 },
        aspects = {
            density = {
                type = Enums.Gen.Rule.Range,
                min = 0.1,
                max = 1.0,
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        { min = 0.1, max = 2.0,  minDensity = 0.3, maxDensity = 1.0 },
                        { min = 2.0, max = 10.0, minDensity = 0.1, maxDensity = 0.5 }
                    }
                }
            },
            composition = {
                type = Enums.Gen.Rule.Weighted,
                values = {
                    { value = Enums.Gen.AsteroidBeltTypes.Rocky,    weight = 0.7 },
                    { value = Enums.Gen.AsteroidBeltTypes.Metallic, weight = 0.2 },
                    { value = Enums.Gen.AsteroidBeltTypes.Icy,      weight = 0.1 }
                },
                condition = {
                    type = Enums.Gen.Condition.SystemMetallicity,
                    ranges = {
                        {
                            min = 0.01,
                            max = 0.02,
                            weights = {
                                [Enums.Gen.AsteroidBeltTypes.Rocky] = 0.8,
                                [Enums.Gen.AsteroidBeltTypes.Metallic] = 0.1,
                                [Enums.Gen.AsteroidBeltTypes.Icy] = 0.1
                            }
                        },
                        {
                            min = 0.02,
                            max = 0.04,
                            weights = {
                                [Enums.Gen.AsteroidBeltTypes.Rocky] = 0.5,
                                [Enums.Gen.AsteroidBeltTypes.Metallic] = 0.4,
                                [Enums.Gen.AsteroidBeltTypes.Icy] = 0.1
                            }
                        }
                    }
                }
            },
            width = {
                type = Enums.Gen.Rule.Range,
                min = 1e7,
                max = 1e8,
                condition = {
                    type = Enums.Gen.Condition.OrbitRadius,
                    ranges = {
                        { min = 0.1, max = 2.0,  minWidth = 1e7, maxWidth = 5e7 },
                        { min = 2.0, max = 10.0, minWidth = 3e7, maxWidth = 1e8 }
                    }
                }
            }
        }
    },
    starZoneRadius = { type = Enums.Gen.Rule.Fixed, value = 1.5e11 }
}

return Ruleset
