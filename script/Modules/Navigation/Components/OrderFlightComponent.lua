local Component = require("Core.ECS.Component")

---@class OrderFlightComponent: Component
---@overload fun(self: OrderFlightComponent): OrderFlightComponent subclass internal
---@overload fun(): OrderFlightComponent subclass external
--- The order system's own state for flying an order: the plan being flown (`PlanFlight`), what the
--- plan was made under (a replan is due when it differs), the plan being asked for while the ship flies
--- the first one, the time since the order began, whether the ship has left the plan for a behaviour of
--- the world, and what the stall check needs.
local OrderFlightComponent = Subclass("OrderFlightComponent", Component, function(self)
    self:setComponentName("OrderFlight")
    ---@type PlanFlightState|nil
    self.plan = nil
    ---@type { plan: PlanFlightState, announce: boolean, legs: integer }|nil
    self.pending = nil               -- the refresh of the plan that is out: the ship flies `plan` until it comes; `announce` is whether its adoption is announced, `legs` the legs of `plan` when it was asked for
    self.adopted = 0                 -- how many refreshes have replaced the plan
    self.plannedGoalKey = nil        -- the revision of the goal the plan flies to
    self.plannedProfileRevision = nil -- the revision of the ship's cost profile at the newest request
    self.plannedZoneSignature = nil  -- `CostZones.signature` at the newest request
    self.nextRefresh = 0             -- `elapsed` at which the timer asks for a refresh
    self.refreshedLegs = 0           -- the plan's `legs` at the newest request: a refresh is due after more
    self.legsBefore = 0              -- the legs of the plans a refresh has replaced, so the stall check sees one run of legs
    self.replanDue = false           -- an announced refresh is owed: the stall check fired, or one that was out went stale
    self.elapsed = 0                 -- seconds since the order began
    self.begun = false
    self.tracking = false            -- the ship keeps the world's Orbit or Follow behaviour, with no plan
    ---@type ProgressWatch|nil
    self.watch = nil                 -- the stall check of the flight, made when the order begins
    ---@type integer|nil
    self.watchedLeg = nil            -- what the watch measures: the plan's leg, or -1 for the goal; nil to start it over
    self.advanced = false            -- the order went on to its next goal (a Patrol's next point) at its last arrival
end)

return OrderFlightComponent
