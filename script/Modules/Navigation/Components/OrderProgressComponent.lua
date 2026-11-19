local Component = require("Core.ECS.Component")

---@class OrderProgressComponent: Component
---@overload fun(self: OrderProgressComponent): OrderProgressComponent subclass internal
---@overload fun(): OrderProgressComponent subclass external
--- How far an order is from its end, written by the order system every update the order is the
--- ship's head order. Nil fields have no value yet.
local OrderProgressComponent = Subclass("OrderProgressComponent", Component, function(self)
    self:setComponentName("OrderProgress")
    self.distanceLeft = nil -- gameunit: to the order's goal
    self.eta = nil          -- seconds until the ship is at the goal
    self.waypointIndex = nil -- Patrol: the 1-based point the ship flies to
    self.nextWaypoint = nil -- Position: where the ship steers now (the plan's next corner or arc point, else the goal); nil for Orbit, Follow and Hold
end)

return OrderProgressComponent
