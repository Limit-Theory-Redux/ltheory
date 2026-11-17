local Component = require("Core.ECS.Component")

---@class NavAgentComponent: Component
---@overload fun(self: NavAgentComponent, slot: integer): NavAgentComponent subclass internal
---@overload fun(slot: integer): NavAgentComponent subclass external
--- The ship's agent in the navigation world: its slot. The component is on the ship while the
--- ship has an order to fly, and the slot may be handed to another agent after it is removed.
local NavAgentComponent = Subclass("NavAgentComponent", Component, function(self, slot)
    self:setComponentName("NavAgent")
    self._slot = slot
end)

---@return integer
function NavAgentComponent:getSlot() return self._slot end

return NavAgentComponent
