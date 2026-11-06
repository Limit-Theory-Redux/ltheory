-- Debug drawing of lines and spheres at world `Position`s. The render pipeline draws with the
-- eye at the origin, so each point is made camera-relative in double precision first.
-- Call between `begin` and `finish`, inside a render function (an open scene pass).
-- Design: doc/script/navigation-sandbox.md
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local Cache         = require("Render.Cache")
local Pipelines     = require("Render.Pipelines")

-- A sphere is drawn at least this big as seen from the eye (radius in radians), so that an
-- object far below a pixel still shows.
local MIN_ANGULAR_RADIUS = 0.004

-- Subdivisions per face of the unit sphere mesh.
local SPHERE_MESH_RESOLUTION = 8
-- A wireframe sphere is three great circles, each this many segments; lines are this wide in pixels.
local WIREFRAME_SEGMENTS   = 48
local WIREFRAME_LINE_WIDTH = 1

-- Points of the unit circle, from angle 0 round to 2 pi (the last point equals the first).
local UNIT_CIRCLE = {}
for i = 0, WIREFRAME_SEGMENTS do
    local angle = i * 2 * math.pi / WIREFRAME_SEGMENTS
    UNIT_CIRCLE[i + 1] = { math.cos(angle), math.sin(angle) }
end

local SHADER_VERTEX, SHADER_FRAGMENT = "wvp", "flat_color_draw"

-- Entries of a column-major 4x4 matrix: the diagonal and the translation column.
local MATRIX_XX, MATRIX_YY, MATRIX_ZZ, MATRIX_WW = 0, 5, 10, 15
local MATRIX_TX, MATRIX_TY, MATRIX_TZ = 12, 13, 14

---@class DebugColor
---@field [1] number red
---@field [2] number green
---@field [3] number blue
---@field [4] number alpha

---@class NavDebugDraw
local NavDebugDraw = {}

local sphereMesh, sphereShader, DrawBlock
local eye = nil

---Creates the sphere mesh, shader and draw-block type on first use, binds the sphere pipeline,
---and reads the eye once for the draws that follow.
function NavDebugDraw.begin()
    if not sphereMesh then
        sphereMesh = Mesh.BoxSphere(SPHERE_MESH_RESOLUTION)
        sphereShader = Cache.Shader(SHADER_VERTEX, SHADER_FRAGMENT)
        DrawBlock = sphereShader:blockType("DrawBlock")
    end
    -- Uploads the mesh now: an upload between `alloc` and the draw would flush the block early.
    sphereMesh:resourceId(Renderer)
    -- The immediate lines bind their own pipeline; the pass restores this one before a mesh draw.
    Renderer:currentPass():setPipeline(Pipelines.get(sphereShader, Pipelines.Additive))
    eye = CameraManager:getEye()
end

function NavDebugDraw.finish()
    eye = nil
end

---@param a Position
---@param b Position
---@param color DebugColor
function NavDebugDraw.line(a, b, color)
    Imm.Line3(a:relativeTo(eye), b:relativeTo(eye), Color(color[1], color[2], color[3], color[4]),
        WIREFRAME_LINE_WIDTH)
end

---Three great circles, one in each axis plane.
---@param centre Vec3f camera-relative
---@param radius number
---@param color DebugColor
local function wireframeSphere(centre, radius, color)
    local lineColor = Color(color[1], color[2], color[3], color[4])
    local cx, cy, cz = centre.x, centre.y, centre.z
    for segment = 1, WIREFRAME_SEGMENTS do
        local c0, s0 = UNIT_CIRCLE[segment][1] * radius, UNIT_CIRCLE[segment][2] * radius
        local c1, s1 = UNIT_CIRCLE[segment + 1][1] * radius, UNIT_CIRCLE[segment + 1][2] * radius
        -- XY, XZ and YZ planes.
        Imm.Line3(Vec3f(cx + c0, cy + s0, cz), Vec3f(cx + c1, cy + s1, cz), lineColor, WIREFRAME_LINE_WIDTH)
        Imm.Line3(Vec3f(cx + c0, cy, cz + s0), Vec3f(cx + c1, cy, cz + s1), lineColor, WIREFRAME_LINE_WIDTH)
        Imm.Line3(Vec3f(cx, cy + c0, cz + s0), Vec3f(cx, cy + c1, cz + s1), lineColor, WIREFRAME_LINE_WIDTH)
    end
end

---@param centre Vec3f camera-relative
---@param radius number
---@param color DebugColor
local function filledSphere(centre, radius, color)
    local pass = Renderer:currentPass()
    local block = pass:alloc(DrawBlock)
    local world, worldIT = block.mWorld, block.mWorldIT
    -- A uniform scale, so the normal matrix is the identity up to scale.
    world[MATRIX_XX], world[MATRIX_YY], world[MATRIX_ZZ], world[MATRIX_WW] = radius, radius, radius, 1
    worldIT[MATRIX_XX], worldIT[MATRIX_YY], worldIT[MATRIX_ZZ], worldIT[MATRIX_WW] = 1, 1, 1, 1
    world[MATRIX_TX], world[MATRIX_TY], world[MATRIX_TZ] = centre.x, centre.y, centre.z
    block.drawUser[0], block.drawUser[1], block.drawUser[2], block.drawUser[3] =
        color[1], color[2], color[3], color[4]
    pass:drawMesh(sphereMesh)
end

---A sphere of `radius` game units, or the minimum angular size if that is larger.
---@param centre Position
---@param radius number
---@param color DebugColor
---@param wireframe boolean|nil Outline instead of a filled sphere
function NavDebugDraw.sphere(centre, radius, color, wireframe)
    local relative = centre:relativeTo(eye)
    radius = math.max(radius, relative:length() * MIN_ANGULAR_RADIUS)
    if wireframe then
        wireframeSphere(relative, radius, color)
    else
        filledSphere(relative, radius, color)
    end
end

return NavDebugDraw
