local Component = require("Core.ECS.Component")

---@class NavOrderComponent: Component
---@overload fun(self: NavOrderComponent, shipId: EntityId, kind: NavOrderKindId): NavOrderComponent subclass internal
---@overload fun(shipId: EntityId, kind: NavOrderKindId): NavOrderComponent subclass external
--- Marks an entity as a navigation order, and says whose it is and what kind. The kind's own data
--- is in the order's `MoveToOrder`, `PatrolOrder` or `HoldOrder` component.
local NavOrderComponent = Subclass("NavOrderComponent", Component, function(self, shipId, kind)
    self:setComponentName("NavOrder")
    assert(rawtype(shipId) == "number", "NavOrder: the ship is an entity id")
    self._shipId = shipId
    self._kind = kind
end)

---@return EntityId
function NavOrderComponent:getShipId() return self._shipId end

---@return NavOrderKindId
function NavOrderComponent:getKind() return self._kind end

return NavOrderComponent
