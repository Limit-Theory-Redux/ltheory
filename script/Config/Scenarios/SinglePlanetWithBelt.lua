-- The game's default start: one star, an Earth-like planet with a ring of
-- rocks and a moon. The player's ship starts next to a rock in the ring.
local Rulesets = require("Config.Gen.Rulesets")

return {
    name = "Single Planet With Belt",
    description = "One star, an Earth-like planet with a ring and a moon.",
    ruleset = Rulesets.SinglePlanetWithBelt,
}
