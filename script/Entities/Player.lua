local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local PlayerBankAccount = require("Components.Economy.PlayerBankAccountComponent")

---@class Player: Entity
---@overload fun(self: Player, name: string, isAiPlayer: boolean) subclass internal
---@overload fun(name: string, isAiPlayer: boolean) subclass external
local Player = Subclass("Player", Entity, function(self, name, isAiPlayer)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.PlayerEntity)

    -- Name Component
    self:addComponent(NameComponent(name))

    -- Bank Account Component
    local startCredits = isAiPlayer and Config.econ.eStartCredits or Config.econ.pStartCredits
    self:addComponent(PlayerBankAccount(startCredits))

    -- AI Component
    --self:addComponent()
end)

return Player
