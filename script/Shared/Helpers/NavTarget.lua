-- Design: doc/script/nav/navigation.md
---@alias NavTarget Entity|Position An entity to fly to or face, or a point in the world

---What an order names as a target: an entity (followed as it moves) or a point.
---@class NavTargetHelper
local NavTarget = {}

---Splits a target into the entity's id or a copy of the point.
---@param target NavTarget
---@param who string The order's name, for the message when the target is neither
---@return EntityId|nil entityId Nil for a point
---@return Position|nil point Nil for an entity
function NavTarget.Split(target, who)
    local isEntity = rawtype(target) == "table" and target.isValid ~= nil
    local isPoint = not isEntity and target ~= nil and target.x ~= nil and target.y ~= nil and target.z ~= nil
    assert(isEntity or isPoint, who .. ": the target is an entity or a position")
    if isEntity then return target.id, nil end
    return nil, Position(target.x, target.y, target.z)
end

return NavTarget
