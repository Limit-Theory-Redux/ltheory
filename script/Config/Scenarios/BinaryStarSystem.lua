-- Two stars and their planets. Known issue: the stars are placed far outside
-- the camera's range and only one lights the scene.
local Rulesets = require("Config.Gen.Rulesets")

return {
    name = "Binary Star System",
    description = "Two stars and their planets (the stars aren't drawn yet).",
    ruleset = Rulesets.StandardBinarySolarSystem,
}
