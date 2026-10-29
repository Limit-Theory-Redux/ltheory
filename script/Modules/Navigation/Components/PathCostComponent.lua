local Component = require("Core.ECS.Component")

---@alias CostPerMetre number Extra path cost of each metre travelled, dimensionless

---@class PathCostComponent: Component
---@overload fun(self: PathCostComponent, cost: CostPerMetre): PathCostComponent subclass internal
---@overload fun(cost: CostPerMetre): PathCostComponent subclass external
--- The extra cost of each metre travelled inside a region (a zone entity), on top of the distance
--- itself. Zero costs the same as open space.
local PathCostComponent = Subclass("PathCostComponent", Component, function(self, cost)
    self:setComponentName("PathCost")
    self:setCost(cost)
end)

---@param cost CostPerMetre At least zero
function PathCostComponent:setCost(cost)
    assert(type(cost) == "number" and cost >= 0, "PathCost: the cost must be a number of at least zero")
    self._cost = cost
end

---@return CostPerMetre
function PathCostComponent:getCost()
    return self._cost
end

return PathCostComponent
