local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")
local Factions = require("Modules.Factions.Components")

---A faction: its name, kind and colour, its relations, and its treasury.
---@param name string
---@param factionType integer An `Enums.FactionType`
---@param color Color|nil
---@param treasury number|nil Credits
---@return Entity
return function(name, factionType, color, treasury)
    return Entity.Create(
        name,
        Factions.Faction(name, factionType, color),
        Factions.FactionRelations(),
        Economy.PlayerBankAccount(treasury or 0)
    )
end
