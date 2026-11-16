-- Ship Showcase: the player's own ship to look at. The camera starts orbiting
-- it; Regenerate (B) builds a new hull of the next type. C cycles the cameras
-- as always, so the hull can be flown too. (Was the ShipTest state.)

local Helpers = require("Config.Scenarios.Helpers")
local Overlay = require("Config.Scenarios.Overlay")

-- Ship type and its size (index into Config.game.shipHulls)
local HULLS = {
    { name = "Fighter", shipType = Enums.ShipType.Fighter, hull = 1 },
    { name = "Basic",   shipType = Enums.ShipType.Basic,   hull = 3 },
    { name = "Capital", shipType = Enums.ShipType.Capital, hull = 6 },
}

local function focus(game)
    game:setOrbitFocus(game.playerShip, game:orbitDistances(Helpers.radius(game.playerShip)))
end

return {
    name = "Ship Showcase",
    description = "Your ship up close; B builds a new hull (fighter, basic, capital).",

    setup = function(game)
        game.scenarioData.hull = 1 -- the game builds a fighter
        focus(game)
    end,

    regenerate = function(game)
        local data = game.scenarioData
        data.hull = data.hull % #HULLS + 1
        local hull = HULLS[data.hull]
        game:replacePlayerShip(hull.shipType, hull.hull, game.rng:get31())
        focus(game)
    end,

    controls = {
        { keys = "B", action = "Next hull (fighter, basic, capital)" },
    },

    drawOverlay = function(game)
        Overlay.draw({
            "Ship Showcase: " .. HULLS[game.scenarioData.hull].name .. " hull",
            "B: next hull    C: camera / fly    Right mouse: orbit    Wheel: zoom",
        })
    end,
}
