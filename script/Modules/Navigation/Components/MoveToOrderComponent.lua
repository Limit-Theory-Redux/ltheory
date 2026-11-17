local Component    = require("Core.ECS.Component")
local ApproachSpec = require("Shared.Types.ApproachSpec")
local NavTarget    = require("Shared.Helpers.NavTarget")

---@class MoveToOrderComponent: Component
---@overload fun(self: MoveToOrderComponent, target: NavTarget, approach: ApproachSpecId|ApproachSpecConstructor|ApproachSpec|nil): MoveToOrderComponent subclass internal
---@overload fun(target: NavTarget, approach: ApproachSpecId|ApproachSpecConstructor|ApproachSpec|nil): MoveToOrderComponent subclass external
--- What a MoveTo order flies to. The revision rises with every change of target, so a plan made
--- for an earlier one is known to be stale by comparing numbers. The approach is the spec the order
--- names, if any; without one the target's kind decides where the ship stops.
local MoveToOrderComponent = Subclass("MoveToOrderComponent", Component, function(self, target, approach)
    self:setComponentName("MoveToOrder")
    self._revision = 0
    self:setTarget(target)
    assert(approach == nil or self._targetEntityId, "MoveToOrder: an approach is for an entity target, not a point")
    self._approach = approach ~= nil and ApproachSpec.Of(approach) or nil
end)

---@param target NavTarget
function MoveToOrderComponent:setTarget(target)
    self._targetEntityId, self._targetPos = NavTarget.Split(target, "MoveToOrder")
    self._revision = self._revision + 1
end

---@return EntityId|nil The target entity's id; nil for a point
function MoveToOrderComponent:getTargetEntityId() return self._targetEntityId end

---@return Position|nil The target point; nil for an entity
function MoveToOrderComponent:getTargetPos() return self._targetPos end

---@return ApproachSpec|nil The approach the order names; nil for the target kind's own
function MoveToOrderComponent:getApproach() return self._approach end

---@return integer Starts at one and rises by one with each `setTarget`
function MoveToOrderComponent:getRevision() return self._revision end

return MoveToOrderComponent
