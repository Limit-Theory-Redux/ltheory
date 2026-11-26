local Component = require("Core.ECS.Component")

---@class PatrolOrderComponent: Component
---@overload fun(self: PatrolOrderComponent, points: Position[], loop: boolean): PatrolOrderComponent subclass internal
---@overload fun(points: Position[], loop: boolean): PatrolOrderComponent subclass external
--- The points a Patrol order flies through in turn, and whether it starts over after the last.
--- The index is the point the ship is flying to; the order system advances it.
local PatrolOrderComponent = Subclass("PatrolOrderComponent", Component, function(self, points, loop)
    self:setComponentName("PatrolOrder")
    assert(rawtype(points) == "table" and #points > 0, "PatrolOrder: at least one point")
    self._points = {}
    for i, point in ipairs(points) do
        self._points[i] = Position(point.x, point.y, point.z)
    end
    self._loop = loop == true
    self._index = 1
end)

---@return Position[]
function PatrolOrderComponent:getPoints() return self._points end

---@return boolean
function PatrolOrderComponent:isLoop() return self._loop end

---@return integer The 1-based index of the point the ship flies to
function PatrolOrderComponent:getIndex() return self._index end

---@param index integer
function PatrolOrderComponent:setIndex(index) self._index = index end

return PatrolOrderComponent
