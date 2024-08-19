local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

---@class PlayerList: Component
---@overload fun(self: PlayerList): PlayerList subclass internal
---@overload fun(): PlayerList subclass external
local PlayerList = Subclass(Component, function(self)
    self:setComponentName("EconomyPlayerList")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.PlayerListComponent)

    self:addPlayerList()
end)

function PlayerList:addPlayerList()
    self.players = {}
end

---@return table<Player>
function PlayerList:getPlayerList()
    return self.players
end

---@return integer
function PlayerList:getPlayerCount()
    return #self.players
end

return PlayerList
