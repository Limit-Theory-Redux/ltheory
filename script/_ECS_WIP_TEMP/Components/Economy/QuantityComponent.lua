local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class QuantityComponent: Component
---@overload fun(self: QuantityComponent, quantity: number): QuantityComponent subclass internal
---@overload fun(quantity: number): QuantityComponent subclass external
local QuantityComponent = Subclass(Component, function(self, quantity)
    self:setComponentName("EconomyQuantity")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.QuantityComponent)

    self:init(quantity)
end)

function QuantityComponent:init(quantity)
    self.quantity = quantity or 0
end

---@param quantity integer
function QuantityComponent:setQuantity(quantity)
    self.quantity = quantity
end

---@return integer quantity
function QuantityComponent:getQuantity()
    return self.quantity
end

return QuantityComponent
