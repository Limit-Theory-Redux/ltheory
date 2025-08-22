local Entity = require("Core.ECS.Entity")

---@class PlayerEntity: Entity
---@param name string
---@param isAiPlayer boolean
return function(name, isAiPlayer)
    local Economy = require("Modules.Economy")

    local startCredits = isAiPlayer and Config.econ.eStartCredits or Config.econ.pStartCredits
    return Entity(
        name,
        Economy.Components.PlayerBankAccount(startCredits)
    )
end
