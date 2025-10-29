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
    SystemMetallicity = 7
}

Enums.Gen.StarSystem = {
    ---@enum GenStarSystemStability
    Stability = {
        Stable = 1,
        Unstable = 2,
        Chaotic = 3
    }
}
