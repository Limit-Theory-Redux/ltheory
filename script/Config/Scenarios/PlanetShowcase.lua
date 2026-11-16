-- Planet Showcase: one planet to look at, different per seed: its size, type
-- and colours, one to three moons, and a ring for about a third of them
-- (Rulesets.PlanetShowcase). The camera starts orbiting it; Regenerate (B)
-- makes a new one. (Was the PlanetTest state.)

local Rulesets = require("Config.Gen.Rulesets")
local PhysicsComponents = require("Modules.Physics.Components")
local Overlay = require("Config.Scenarios.Overlay")

return {
    name = "Planet Showcase",
    description = "One planet up close, with moons and sometimes a ring; B makes a new one.",
    ruleset = Rulesets.PlanetShowcase,
    stations = false,

    setup = function(game)
        local planet = game.planets[1]
        if planet then
            local radius = planet:get(PhysicsComponents.Transform):getScale()
            game:setOrbitFocus(planet, game:orbitDistances(radius))
        end
    end,

    controls = {
        { keys = "B", action = "New planet" },
    },

    drawOverlay = function(game)
        Overlay.draw({
            "Planet Showcase",
            "B: new planet    C: camera / fly    Right mouse: orbit    Wheel: zoom",
        })
    end,
}
