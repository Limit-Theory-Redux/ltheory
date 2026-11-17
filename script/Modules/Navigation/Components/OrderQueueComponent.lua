local Component = require("Core.ECS.Component")

---@class OrderQueueComponent: Component
---@overload fun(self: OrderQueueComponent): OrderQueueComponent subclass internal
---@overload fun(): OrderQueueComponent subclass external
--- The orders of a ship, in the order they run: the ids of the order entities, the first one is
--- the head. Ids are Lua numbers.
local OrderQueueComponent = Subclass("OrderQueueComponent", Component, function(self)
    self:setComponentName("OrderQueue")
    ---@type EntityId[]
    self._ids = {}
end)

---@param orderId EntityId
function OrderQueueComponent:push(orderId)
    self._ids[#self._ids + 1] = orderId
end

---@param orderId EntityId
---@return boolean removed
function OrderQueueComponent:remove(orderId)
    for i, id in ipairs(self._ids) do
        if id == orderId then
            table.remove(self._ids, i)
            return true
        end
    end
    return false
end

---@return EntityId|nil
function OrderQueueComponent:head() return self._ids[1] end

---@return integer
function OrderQueueComponent:count() return #self._ids end

---@return EntityId[] The ids in order; a copy
function OrderQueueComponent:ids() return { table.unpack(self._ids) } end

return OrderQueueComponent
