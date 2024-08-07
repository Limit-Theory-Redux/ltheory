local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

---@class PlayerList: Component
---@overload fun(): PlayerList subclass external
local PlayerList = Subclass(Component, function(self)
    ---@cast self PlayerList
    self:setComponentName("PlayerList")

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
