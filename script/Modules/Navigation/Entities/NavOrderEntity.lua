local Entity     = require("Core.ECS.Entity")
local Components = require("Modules.Navigation.Components")

--- The components of an order of each kind, besides the ones every order has.
---@type table<NavOrderKindId, fun(spec: table): Component>
local SPEC_COMPONENTS = {
    [Enums.NavOrderKind.MoveTo] = function(spec) return Components.MoveToOrder(spec.target, spec.approach, spec.frame) end,
    [Enums.NavOrderKind.Patrol] = function(spec) return Components.PatrolOrder(spec.points, spec.loop) end,
    [Enums.NavOrderKind.Hold]   = function() return Components.HoldOrder() end,
    [Enums.NavOrderKind.Orbit]  = function(spec) return Components.OrbitOrder(spec.target, spec.altitude, spec.normal) end,
    [Enums.NavOrderKind.Follow] = function(spec) return Components.FollowOrder(spec.target, spec.offset) end,
}

---@param shipId EntityId The ship the order is for
---@param spec table An order spec from `Nav.Orders`
---@return Entity
return function(shipId, spec)
    local specComponent = SPEC_COMPONENTS[spec.kind]
    assert(specComponent, "Nav.order: not an order spec (build one with Nav.Orders)")
    -- Every component is made before the entity: a spec that is refused leaves no order behind.
    local components = {
        Components.NavOrder(shipId, spec.kind, spec.priority),
        specComponent(spec),
        Components.OrderStatus(),
        Components.OrderProgress(),
        Components.OrderFlight(),
    }
    if spec.facing then components[#components + 1] = Components.FacingGoal(spec.facing.target) end
    return Entity.Create("NavOrderEntity", unpack(components))
end
