-- A key that rises when a goal point has moved farther than a range from the point the key was made
-- for, so a plan is made again only for a goal that has really moved. One state per order.
-- Design: doc/script/nav/navigation.md

---@class GoalDriftState
---@field key integer Starts at zero and rises by one each time the point moves out of range
---@field point Position|nil The point the current key was made for

---@class GoalDrift
local GoalDrift = {}

---@return GoalDriftState
function GoalDrift.new()
    return { key = 0 }
end

---Takes the goal point now. The key rises by one when it is farther than `range` from the point the
---current key was made for (the first point always makes a key), and the point is kept then.
---@param state GoalDriftState
---@param point Position The goal now
---@param range gameunit
---@return integer key
function GoalDrift.key(state, point, range)
    if not state.point or point:distance(state.point) > range then
        state.point = point
        state.key = state.key + 1
    end
    return state.key
end

return GoalDrift
