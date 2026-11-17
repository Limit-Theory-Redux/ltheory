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
---vertex shader (vertex/dust_streak.glsl) does the wrapping and turns each
---speck into a streak from where it is to where it was `streakTime` seconds
---ago (as a camera shutter would), a fixed number of pixels wide; specks fade
---out towards the cube's faces so the wrap never shows. Two layers of
---different sizes cover slow flight (small cube, specks close by) and boost
---(big cube); whoever drives the dust (SpeedFeelSystem) sets each layer's
---opacity and the velocity every frame.
---
---The dust is static in the world: its motion on screen is only the camera's.

---@class SpaceDustLayer
---@field cell number cube size (game units)
---@field count integer specks
---@field widthPx number streak width in pixels
---@field color Color
---@field opacity number 0..1, set every frame
---@field mesh Mesh

---@class SpaceDustState
---@field layers SpaceDustLayer[]
---@field vx number camera velocity (game units / s), set every frame
---@field vy number
---@field vz number
---@field streakTime number seconds of motion a streak shows

local dustShader
local DrawBlock

local function ensureShader()
    if not dustShader then
        dustShader = Cache.Shader("dust_streak", "effect/dust_streak")
        DrawBlock = dustShader:blockType("DrawBlock")
    end
end

-- One quad per speck, all four corners at the speck's place in the unit cube;
-- uv.x picks the end (0 = where it is, 1 = where it was) and uv.y the side.
---@param count integer
---@param seed integer
---@return Mesh
local function buildMesh(count, seed)
    local rng = RNG.Create(seed)
    local mesh = Mesh.Create()
    for i = 0, count - 1 do
        local x, y, z = rng:getUniform(), rng:getUniform(), rng:getUniform()
        mesh:addVertex(x, y, z, 0, 0, 0, 0, -1)
        mesh:addVertex(x, y, z, 0, 0, 0, 1, -1)
        mesh:addVertex(x, y, z, 0, 0, 0, 1, 1)
        mesh:addVertex(x, y, z, 0, 0, 0, 0, 1)
        local i0 = 4 * i
        mesh:addQuad(i0, i0 + 1, i0 + 2, i0 + 3)
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
    local pass = Renderer:currentPass()
    pass:setPipeline(Pipelines.get(dustShader, Pipelines.Additive))

    for _, layer in ipairs(state.layers) do
        if layer.opacity > 0.001 then
            local d = pass:alloc(DrawBlock)
            local m = d.mWorld
            m[0], m[5], m[10], m[15] = 1, 1, 1, 1
            local user = d.drawUser
            user[0], user[1], user[2], user[3] =
                frac(eye.x / layer.cell), frac(eye.y / layer.cell), frac(eye.z / layer.cell), layer.cell
            user[4], user[5], user[6], user[7] =
                state.vx * state.streakTime, state.vy * state.streakTime, state.vz * state.streakTime,
                layer.opacity
            user[8] = layer.widthPx
            user[12], user[13], user[14] = layer.color.r, layer.color.g, layer.color.b
            pass:drawMesh(layer.mesh)
        end
    end
end

---@param seed integer
---@param layers { cell: number, count: integer, widthPx: number, color: Color }[]
---@return Entity entity
---@return SpaceDustState state what the driver sets every frame (layer opacity, velocity)
return function(seed, layers)
    local state = { layers = {}, vx = 0, vy = 0, vz = 0, streakTime = 0.05 }
    for i, cfg in ipairs(layers) do
        state.layers[i] = {
            cell = cfg.cell,
            count = cfg.count,
            widthPx = cfg.widthPx,
            color = cfg.color,
            opacity = 0,
            mesh = buildMesh(cfg.count, seed + i * 7919),
        }
    end

    local entity = Entity.Create("SpaceDustEntity",
        Rendering.Render(function(_, blendMode) render(state, blendMode) end))
    return entity, state
end
