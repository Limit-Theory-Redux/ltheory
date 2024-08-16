local Component = require('Component')

---@class Ownership: Component
---@overload fun(playerId: integer|nil): Ownership subclass external
local Ownership = Subclass(Component, function(self, playerId)
    ---@cast self Ownership
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
