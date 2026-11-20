local CameraManager = require("Modules.Cameras.Managers.CameraManager")

--- World-to-screen projection through a view and a projection matrix.
--- A projector is `{ view, proj, origin, x, y, w, h }`: the matrices work on points relative to `origin`
--- (the eye, or the map's focus), and the viewport is the rectangle in pixels the result lands in.
---@class ScreenProjector
---@field view Matrix
---@field proj Matrix
---@field origin Position
---@field x number
---@field y number
---@field w number
---@field h number

local ScreenProjection = {}

--- Pixels of a point given as an offset from the projector's origin.
--- The offset is subtracted in f64 by the caller, so the f32 matrices only see small numbers.
---@param p ScreenProjector
---@return number|nil px nil when the point is behind the eye
---@return number|nil py
function ScreenProjection.relativeToScreen(p, dx, dy, dz)
    local clip = p.proj:mulVec(p.view:mulVec(Vec4f(dx, dy, dz, 1.0)))
    if clip.w <= 0 then return nil, nil end
    return p.x + (clip.x / clip.w + 1) * 0.5 * p.w,
        p.y + (1 - clip.y / clip.w) * 0.5 * p.h
end

--- Pixels of a world position.
---@param p ScreenProjector
---@param pos Position
---@return number|nil px nil when the point is behind the eye
---@return number|nil py
function ScreenProjection.toScreen(p, pos)
    local rel = pos:relativeTo(p.origin)
    return ScreenProjection.relativeToScreen(p, rel.x, rel.y, rel.z)
end

--- Is the pixel inside the projector's viewport, grown by `margin` pixels?
---@param p ScreenProjector
---@param margin number|nil
---@return boolean
function ScreenProjection.inViewport(p, px, py, margin)
    margin = margin or 0
    return px >= p.x - margin and px <= p.x + p.w + margin
        and py >= p.y - margin and py <= p.y + p.h + margin
end

--- Fills `out` (or a new table) with the active camera's projector over the whole window.
---@param out ScreenProjector|nil
---@return ScreenProjector
function ScreenProjection.active(out)
    out = out or {}
    out.view = CameraManager:getViewMatrix()
    out.proj = CameraManager:getProjectionMatrix()
    out.origin = CameraManager:getEye()
    out.x, out.y = 0, 0
    out.w, out.h = Window:width(), Window:height()
    return out
end

return ScreenProjection
