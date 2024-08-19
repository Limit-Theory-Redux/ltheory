local Entity = require("_ECS_WIP_TEMP.Entity")

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.EntityName")
local TypeComponent = require("_ECS_WIP_TEMP.Components.EntityType")
local PlayerBankAccount = require("_ECS_WIP_TEMP.Components.PlayerBankAccount")

---@class Player: Entity
---@overload fun(self: table, name: string, isAiPlayer: boolean) subclass interal
---@overload fun(name: string, isAiPlayer: boolean) subclass external
local Player = Subclass(Entity, function(self, name, isAiPlayer)
    ---@cast self Player

    -- Name Component
    self:addComponent(NameComponent(name))
    -- Type Component
    self:addComponent(TypeComponent(Enums.EntityType.Player))

    -- Bank Account Component
    local startCredits = isAiPlayer and Config.econ.eStartCredits or Config.econ.pStartCredits
    self:addComponent(PlayerBankAccount(startCredits))

    -- AI Component
    self:addComponent()
end)

return Player
