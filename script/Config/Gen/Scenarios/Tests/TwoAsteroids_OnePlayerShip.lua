local Items = require("Shared.Registries.Items")

local CelestialObjects = require("Modules.CelestialObjects.Entities")
local Constructs = require("Modules.Constructs.Entities")

local CoreComponents = require("Modules.Core.Components")

return {
    name = "Test: 2 Asteroids + Player Ship",

    seed = 1,

    rules = {
        starSystems = { type = Enums.Gen.Rule.Fixed, value = 1 },
        planets     = { type = Enums.Gen.Rule.Fixed, value = 0 },
        stations    = { type = Enums.Gen.Rule.Fixed, value = 0 },
        ships       = { type = Enums.Gen.Rule.Fixed, value = 0 },
    },

    ---@type table<string, table>
    overrides = {
        starSystem = {
            {
                type = CelestialObjects.Asteroid(1, Items.RawMaterials.SilicateOre, 50000),
                position = Position(10000, -5000, 0)
            },
            {
                type = CelestialObjects.Asteroid(1, Items.RawMaterials.IronOre, 50000),
                position = Position(-8000, 3000, 0)
            },
            {
                type = Constructs.Spaceship(1),
                position = Position(1000, 2000, 0),
            },
        },
    },
}
