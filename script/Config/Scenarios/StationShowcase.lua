-- Station Showcase: a generated station next to the player's ship. The camera
-- starts orbiting it; Regenerate (B) builds a new one. (Was the StationTest
-- state.)

local StationGenerator = require("Modules.Constructs.Managers.Generators.StationGenerator")
local PhysicsComponents = require("Modules.Physics.Components")
local Helpers = require("Config.Scenarios.Helpers")

local STATION_SCALE = 0.3 -- the game's stations (about 3 km)

local function spawnStation(game)
    local station = StationGenerator:createStation(game.rng:get31(), {
        position = Position(0, 0, 0),
        scale = STATION_SCALE,
        isKinematic = true,
    })
    -- Placed once its size is known: three radii beyond the ship
    local radius = Helpers.radius(station)
    station:get(PhysicsComponents.RigidBody):getRigidBody():setPos(Helpers.outward(game, radius * 3))
    game:addSceneEntity(station, { label = "Station", addBody = true })
    game.scenarioData.station = station
    game:setOrbitFocus(station, game:orbitDistances(radius))
end

return {
    name = "Station Showcase",
    description = "A generated station up close; B builds a new one.",
    stations = false,

    setup = spawnStation,

    regenerate = function(game)
        game:removeSceneEntity(game.scenarioData.station)
        spawnStation(game)
    end,

    controls = {
        { keys = "B", action = "New station" },
    },

    overlayLines = function(game)
        return {
            "Station Showcase",
            "B: new station    C: camera / fly    Right mouse: orbit    Wheel: zoom",
        }
    end,
}
