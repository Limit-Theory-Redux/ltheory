local Component = require("Core.ECS.Component")

---@alias NavTarget Entity|Position An entity to fly to, or a point in the world

---@class MoveToOrderComponent: Component
---@overload fun(self: MoveToOrderComponent, target: NavTarget, gap: gameunit|nil): MoveToOrderComponent subclass internal
---@overload fun(target: NavTarget, gap: gameunit|nil): MoveToOrderComponent subclass external
--- What a MoveTo order flies to. The revision rises with every change of target, so a plan made
--- for an earlier one is known to be stale by comparing numbers.
local MoveToOrderComponent = Subclass("MoveToOrderComponent", Component, function(self, target, gap)
    self:setComponentName("MoveToOrder")
    assert(gap == nil or (rawtype(gap) == "number" and gap > 0), "MoveToOrder: the gap is a distance greater than zero")
    self._gap = gap
    self._revision = 0
    self:setTarget(target)
end)

---@param target NavTarget
function MoveToOrderComponent:setTarget(target)
    local isEntity = rawtype(target) == "table" and target.isValid ~= nil
    local isPoint = not isEntity and target ~= nil and target.x ~= nil and target.y ~= nil and target.z ~= nil
    assert(isEntity or isPoint, "MoveToOrder: the target is an entity or a position")
    self._targetEntityId = isEntity and target.id or nil
    self._targetPos = isPoint and Position(target.x, target.y, target.z) or nil
    self._revision = self._revision + 1
end

---@return EntityId|nil The target entity's id; nil for a point
function MoveToOrderComponent:getTargetEntityId() return self._targetEntityId end

---@return Position|nil The target point; nil for an entity
function MoveToOrderComponent:getTargetPos() return self._targetPos end

---@return gameunit|nil The wanted distance between the hulls at the stop; nil for the default stand-off
function MoveToOrderComponent:getGap() return self._gap end

---@return integer Starts at one and rises by one with each `setTarget`
function MoveToOrderComponent:getRevision() return self._revision end

return MoveToOrderComponent
