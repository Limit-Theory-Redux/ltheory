local Component = require('Component')

---@class PlayerList: Component
---@overload fun(): PlayerList subclass external
local PlayerList = Subclass(Component, function(self)
    ---@cast self PlayerList
    self:setComponentName("PlayerList")
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
