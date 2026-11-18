local Component = require("Core.ECS.Component")

---@class OrderFlightComponent: Component
---@overload fun(self: OrderFlightComponent): OrderFlightComponent subclass internal
---@overload fun(): OrderFlightComponent subclass external
--- The order system's own state for flying an order: the plan being flown (`PlanFlight`), what the
--- plan was made under (a replan is due when it differs), the time since the order began, and whether
--- the ship has left the plan for a behaviour of the world.
local OrderFlightComponent = Subclass("OrderFlightComponent", Component, function(self)
    self:setComponentName("OrderFlight")
    ---@type PlanFlightState|nil
    self.plan = nil
    self.plannedGoalKey = nil        -- the revision of the goal the plan flies to
    self.plannedProfileRevision = nil -- the revision of the ship's cost profile
    self.elapsed = 0                 -- seconds since the order began
    self.begun = false
    self.tracking = false            -- the ship keeps the world's Orbit or Follow behaviour, with no plan
end)

return OrderFlightComponent
