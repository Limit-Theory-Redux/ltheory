local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

---@class PlayerListComponent: Component
---@overload fun(self: PlayerList): PlayerList subclass internal
---@overload fun(): PlayerList subclass external
local PlayerListComponent = Subclass(Component, function(self)
    self:setComponentName("EconomyPlayerList")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.PlayerListComponent)

    self:addPlayerList()
end)

function PlayerListComponent:addPlayerList()
    self.players = {}
end

---@return table<Player>
function PlayerListComponent:getPlayerList()
    return self.players
end

---@return integer
function PlayerListComponent:getPlayerCount()
    return #self.players
end

return PlayerListComponent
