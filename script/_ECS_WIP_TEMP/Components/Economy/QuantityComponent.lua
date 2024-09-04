local Component = require('Component')

---@class QuantityComponent: Component
---@overload fun(self: QuantityComponent, playerId: integer|nil): QuantityComponent subclass internal
---@overload fun(playerId: integer|nil): QuantityComponent subclass external
local QuantityComponent = Subclass(Component, function(self)
    self:setComponentName("EconomyQuantity")
end)

---@param quantity integer
function QuantityComponent:setQuantity(quantity)
    self.quantity = quantity
end

---@return integer quantity
function QuantityComponent:getQuantity()
    return self.quantity
end

return QuantityComponent
