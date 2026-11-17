local Entity     = require("Core.ECS.Entity")
local Components = require("Modules.Navigation.Components")

--- The components of an order of each kind, besides the ones every order has.
---@type table<NavOrderKindId, fun(spec: table): Component>
local SPEC_COMPONENTS = {
    [Enums.NavOrderKind.MoveTo] = function(spec) return Components.MoveToOrder(spec.target, spec.gap) end,
    [Enums.NavOrderKind.Patrol] = function(spec) return Components.PatrolOrder(spec.points, spec.loop) end,
    [Enums.NavOrderKind.Hold]   = function() return Components.HoldOrder() end,
}

---@param shipId EntityId The ship the order is for
---@param spec table An order spec from `Nav.Orders`
---@return Entity
return function(shipId, spec)
    local specComponent = SPEC_COMPONENTS[spec.kind]
    assert(specComponent, "Nav.order: not an order spec (build one with Nav.Orders)")
    return Entity.Create("NavOrderEntity",
        Components.NavOrder(shipId, spec.kind),
        specComponent(spec),
        Components.OrderStatus(),
        Components.OrderProgress(),
        Components.OrderFlight())
end
