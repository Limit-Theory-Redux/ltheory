local Component = require('Components.Component')

--* is type a good naming here or nah?

---@class OrderItemTypeComponent: Component
---@overload fun(self: OrderItemTypeComponent, type: integer): OrderItemTypeComponent subclass internal
---@overload fun(type: integer): OrderItemTypeComponent subclass external
local OrderItemTypeComponent = Subclass("OrderItemTypeComponent", Component, function(self, type)
    self:setComponentName("EconomyOrderItemType")

    self:setItemType(type)
end)

---@param type integer
function OrderItemTypeComponent:setItemType(type)
    self.type = type
end

---@return integer type
function OrderItemTypeComponent:getItemType()
    return self.type
end

return OrderItemTypeComponent
