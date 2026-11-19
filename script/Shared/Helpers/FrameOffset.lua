local PhysicsComponents = require("Modules.Physics.Components")

---@alias FrameOffsetVector { x: gameunit, y: gameunit, z: gameunit } A world-axes offset from a frame body's centre

---A point kept at a fixed offset from an entity's body, translation only: the point does not turn with
---the body.
---@class FrameOffset
local FrameOffset = {}

---@param frame Entity
---@return RigidBody|nil
local function bodyOf(frame)
    local component = frame:get(PhysicsComponents.RigidBody)
    return component and component:getRigidBody() or nil
end

---The offset of a world point from a frame entity's body.
---@param frame Entity Has a rigid body
---@param point Position
---@return FrameOffsetVector
function FrameOffset.of(frame, point)
    local body = bodyOf(frame)
    assert(body, "FrameOffset.of: the frame entity has no rigid body")
    local at = body:getPos()
    return { x = point.x - at.x, y = point.y - at.y, z = point.z - at.z }
end

---Where a point of the given offset is now.
---@param frame Entity
---@param offset FrameOffsetVector From `of`
---@return Position|nil Nil when the frame has no body now
function FrameOffset.point(frame, offset)
    local body = bodyOf(frame)
    if not body then return nil end
    local at = body:getPos()
    return Position(at.x + offset.x, at.y + offset.y, at.z + offset.z)
end

return FrameOffset
