local Component = require("Core.ECS.Component")

---@class PriceComponent: Component
---@overload fun(self: PriceComponent, price: number|nil): PriceComponent subclass internal
---@overload fun(price: number|nil): PriceComponent subclass external
local PriceComponent = Subclass("PriceComponent", Component, function(self, price)
    self:setComponentName("EconomyPrice")

    self:init(price)
end)

function PriceComponent:init(price)
    self.price = price or 0
end

---@param price integer
function PriceComponent:setPrice(price)
    self.price = price
end

---@return integer price
function PriceComponent:getPrice()
    return self.price
end

return PriceComponent
