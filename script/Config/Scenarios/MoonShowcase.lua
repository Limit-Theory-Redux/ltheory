-- Moon Showcase: a generated moon next to the player's ship, made by the same
-- code as the star systems' moons. The camera starts orbiting it;
-- Regenerate (B) makes a new one. (Was the MoonTest state.)

local CelestialEntities = require("Modules.CelestialObjects.Entities")
local SolarSystemVisualizer = require("Modules.CelestialObjects.Managers.SolarSystemVisualizer")
local PhysicsComponents = require("Modules.Physics.Components")
local Helpers = require("Config.Scenarios.Helpers")
local Overlay = require("Config.Scenarios.Overlay")

local function spawnMoon(game)
    -- About the Moon's size next to the planet (Earth-like in the default system)
    local planet = game.planets[1]
    local radius = planet and planet:get(PhysicsComponents.Transform):getScale() * 0.27 or 2000

    local moon = CelestialEntities.Moon(game.rng:get64())
    local transform = moon:get(PhysicsComponents.Transform)
    transform:setScale(radius)
    transform:setPos(Helpers.outward(game, radius * 4))
    SolarSystemVisualizer:materializeEntity(moon, game.world)

    game:addSceneEntity(moon, { label = "Moon", bodyAdded = true })
    game.scenarioData.moon = moon
    game:setOrbitFocus(moon, game:orbitDistances(radius))
end

return {
    name = "Moon Showcase",
    description = "A generated moon up close; B makes a new one.",
    stations = false,

    setup = spawnMoon,

    regenerate = function(game)
        game:removeSceneEntity(game.scenarioData.moon)
        spawnMoon(game)
    end,

    drawOverlay = function(game)
        Overlay.draw({
            "Moon Showcase",
            "B: new moon    C: camera / fly    Right mouse: orbit    Wheel: zoom",
        })
    end,
}
