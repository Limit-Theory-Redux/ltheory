local Entity        = require("Core.ECS.Entity")
local CoreComponents = require("Modules.Core.Components")
local Physics       = require("Modules.Physics.Components")
local Rendering     = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")

local Primitive = require("Legacy.Systems.Gen.Primitive")

---Engine exhaust plume: volumetric streaming jet (fragment/effect/
---exhaust_plume.glsl). Attach to a construct's nozzle; thrust direction
---and length follow config. Render-only component; lifetime is managed by
---the owner (no system needed - it burns while alive).
---@param seed integer
---@param config table {position, direction = {x,y,z} thrust axis,
--- length = number (world units), radius = number (nozzle radius)}
---@return Entity
return function(seed, config)
    config = config or {}

    local mesh
    local shader
    local function ensureAssets()
        if not mesh then
            mesh = Primitive.Billboard(-1, -1, 1, 1)
            shader = Cache.Shader("billboard/exhaust", "effect/exhaust_plume")
        end
    end

    local dir = config.direction or { x = 0, y = 1, z = 0 }
    local dirLen = math.sqrt(dir.x * dir.x + dir.y * dir.y + dir.z * dir.z)
    local ux, uy, uz = dir.x / dirLen, dir.y / dirLen, dir.z / dirLen
    local plumeLen = config.length or 8.0
    local radius = config.radius or 0.5
    -- Live thrust strength 0..1 (setBoost): drives color/density each frame.
    local boostState = { value = config.boost or 0.5 }

    local function render(entity, blendMode)
        if blendMode ~= BlendMode.Additive then
            return
        end
        ensureAssets()
        local transform = entity:get(Physics.Transform)
        local pos = transform and transform:getPos()
        if not pos then
            return
        end

        shader:start()
        -- Camera-relative draw boundary (same convention as the explosion
        -- effect): the projection math runs in the camera's frame, so the
        -- origin is passed relative to the eye - absolute coordinates
        -- project to the wrong place ("renders at the skybox").
        local eye = CameraManager:getEye()
        shader:setFloat3("origin", pos.x - eye.x, pos.y - eye.y, pos.z - eye.z)
        shader:setFloat3("up", ux, uy, uz)
        shader:setFloat("size", radius)
        shader:setFloat("plumeLen", plumeLen)
        shader:setFloat("age", Engine:getTime() + (seed or 0) % 64)
        shader:setFloat("seed", (seed or 0) % 1024)
        shader:setFloat("boost", boostState.value)
        -- Single camera-facing card: visible from every angle (crossed
        -- cards turned edge-on from some views and made the jet vanish).
        mesh:draw()
        shader:stop()
    end

    local entity = Entity.Create(
        "ExhaustPlumeEntity",
        CoreComponents.Seed(seed or 0),
        Physics.Transform(),
        Rendering.Render(render))

    local transform = entity:get(Physics.Transform)
    transform:setPos(config.position or Position())

    ---Update the thrust strength (0..1) live; drives the jet color/density.
    ---@param value number
    function entity:setBoost(value)
        boostState.value = value
    end

    return entity
end
