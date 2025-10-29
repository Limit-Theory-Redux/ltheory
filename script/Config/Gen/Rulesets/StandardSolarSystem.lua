local Ruleset = {
    name = "StandardSolarSystem",
    starSystems = {
        count = { type = "Fixed", value = 1 },
        aspects = {
            age = {
                type = "Weighted",
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
                type = "Range",
                min = 0.01,
                max = 0.04,
                default = 0.02
            },
            stability = {
                type = "Weighted",
                values = {
                    { value = Enums.Gen.StarSystem.Stability.Stable,   weight = 0.7 },
                    { value = Enums.Gen.StarSystem.Stability.Unstable, weight = 0.2 },
                    { value = Enums.Gen.StarSystem.Stability.Chaotic,  weight = 0.1 }
                },
                condition = {
                    type = "SystemAge",
                    ranges = {
                        { min = 1e9, max = 3e9,  weights = { Stable = 0.5, Unstable = 0.3, Chaotic = 0.2 } },
                        { min = 3e9, max = 10e9, weights = { Stable = 0.8, Unstable = 0.15, Chaotic = 0.05 } }
                    }
                }
            }
        }
    },
    stars = {
        count = { type = "Fixed", value = 1 },
        aspects = {
            type = {
                type = "Fixed",
                value = "MainSequence",
                condition = {
                    type = "SystemAge",
                    ranges = {
                        { min = 1e9, max = 5e9,  value = "MainSequence" },
                        { min = 5e9, max = 8e9,  value = "RedGiant" },
                        { min = 8e9, max = 10e9, value = "WhiteDwarf" }
                    }
                }
            },
            mass = {
                type = "Range",
                min = 0.8,
                max = 2.0,
                condition = {
                    type = "StarType",
                    types = {
                        MainSequence = { min = 0.8, max = 1.5 },
                        RedGiant = { min = 1.0, max = 2.0 },
                        WhiteDwarf = { min = 0.8, max = 1.2 }
                    }
                }
            },
            luminosity = {
                type = "Range",
                min = 0.1,
                max = 100,
                condition = {
                    type = "StarMass",
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
            type = "Weighted",
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
            orbitRadius = { type = "Range", min = 0.1, max = 10.0 },
            size = {
                type = "Range",
                min = 0.5,
                max = 10.0,
                condition = {
                    type = "OrbitRadius",
                    ranges = {
                        { min = 0.1, max = 0.7,  minSize = 0.5, maxSize = 2.0 },
                        { min = 0.7, max = 2.0,  minSize = 1.0, maxSize = 5.0 },
                        { min = 2.0, max = 10.0, minSize = 5.0, maxSize = 10.0 }
                    }
                }
            },
            type = {
                type = "Weighted",
                values = {
                    { value = "Rocky",    weight = 0.5 },
                    { value = "GasGiant", weight = 0.3 },
                    { value = "Ice",      weight = 0.15 },
                    { value = "Desert",   weight = 0.05 }
                },
                condition = {
                    type = "OrbitRadius",
                    ranges = {
                        { min = 0.1, max = 0.7,  weights = { Rocky = 0.8, Desert = 0.2 } },
                        { min = 0.7, max = 2.0,  weights = { Rocky = 0.5, GasGiant = 0.3, Ice = 0.2 } },
                        { min = 2.0, max = 10.0, weights = { GasGiant = 0.6, Ice = 0.4 } }
                    }
                }
            },
            atmosphere = {
                type = "Chance",
                value = 0.6,
                condition = { type = "PlanetType", types = { "Rocky", "GasGiant", "Ice" } }
            },
            asteroidRing = {
                type = "Weighted",
                values = {
                    { value = "None",  weight = 0.7 },
                    { value = "Rocky", weight = 0.2 },
                    { value = "Icy",   weight = 0.1 }
                },
                condition = {
                    type = "PlanetType",
                    types = {
                        GasGiant = { weights = { None = 0.4, Rocky = 0.3, Icy = 0.3 } },
                        Rocky = { weights = { None = 0.9, Rocky = 0.08, Icy = 0.02 } },
                        Ice = { weights = { None = 0.7, Rocky = 0.1, Icy = 0.2 } },
                        Desert = { weights = { None = 0.95, Rocky = 0.05 } }
                    }
                }
            },
            temperature = {
                type = "Range",
                min = 100,
                max = 500,
                condition = {
                    type = "OrbitRadius",
                    ranges = {
                        { min = 0.1, max = 0.7,  minTemp = 300, maxTemp = 500 },
                        { min = 0.7, max = 2.0,  minTemp = 200, maxTemp = 350 },
                        { min = 2.0, max = 10.0, minTemp = 100, maxTemp = 200 }
                    }
                }
            },
            gravity = {
                type = "Range",
                min = 0.1,
                max = 2.5,
                condition = {
                    type = "PlanetSize",
                    ranges = {
                        { min = 0.5, max = 2.0,  minGravity = 0.1, maxGravity = 0.8 },
                        { min = 2.0, max = 5.0,  minGravity = 0.8, maxGravity = 1.5 },
                        { min = 5.0, max = 10.0, minGravity = 1.5, maxGravity = 2.5 }
                    }
                }
            },
            rotationPeriod = {
                type = "Range",
                min = 8,
                max = 100,
                condition = {
                    type = "PlanetType",
                    types = {
                        Rocky = { min = 8, max = 30 },
                        GasGiant = { min = 10, max = 20 },
                        Ice = { min = 20, max = 50 },
                        Desert = { min = 15, max = 40 }
                    }
                }
            },
            eccentricity = { type = "Range", min = 0.0, max = 0.2, default = 0.0167 },
            magneticField = {
                type = "Chance",
                value = 0.5,
                condition = { type = "PlanetType", types = { "Rocky", "GasGiant" } }
            },
            inclination = { type = "Range", min = 0.0, max = 10.0, default = 0.0 }
        }
    },
    moons = {
        count = {
            type = "Count",
            min = 0,
            max = 3,
            condition = { type = "PlanetCount", ranges = { { min = 0, max = 4 }, { min = 5, max = 8 }, { min = 9, max = 12 } } }
        },
        aspects = {
            size = {
                type = "Range",
                min = 0.05,
                max = 0.5,
                condition = {
                    type = "PlanetType",
                    types = {
                        GasGiant = { min = 0.1, max = 0.5 },
                        Rocky = { min = 0.05, max = 0.2 },
                        Ice = { min = 0.05, max = 0.3 },
                        Desert = { min = 0.05, max = 0.15 }
                    }
                }
            },
            type = {
                type = "Weighted",
                values = {
                    { value = "Rocky", weight = 0.6 },
                    { value = "Icy",   weight = 0.4 }
                },
                condition = {
                    type = "PlanetType",
                    types = {
                        GasGiant = { weights = { Rocky = 0.5, Icy = 0.5 } },
                        Rocky = { weights = { Rocky = 0.8, Icy = 0.2 } },
                        Ice = { weights = { Rocky = 0.3, Icy = 0.7 } },
                        Desert = { weights = { Rocky = 0.9, Icy = 0.1 } }
                    }
                }
            },
            orbitalDistance = {
                type = "Range",
                min = 1e5,
                max = 1e6,
                condition = {
                    type = "PlanetSize",
                    ranges = {
                        { min = 0.5, max = 2.0,  minDistance = 1e5, maxDistance = 3e5 },
                        { min = 2.0, max = 5.0,  minDistance = 2e5, maxDistance = 6e5 },
                        { min = 5.0, max = 10.0, minDistance = 4e5, maxDistance = 1e6 }
                    }
                }
            },
            inclination = { type = "Range", min = 0.0, max = 5.0, default = 0.0 }
        }
    },
    rings = {
        count = { type = "Chance", value = 0.3 },
        aspects = {
            composition = {
                type = "Weighted",
                values = {
                    { value = "Ice",  weight = 0.6 },
                    { value = "Rock", weight = 0.4 }
                },
                condition = {
                    type = "OrbitRadius",
                    ranges = {
                        { min = 0.1, max = 2.0,  weights = { Ice = 0.4, Rock = 0.6 } },
                        { min = 2.0, max = 10.0, weights = { Ice = 0.7, Rock = 0.3 } }
                    }
                }
            },
            thickness = {
                type = "Range",
                min = 10,
                max = 1000,
                condition = {
                    type = "PlanetType",
                    types = {
                        GasGiant = { min = 100, max = 1000 },
                        Rocky = { min = 10, max = 200 },
                        Ice = { min = 50, max = 500 },
                        Desert = { min = 10, max = 100 }
                    }
                }
            }
        }
    },
    asteroidBelts = {
        count = { type = "Chance", value = 0.7 },
        aspects = {
            density = {
                type = "Range",
                min = 0.1,
                max = 1.0,
                condition = {
                    type = "OrbitRadius",
                    ranges = {
                        { min = 0.1, max = 2.0,  minDensity = 0.3, maxDensity = 1.0 },
                        { min = 2.0, max = 10.0, minDensity = 0.1, maxDensity = 0.5 }
                    }
                }
            },
            composition = {
                type = "Weighted",
                values = {
                    { value = "Rocky",    weight = 0.7 },
                    { value = "Metallic", weight = 0.2 },
                    { value = "Icy",      weight = 0.1 }
                },
                condition = {
                    type = "SystemMetallicity",
                    ranges = {
                        { min = 0.01, max = 0.02, weights = { Rocky = 0.8, Metallic = 0.1, Icy = 0.1 } },
                        { min = 0.02, max = 0.04, weights = { Rocky = 0.5, Metallic = 0.4, Icy = 0.1 } }
                    }
                }
            },
            width = {
                type = "Range",
                min = 1e7,
                max = 1e8,
                condition = {
                    type = "OrbitRadius",
                    ranges = {
                        { min = 0.1, max = 2.0,  minWidth = 1e7, maxWidth = 5e7 },
                        { min = 2.0, max = 10.0, minWidth = 3e7, maxWidth = 1e8 }
                    }
                }
            }
        }
    },
    starZoneRadius = { type = "Fixed", value = 1.5e11 }
}

return Ruleset
