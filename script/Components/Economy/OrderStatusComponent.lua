local Component = require('Components.Component')

---@class OrderStatusComponent: Component
---@overload fun(self: OrderStatusComponent, playerId: integer|nil): OrderStatusComponent subclass internal
---@overload fun(playerId: integer|nil): OrderStatusComponent subclass external
local OrderStatusComponent = Subclass("OrderStatusComponent", Component, function(self)
    self:setComponentName("EconomyOrderStatus")

    self:setStatus(Enums.OrderStatus.Created)
end)

---@param status OrderStatus
function OrderStatusComponent:setStatus(status)
    self.status = status
end

---@return OrderStatus status
function OrderStatusComponent:getStatus()
    return self.status
end

return OrderStatusComponent
