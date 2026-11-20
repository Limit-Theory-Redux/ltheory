local Component = require("Core.ECS.Component")

---@class NavOrderComponent: Component
---@overload fun(self: NavOrderComponent, shipId: EntityId, kind: NavOrderKindId, priority: NavOrderPriorityId|nil): NavOrderComponent subclass internal
---@overload fun(shipId: EntityId, kind: NavOrderKindId, priority: NavOrderPriorityId|nil): NavOrderComponent subclass external
--- Marks an entity as a navigation order, and says whose it is, what kind and how urgent (`Routine`
--- when nil). The kind's own data is in the order's `MoveToOrder`, `PatrolOrder` or `HoldOrder`
--- component.
local NavOrderComponent = Subclass("NavOrderComponent", Component, function(self, shipId, kind, priority)
    self:setComponentName("NavOrder")
    assert(rawtype(shipId) == "number", "NavOrder: the ship is an entity id")
    priority = priority or Enums.NavOrderPriority.Routine
    local known = false
    for _, level in pairs(Enums.NavOrderPriority) do known = known or level == priority end
    assert(known, "NavOrder: the priority is not an Enums.NavOrderPriority value")
    self._shipId = shipId
    self._kind = kind
    self._priority = priority
end)

---@return EntityId
function NavOrderComponent:getShipId() return self._shipId end

---@return NavOrderKindId
function NavOrderComponent:getKind() return self._kind end

---@return NavOrderPriorityId
function NavOrderComponent:getPriority() return self._priority end

return NavOrderComponent
