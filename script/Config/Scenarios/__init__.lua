-- Scenarios: how a game starts. New Game lists them (in this order) and the
-- game state (LTheoryRedux) builds and runs the one picked: a star system from
-- a ruleset, the player's ship, the stations, and whatever the scenario adds.
-- After that it is the normal game: the player's ship and cameras, the HUD,
-- the pause menu, the maps.
--
-- The startup config loader skips this folder (like Rulesets): scenarios hold
-- code and are only loaded through this list.
--
-- A scenario is a table; every field is optional except `name`:
--
--   name         shown in the New Game list
--   description  one line under the list
--   ruleset      the star system (default: Rulesets.SinglePlanetWithBelt)
--   orbits       planets and moons move along their orbits (default: false)
--   stations     the stations near planets (default: true)
--   spawnShip(game)        where the player's ship starts: a Position, or nil
--                          for the default (next to the belt rock nearest the
--                          first planet)
--   setup(game)            spawns the scenario's own things; runs after the
--                          star system, the player's ship and the stations
--   update(game, dt)       each simulation step, before the physics step
--   postUpdate(game, dt)   each simulation step, after the physics step
--   onInput(game, dt)      the scenario's own keys (not while paused or in a map)
--   regenerate(game)       what Regenerate (B) does (default: a new seed, and
--                          everything is built again)
--   drawOverlay(game)      text over the game, with the HUD
--   teardown(game)         undo what setup changed outside the scene (things
--                          added with `game:addSceneEntity` are removed anyway)
--
-- `game` is the game state. What a scenario uses of it:
--   game.world, game.constructManager, game.seed, game.rng, game.universe,
--   game.planets, game.starEntity, game.playerShip, game.player (the
--   PlayerController), game.scenarioData (a table of the scenario's own, new
--   for every start), game:addSceneEntity(entity, label),
--   game:removeSceneEntity(entity), game:setOrbitFocus(entity, distances),
--   game:regenerate().

---@class Scenario
---@field id string the file name, set by this list
---@field name string
---@field description string|nil
---@field ruleset Ruleset|nil
---@field orbits boolean|nil
---@field stations boolean|nil
---@field spawnShip fun(game: table): Position|nil
---@field setup fun(game: table)|nil
---@field update fun(game: table, dt: number)|nil
---@field postUpdate fun(game: table, dt: number)|nil
---@field onInput fun(game: table, dt: number)|nil
---@field regenerate fun(game: table)|nil
---@field drawOverlay fun(game: table)|nil
---@field teardown fun(game: table)|nil

local ids = {
    "SinglePlanetWithBelt",
    "StandardSolarSystem",
    "BinaryStarSystem",
    "TrinaryStarSystem",
    "ShipShowcase",
    "StationShowcase",
    "MoonShowcase",
    "PlanetShowcase",
    "PhysicsTest",
    "CameraTest",
}

---@type Scenario[]
local Scenarios = {}
for i, id in ipairs(ids) do
    local scenario = require("Config.Scenarios." .. id)
    scenario.id = id
    Scenarios[i] = scenario
end

---The scenario New Game starts with, and the menu background's star system.
Scenarios.default = Scenarios[1]

---The scenario with this id (its file name), or nil.
---@param id string|nil
---@return Scenario|nil
function Scenarios.find(id)
    for _, scenario in ipairs(Scenarios) do
        if scenario.id == id then
            return scenario
        end
    end
    return nil
end

return Scenarios
