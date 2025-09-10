local Component = require("Core.ECS.Component")

---@class OwnershipComponent: Component
---@overload fun(self: OwnershipComponent, playerId: integer|nil): OwnershipComponent subclass internal
---@overload fun(playerId: integer|nil): OwnershipComponent subclass external
local OwnershipComponent = Subclass("OwnershipComponent", Component, function(self, playerId)
    self:setComponentName("EconomyOwnership")

    self:setOwner(playerId)
end)

---@param playerId integer|nil
function OwnershipComponent:setOwner(playerId)
    self.owner = playerId
end

---@return integer playerId
function OwnershipComponent:getOwner()
    return self.owner
end

return OwnershipComponent
