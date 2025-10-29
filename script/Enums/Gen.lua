Enums.Gen = {}

---@enum GenRule
Enums.Gen.Rule = {
    Count    = 1,
    Chance   = 2,
    Fixed    = 3,
    Range    = 4,
    Weighted = 5,
    Custom   = 6
}

---@enum GenCondition
Enums.Gen.Condition = {
    SystemAge = 1,
    StarType = 2,
    StarMass = 3,
    OrbitRadius = 4,
    PlanetType = 5,
    PlanetSize = 6,
    PlanetCount = 7,
    SystemMetallicity = 8
}

Enums.Gen.StarSystem = {
    ---@enum GenStarSystemStability
    Stability = {
        Stable = 1,
        Unstable = 2,
        Chaotic = 3
    }
}

---@enum GenStarTypes
Enums.Gen.StarTypes = {
    MainSequence = 1,
    RedGiant = 2,
    WhiteDwarf = 3
}

---@enum GenPlanetTypes
Enums.Gen.PlanetTypes = {
    Rocky = 1,
    GasGiant = 2,
    Icy = 3,
    Desert = 4
}

---@enum GenMoonTypes
Enums.Gen.MoonTypes = {
    Rocky = 1,
    Icy = 2
}

---@enum GenAsteroidRingTypes
Enums.Gen.AsteroidRingTypes = {
    None = 1,
    Rocky = 2,
    Icy = 3
}

---@enum GenAsteroidBeltTypes
Enums.Gen.AsteroidBeltTypes = {
    Rocky = 1,
    Metallic = 2,
    Icy = 3
}
