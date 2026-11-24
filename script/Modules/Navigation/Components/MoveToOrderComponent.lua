local Component    = require("Core.ECS.Component")
local ApproachSpec = require("Shared.Types.ApproachSpec")
local NavTarget    = require("Shared.Helpers.NavTarget")
local GoalDrift    = require("Shared.Helpers.GoalDrift")
local FrameOffset  = require("Shared.Helpers.FrameOffset")

---@class MoveToOrderComponent: Component
---@overload fun(self: MoveToOrderComponent, target: NavTarget, approach: ApproachSpecId|ApproachSpecConstructor|ApproachSpec|nil, frame: Entity|nil): MoveToOrderComponent subclass internal
---@overload fun(target: NavTarget, approach: ApproachSpecId|ApproachSpecConstructor|ApproachSpec|nil, frame: Entity|nil): MoveToOrderComponent subclass external
--- What a MoveTo order flies to. The revision rises with every change of target, so a plan made
--- for an earlier one is known to be stale by comparing numbers. The approach is the spec the order
--- names, if any; without one the target's kind decides where the ship stops. A point target may
--- name a frame entity: the point then keeps its offset from that entity's body.
local MoveToOrderComponent = Subclass("MoveToOrderComponent", Component, function(self, target, approach, frame)
    self:setComponentName("MoveToOrder")
    self._revision = 0
    self._drift = GoalDrift.new()
    self:setTarget(target, frame)
    assert(approach == nil or self._targetEntityId, "MoveToOrder: an approach is for an entity target, not a point")
    self._approach = approach ~= nil and ApproachSpec.Of(approach) or nil
end)

---@param target NavTarget
---@param frame Entity|nil For a point target only: the entity whose body the point rides on
function MoveToOrderComponent:setTarget(target, frame)
    self._targetEntityId, self._targetPos = NavTarget.Split(target, "MoveToOrder")
    assert(frame == nil or self._targetPos, "MoveToOrder: a frame is for a point target, not an entity")
    if frame then
        assert(rawtype(frame) == "table" and frame.isValid ~= nil, "MoveToOrder: the frame is an entity")
        self._frameEntityId = frame.id
        self._frameOffset = FrameOffset.of(frame, self._targetPos)
    else
        self._frameEntityId, self._frameOffset = nil, nil
    end
    self._revision = self._revision + 1
    -- The drift of the old target says nothing about the new one; the counter keeps rising, so the
    -- key of the new target is never one an earlier plan was made under.
    self._drift.point = nil
end

---The drift key of a goal that moves: it rises when the goal has gone farther than `range` from the
---point the current key was made for. A new target (`setTarget`) starts it over.
---@param goal Position The goal now
---@param range gameunit
---@return integer
function MoveToOrderComponent:driftKey(goal, range)
    return GoalDrift.key(self._drift, goal, range)
end

---@return EntityId|nil The target entity's id; nil for a point
function MoveToOrderComponent:getTargetEntityId() return self._targetEntityId end

---@return Position|nil The target point; nil for an entity
function MoveToOrderComponent:getTargetPos() return self._targetPos end

---@return EntityId|nil The frame entity's id; nil for a point fixed in the world, or an entity target
function MoveToOrderComponent:getFrameEntityId() return self._frameEntityId end

---@return FrameOffsetVector|nil The point's offset from the frame's body; nil with no frame
function MoveToOrderComponent:getFrameOffset() return self._frameOffset end

---@return ApproachSpec|nil The approach the order names; nil for the target kind's own
function MoveToOrderComponent:getApproach() return self._approach end

---@return integer Starts at one and rises by one with each `setTarget`
function MoveToOrderComponent:getRevision() return self._revision end

return MoveToOrderComponent
