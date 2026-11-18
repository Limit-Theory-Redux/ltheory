local Component = require("Core.ECS.Component")

---@class NavOrderQueueEntry
---@field id EntityId
---@field priority NavOrderPriorityId

---@class OrderQueueComponent: Component
---@overload fun(self: OrderQueueComponent): OrderQueueComponent subclass internal
---@overload fun(): OrderQueueComponent subclass external
--- The orders of a ship, in the order they run: the ids of the order entities, the first one is
--- the head. A higher priority runs before a lower one, and orders of one priority run in the order
--- they were given. Ids are Lua numbers.
local OrderQueueComponent = Subclass("OrderQueueComponent", Component, function(self)
    self:setComponentName("OrderQueue")
    ---@type NavOrderQueueEntry[]
    self._entries = {}
end)

---Adds an order behind every order of its priority or a higher one, and before the lower ones.
---@param orderId EntityId
---@param priority NavOrderPriorityId
function OrderQueueComponent:push(orderId, priority)
    local at = #self._entries + 1
    while at > 1 and self._entries[at - 1].priority < priority do at = at - 1 end
    table.insert(self._entries, at, { id = orderId, priority = priority })
end

---@param orderId EntityId
---@return boolean removed
function OrderQueueComponent:remove(orderId)
    for i, entry in ipairs(self._entries) do
        if entry.id == orderId then
            table.remove(self._entries, i)
            return true
        end
    end
    return false
end

---@return EntityId|nil
function OrderQueueComponent:head()
    local entry = self._entries[1]
    return entry and entry.id or nil
end

---@return integer
function OrderQueueComponent:count() return #self._entries end

---@return EntityId[] The ids in order; a copy
function OrderQueueComponent:ids()
    local ids = {}
    for i, entry in ipairs(self._entries) do ids[i] = entry.id end
    return ids
end

return OrderQueueComponent
