local Component = require("Core.ECS.Component")
local NavTarget = require("Shared.Helpers.NavTarget")

---@class FacingGoalComponent: Component
---@overload fun(self: FacingGoalComponent, target: NavTarget): FacingGoalComponent subclass internal
---@overload fun(target: NavTarget): FacingGoalComponent subclass external
--- What the nose of the order's ship is to face while the order runs, apart from where it flies: an
--- entity, followed as it moves, or a point.
local FacingGoalComponent = Subclass("FacingGoalComponent", Component, function(self, target)
    self:setComponentName("FacingGoal")
    self._targetEntityId, self._targetPos = NavTarget.Split(target, "FacingGoal")
end)

---@return EntityId|nil The entity to face; nil for a point
function FacingGoalComponent:getTargetEntityId() return self._targetEntityId end

---@return Position|nil The point to face; nil for an entity
function FacingGoalComponent:getTargetPos() return self._targetPos end

return FacingGoalComponent
