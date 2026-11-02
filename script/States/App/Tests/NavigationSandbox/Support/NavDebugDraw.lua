-- Debug drawing of lines and spheres at world `Position`s. The render pipeline draws with the
-- eye at the origin, so each point is made camera-relative in double precision first.
-- Call between `begin` and `finish`, inside a render function.
-- Design: doc/script/navigation-sandbox.md
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local Cache         = require("Render.Cache")

-- A sphere is drawn at least this big as seen from the eye (radius in radians), so that an
-- object far below a pixel still shows.
local MIN_ANGULAR_RADIUS = 0.004

local SHADER_VERTEX, SHADER_FRAGMENT = "vp", "simple_color"

---@class DebugColor
---@field [1] number red
---@field [2] number green
---@field [3] number blue
---@field [4] number alpha

---@class NavDebugDraw
local NavDebugDraw = {}

local shader = nil
local eye    = nil

---Binds the drawing shader and reads the eye once for the draws that follow.
function NavDebugDraw.begin()
    shader = Cache.Shader(SHADER_VERTEX, SHADER_FRAGMENT)
    eye = CameraManager:getEye()
    shader:start()
end

function NavDebugDraw.finish()
    shader:stop()
    shader, eye = nil, nil
end

---@param color DebugColor
local function setColor(color)
    shader:setFloat4("color", color[1], color[2], color[3], color[4])
end

---@param a Position
---@param b Position
---@param color DebugColor
function NavDebugDraw.line(a, b, color)
    setColor(color)
    Draw.Line3(a:relativeTo(eye), b:relativeTo(eye))
end

---A sphere of `radius` game units, or the minimum angular size if that is larger.
---@param centre Position
---@param radius number
---@param color DebugColor
---@param wireframe boolean|nil Outline instead of a filled sphere
function NavDebugDraw.sphere(centre, radius, color, wireframe)
    local relative = centre:relativeTo(eye)
    radius = math.max(radius, relative:length() * MIN_ANGULAR_RADIUS)
    setColor(color)
    if wireframe then RenderState.PushWireframe(true) end
    Draw.Sphere(relative, radius)
    if wireframe then RenderState.PopWireframe() end
end

return NavDebugDraw
