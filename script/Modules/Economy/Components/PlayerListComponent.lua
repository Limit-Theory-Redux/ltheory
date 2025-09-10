local Component = require("Core.ECS.Component")

---@class PlayerListComponent: Component
---@overload fun(self: PlayerList): PlayerList subclass internal
---@overload fun(): PlayerList subclass external
local PlayerListComponent = Subclass("PlayerListComponent", Component, function(self)
    self:setComponentName("EconomyPlayerList")

    self:addPlayerList()
end)

function PlayerListComponent:addPlayerList()
    self.players = {}
end

---@return table<PlayerEntity>
function PlayerListComponent:getPlayerList()
    return self.players
end

---@return integer
function PlayerListComponent:getPlayerCount()
    return #self.players
end

return PlayerListComponent
