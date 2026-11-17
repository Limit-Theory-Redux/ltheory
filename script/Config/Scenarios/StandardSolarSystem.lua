-- A star system that differs per seed: several planets, moons and rings,
-- (Was the SolarSystemPlayable test state, which also ran the orbits: the
-- planets left the player's ship behind within seconds.)
local Rulesets = require("Config.Gen.Rulesets")

return {
    name = "Standard Solar System",
    description = "Several planets, moons and rings, different per seed.",
    ruleset = Rulesets.StandardSolarSystem,
}
