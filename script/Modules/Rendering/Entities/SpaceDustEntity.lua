local Entity    = require("Core.ECS.Entity")
local Rendering = require("Modules.Rendering.Components")
local Pipelines = require("Render.Pipelines")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")

---Space dust: specks of dust around the camera that streak along its motion,
---so flying past them shows speed (stars and nebula are too far away to move).
---
---Each layer is a cube of `count` specks, `cell` game units wide, that wraps
---around the camera: a speck that falls behind one face comes back in at the
---other, so there are always specks around, wherever the camera flies. The
---vertex shader (vertex/dust_streak.glsl) does the wrapping and draws each
---speck as a streak along `streak` (where it was a moment ago, relative to
---the camera), a fixed number of pixels wide, in `SEGMENTS` pieces so it can
---bend: around the bubble, a disc on screen around the ship that pushes the
---dust aside (travel mode). Specks fade out towards the cube's faces so the
---wrap never shows.
---
---Whoever drives the dust (SpeedFeelSystem) sets, every frame, each layer's
---opacity, brightness, streak and offset, and the bubble. The offset is how
---far the dust has fallen behind the camera's real motion: 0 keeps the dust
---static in the world; a driver that caps the dust's speed (travel mode is far
---too fast for specks to be seen) adds the difference to it.

---@class SpaceDustLayer
---@field cell number cube size (game units)
---@field count integer specks
---@field widthPx number streak width in pixels at 1080 lines
---@field color Color
---@field opacity number 0..1, set every frame
---@field brightness number multiplies the colour, set every frame
---@field streakX number the streak (game units, camera-relative), set every frame
---@field streakY number
---@field streakZ number
---@field offsetX number how far the dust lags the camera (game units)
---@field offsetY number
---@field offsetZ number
---@field mesh Mesh

---@class SpaceDustBubble
---@field x number the ship on screen, NDC (-1..1, y up)
---@field y number
---@field radius number fraction of the screen height
---@field strength number 0..1 (0 = no bubble)
---@field curl number radians the bubble turns the dust around the ship, at its centre
---@field shiftX number NDC the whole dust moves on screen (third person: so the
---streaks converge on the ship instead of where it's heading)
---@field shiftY number

---@class SpaceDustState
---@field layers SpaceDustLayer[]
---@field bubble SpaceDustBubble

local SEGMENTS = 8 -- pieces per streak

local dustShader
local DrawBlock

local function ensureShader()
    if not dustShader then
        dustShader = Cache.Shader("dust_streak", "effect/dust_streak")
        DrawBlock = dustShader:blockType("DrawBlock")
    end
end

-- One strip per speck: SEGMENTS + 1 pairs of corners, all at the speck's
-- place in the unit cube; uv.x is how far back along the streak (0 = where it
-- is now, 1 = the far end) and uv.y the side (-1, 1).
---@param count integer
---@param seed integer
---@return Mesh
local function buildMesh(count, seed)
    local rng = RNG.Create(seed)
    local mesh = Mesh.Create()
    local perSpeck = 2 * (SEGMENTS + 1)
    for i = 0, count - 1 do
        local x, y, z = rng:getUniform(), rng:getUniform(), rng:getUniform()
        for k = 0, SEGMENTS do
            local t = k / SEGMENTS
            mesh:addVertex(x, y, z, 0, 0, 0, t, -1)
            mesh:addVertex(x, y, z, 0, 0, 0, t, 1)
        end
        local base = i * perSpeck
        for k = 0, SEGMENTS - 1 do
            local i0 = base + 2 * k
            mesh:addQuad(i0, i0 + 2, i0 + 3, i0 + 1)
        end
    end
    return mesh
end

-- The fractional part, done in Lua's doubles: the camera can be far from the
-- origin, and only the position within one cube matters to the shader.
local function frac(x)
    return x - math.floor(x)
end

---Render callback, run by RenderCoreSystem in every scene pass; draws in the
---additive one. Rendering is camera-relative: the shader gets the camera's
---place within each cube, not its position.
---@param state SpaceDustState
---@param blendMode BlendMode
local function render(state, blendMode)
    if blendMode ~= BlendMode.Additive then return end

    ensureShader()
    local eye = CameraManager:getEye()
    local bubble = state.bubble
    local pass = Renderer:currentPass()
    pass:setPipeline(Pipelines.get(dustShader, Pipelines.Additive))

    for _, layer in ipairs(state.layers) do
        if layer.opacity > 0.001 then
            local d = pass:alloc(DrawBlock)
            local m = d.mWorld
            m[0], m[5], m[10], m[15] = 1, 1, 1, 1
            local user = d.drawUser
            local cell = layer.cell
            user[0], user[1], user[2], user[3] =
                frac((eye.x - layer.offsetX) / cell), frac((eye.y - layer.offsetY) / cell),
                frac((eye.z - layer.offsetZ) / cell), cell
            user[4], user[5], user[6], user[7] = layer.streakX, layer.streakY, layer.streakZ, layer.opacity
            user[8], user[9], user[10] = layer.widthPx, layer.brightness, SEGMENTS
            user[12], user[13], user[14] = layer.color.r, layer.color.g, layer.color.b
            user[16], user[17], user[18], user[19] = bubble.x, bubble.y, bubble.radius, bubble.strength
            user[20], user[21], user[22] = bubble.curl, bubble.shiftX, bubble.shiftY
            pass:drawMesh(layer.mesh)
        end
    end
end

---@param seed integer
---@param layers { cell: number, count: integer, widthPx: number, color: Color }[]
---@return Entity entity
---@return SpaceDustState state what the driver sets every frame (see SpaceDustLayer)
return function(seed, layers)
    local state = {
        layers = {},
        bubble = { x = 0, y = 0, radius = 0, strength = 0, curl = 0, shiftX = 0, shiftY = 0 },
    }
    for i, cfg in ipairs(layers) do
        state.layers[i] = {
            cell = cfg.cell,
            count = cfg.count,
            widthPx = cfg.widthPx,
            color = cfg.color,
            opacity = 0,
            brightness = 1,
            streakX = 0, streakY = 0, streakZ = 0,
            offsetX = 0, offsetY = 0, offsetZ = 0,
            mesh = buildMesh(cfg.count, seed + i * 7919),
        }
    end

    local entity = Entity.Create("SpaceDustEntity",
        Rendering.Render(function(_, blendMode) render(state, blendMode) end))
    return entity, state
end
