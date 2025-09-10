local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")

---@class PlayerEntity: Entity
---@param name string
---@param isAiPlayer boolean
---@return Entity
return function(name, isAiPlayer)
    local startCredits = isAiPlayer and Config.econ.eStartCredits or Config.econ.pStartCredits
    return Entity(
        name,
        Economy.PlayerBankAccount(startCredits)
    )
end
