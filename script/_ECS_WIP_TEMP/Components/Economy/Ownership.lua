local Component = require('Component')

---@class Ownership: Component
---@overload fun(self: Ownership, playerId: integer|nil): Ownership subclass internal
---@overload fun(playerId: integer|nil): Ownership subclass external
local Ownership = Subclass(Component, function(self, playerId)
    self:setComponentName("EconomyOwnership")
    self:setOwner(playerId)
end)

---@param playerId integer|nil
function Ownership:setOwner(playerId)
    self.owner = playerId
end

---@return integer playerId
function Ownership:getOwner()
    return self.owner
end

return Ownership
