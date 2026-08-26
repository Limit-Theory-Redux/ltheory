Enums.Gen = {}

---@enum GenRule
Enums.Gen.Rule = {
    Count    = 1,
    Chance   = 2,
    Fixed    = 3,
    Range    = 4,
    Weighted = 5,
    ByType   = 6,
    Custom   = 7
}

---@enum GenCondition
Enums.Gen.Condition = {
    SystemAge = 1,
    StarCount = 2,
    StarType = 3,
    StarMass = 4,
    StarIndex = 5,
    OrbitRadius = 6,
    PlanetType = 7,
    PlanetSize = 8,
    PlanetCount = 9,
    SystemMetallicity = 10,
    Combined = 100
}

Enums.Gen.StarSystem = {
    ---@enum GenStarSystemStability
    Stability = {
        Stable = "Stable",
        Unstable = "Unstable",
        Chaotic = "Chaotic"
    }
}

Enums.Gen.StarSystemTypes = {
    Single = "Single",
    Binary = "Binary",
    Trinary = "Trinary"
}

---@enum GenStarTypes
Enums.Gen.StarTypes = {
    MainSequence = "MainSequence",
    RedGiant = "RedGiant",
    WhiteDwarf = "WhiteDwarf"
}

---@enum GenPlanetTypes
Enums.Gen.PlanetTypes = {
    Rocky = "Rocky",
    GasGiant = "GasGiant",
    Icy = "Icy",
    Desert = "Desert"
}

---@enum GenMoonTypes
Enums.Gen.MoonTypes = {
    Rocky = "Rocky",
    Icy = "Icy"
}

---@enum GenAsteroidRingTypes
Enums.Gen.AsteroidRingTypes = {
    None = "None",
    Rocky = "Rocky",
    Icy = "Icy"
}

---@enum GenAsteroidBeltTypes
Enums.Gen.AsteroidBeltTypes = {
    Rocky = "Rocky",
    Metallic = "Metallic",
    Icy = "Icy"
}

---@enum GenAsteroidTypes
Enums.Gen.AsteroidTypes = {
    Resource = "Resource"
}
