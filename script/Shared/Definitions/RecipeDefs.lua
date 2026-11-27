---Production recipes: what a station's production consumes and makes in one cycle. `income` is the
---credits a cycle pays the station (a colony's earnings), the economy's source of new credits.
local Items = require("Shared.Registries.Items")
require("Shared.Definitions.ItemDefs")

---@class RecipeStack
---@field item ItemDefinition
---@field count integer

---@class RecipeDefinition
---@field name string
---@field inputs RecipeStack[]
---@field outputs RecipeStack[]
---@field duration number Simulated seconds per cycle
---@field income number|nil Credits paid to the station per cycle

---@type table<string, RecipeDefinition>
local Recipes = {
    SolarArray = {
        name = "Solar Array",
        inputs = {},
        outputs = { { item = Items.Miscellaneous.EnergyCell, count = 6 } },
        duration = 10,
    },
    IronRefinery = {
        name = "Iron Refinery",
        inputs = { { item = Items.RawMaterials.IronOre, count = 8 }, { item = Items.Miscellaneous.EnergyCell, count = 2 } },
        outputs = { { item = Items.RefinedMaterials.Iron, count = 4 } },
        duration = 15,
    },
    SteelFoundry = {
        name = "Steel Foundry",
        inputs = { { item = Items.RefinedMaterials.Iron, count = 3 }, { item = Items.Miscellaneous.EnergyCell, count = 2 } },
        outputs = { { item = Items.RefinedMaterials.Steel, count = 2 } },
        duration = 20,
    },
    OutpostUpkeep = {
        name = "Outpost Upkeep",
        inputs = { { item = Items.Miscellaneous.EnergyCell, count = 3 } },
        outputs = {},
        duration = 15,
        income = 150,
    },
    Colony = {
        name = "Colony",
        inputs = { { item = Items.RefinedMaterials.Steel, count = 2 }, { item = Items.Miscellaneous.EnergyCell, count = 4 } },
        outputs = {},
        duration = 30,
        income = 600,
    },
}

return Recipes
