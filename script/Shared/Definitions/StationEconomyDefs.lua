---What a station of each economic kind runs and starts with: its recipes, its credits and the stock
---in its inventory. `EconomySetup.equipStation` applies one to a station entity.
local Items = require("Shared.Registries.Items")
local Recipes = require("Shared.Definitions.RecipeDefs")

---@class StationEconomyDefinition
---@field name string
---@field recipes RecipeDefinition[]
---@field credits number
---@field stock { item: ItemDefinition, count: integer }[]

---@type table<string, StationEconomyDefinition>
local StationEconomies = {
    PowerPlant = {
        name = "Power Plant",
        recipes = { Recipes.SolarArray, Recipes.SolarArray },
        credits = 20000,
        stock = { { item = Items.Miscellaneous.EnergyCell, count = 60 } },
    },
    Refinery = {
        name = "Refinery",
        recipes = { Recipes.IronRefinery },
        credits = 40000,
        stock = { { item = Items.RawMaterials.IronOre, count = 40 }, { item = Items.Miscellaneous.EnergyCell, count = 10 } },
    },
    Foundry = {
        name = "Foundry",
        recipes = { Recipes.SteelFoundry },
        credits = 40000,
        stock = { { item = Items.RefinedMaterials.Iron, count = 12 }, { item = Items.Miscellaneous.EnergyCell, count = 10 } },
    },
    Colony = {
        name = "Colony",
        recipes = { Recipes.Colony },
        credits = 60000,
        stock = { { item = Items.RefinedMaterials.Steel, count = 6 }, { item = Items.Miscellaneous.EnergyCell, count = 12 } },
    },
}

return StationEconomies
