local Entity = require("Core.ECS.Entity")
local Components = require("Components")

---@param name string
---@param isAiPlayer boolean
---@return Entity
local function Player(name, isAiPlayer)
    local startCredits = isAiPlayer and Config.econ.eStartCredits or Config.econ.pStartCredits

    return Entity(
        name,
        Components.PlayerBankAccountComponent(startCredits)
    )
end

return Player
