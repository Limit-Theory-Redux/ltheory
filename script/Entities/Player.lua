local Entity = require("Entities.Entity")
local Components = require("Components")

---@param name string
---@param isAiPlayer boolean
---@return Entity
local function Player(name, isAiPlayer)
    local startCredits = isAiPlayer and Config.econ.eStartCredits or Config.econ.pStartCredits
    
    return Entity(
        "Player",
        Components.NameComponent(name),
        Components.PlayerBankAccountComponent(startCredits)
    )
end

return Player
