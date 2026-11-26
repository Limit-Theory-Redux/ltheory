-- Economy: stations around the planet that produce, consume and trade (a colony, a power plant, a
-- refinery and a foundry), an ore field above the planet's ring, and AI traders and miners that
-- fly and dock between them. The HUD lists every station's credits and stock and what every ship
-- is doing. Design: doc/script/economy.md

local Rulesets          = require("Config.Gen.Rulesets")
local PhysicsComponents = require("Modules.Physics.Components")
local SpawnerSystem     = require("Modules.Spawning.Systems.SpawnerSystem")
local Helpers           = require("Config.Scenarios.Helpers")
local EconomyOverlay    = require("Config.Scenarios.EconomyOverlay")

-- Where the stations are around the first planet: an angle in degrees in the planet's equatorial
-- plane and a distance in planet radii from its centre, at a height over that plane clear of the
-- ring (in planet radii).
local STATIONS = {
    { kind = "Colony", name = "Colony", angleDeg = 0, radii = 2.6 },
    { kind = "PowerPlant", name = "Power Plant", angleDeg = 75, radii = 3.2 },
    { kind = "Refinery", name = "Refinery", angleDeg = 150, radii = 3.0 },
    { kind = "Foundry", name = "Foundry", angleDeg = 230, radii = 2.8 },
}
local HEIGHT_RADII = 1.2
-- The ore field, by the refinery; its sizes are in game units (a unit is 10 km).
local ORE_AT = { angleDeg = 190, radii = 3.4 }
local ORE = { halfExtent = 1.0, count = 8, minRadius = 0.03, maxRadius = 0.06, gap = 0.1, quantity = 400, regrowPerSecond = 0.5 }
-- The player starts this far from the colony, in game units.
local SPAWN_OFFSET = 3

---A point by the first planet, at an angle in its equatorial plane and a distance from its centre
---in planet radii, `HEIGHT_RADII` over that plane.
---@param game table
---@param angleDeg number
---@param radii number
---@return Position
local function aroundPlanet(game, angleDeg, radii)
    local planet = game.planets[1]
    local centre, radius = planet:get(PhysicsComponents.Transform):getPos(), Helpers.radius(planet)
    local angle = math.rad(angleDeg)
    return Position(centre.x + math.cos(angle) * radii * radius, centre.y + HEIGHT_RADII * radius,
        centre.z + math.sin(angle) * radii * radius)
end

return {
    name = "Economy",
    description = "Stations produce and trade around the planet; AI traders and miners dock between them.",
    ruleset = Rulesets.SinglePlanetWithBelt,
    stations = false,
    -- How many agents of each role the scenario spawns.
    traders = 4,
    miners = 3,

    spawnShip = function(game)
        local colony = aroundPlanet(game, STATIONS[1].angleDeg, STATIONS[1].radii)
        return Position(colony.x, colony.y + SPAWN_OFFSET * 0.3, colony.z + SPAWN_OFFSET)
    end,

    setup = function(game)
        local colony
        for _, spec in ipairs(STATIONS) do
            local station = SpawnerSystem:spawnStation({
                kind = spec.kind, name = spec.name, position = aroundPlanet(game, spec.angleDeg, spec.radii),
            })
            colony = colony or station
        end
        local ore = { centre = aroundPlanet(game, ORE_AT.angleDeg, ORE_AT.radii) }
        for key, value in pairs(ORE) do ore[key] = value end
        SpawnerSystem:spawnOreField(ore)
        SpawnerSystem:spawnAgentsAround(colony, {
            [Enums.EconomyAgentRole.Trader] = game.scenario.traders,
            [Enums.EconomyAgentRole.Miner] = game.scenario.miners,
        })
    end,
    overlayLines = EconomyOverlay,
}
