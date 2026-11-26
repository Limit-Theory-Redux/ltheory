local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local Pipelines = require("Render.Pipelines")
local Primitive = require("Shared.Generation.Meshes.Primitive")

local ffi = require("ffi")

local beamMesh
local beamShader
local DrawBlock

-- World positions stay in double precision (a Vec3f is kilometres off this far
-- from the origin); only the eye-relative values below are single precision.
---@return Position|nil
local function getPosition(entity)
    if not entity or not entity:isValid() then
        return nil
    end

    local rigidBody = entity:get(Physics.RigidBody)
    if rigidBody and rigidBody:getRigidBody() then
        return rigidBody:getRigidBody():getPos()
    end

    local transform = entity:get(Physics.Transform)
    return transform and transform:getPos() or nil
end

local function render(entity, blendMode)
    if blendMode ~= BlendMode.Additive then
        return
    end

    local beam = entity:get(Constructs.Beam)
    local startPosition = getPosition(beam:getSource())
    local endPosition = beam:getTargetPoint() or getPosition(beam:getTarget())
    if not startPosition or not endPosition then
        return
    end

    local axis = Vec3f(
        endPosition.x - startPosition.x,
        endPosition.y - startPosition.y,
        endPosition.z - startPosition.z)
    local length = axis:length()
    if length <= 1e-6 then
        return
    end

    if not beamMesh then
        beamMesh = Primitive.Billboard(-1, 0, 1, 1)
        beamShader = Cache.Shader("billboard/axis_draw", "effect/beam_draw")
        DrawBlock = beamShader:blockType("DrawBlock")
    end

    local eye = CameraManager:getEye()
    local startRelative = Vec3f(
        startPosition.x - eye.x,
        startPosition.y - eye.y,
        startPosition.z - eye.z)
    local direction = axis:normalize()
    local matrix = Matrix.LookUp(
        startRelative,
        -direction,
        Math.OrthoVector(direction))
    local visual = beam:getVisual()
    local color = visual.bodyColor

    -- One draw in the additive pass: the pipeline sets the state, the draw
    -- block (group 2) carries the transform and the effect parameters.
    local pass = Renderer:currentPass()
    pass:setPipeline(Pipelines.get(beamShader, Pipelines.Additive))
    local d = pass:alloc(DrawBlock)
    ffi.copy(d.mWorld, matrix, 64)
    local user = d.drawUser
    user[0], user[1], user[2], user[3] = color.r, color.g, color.b, 1.0 -- color, alpha
    user[4], user[5], user[6] = visual.beamWidth or 0.008, length, 0.0  -- size, seed
    pass:drawMesh(beamMesh)
end

---@param seed integer
---@param meshes MeshWithMaterial[]|nil
---@param config table
---@return Entity
return function(seed, meshes, config)
    config = config or {}
    local effect = config.effect
    assert(effect and effect.kind == Enums.Weapon.Effect.Beam,
        "BeamEntity requires a beam effect definition")
    local visual = config.visual or effect.visual or {}

    return Entity.Create(
        "BeamEntity",
        Core.Seed(seed or 0),
        Physics.Transform(),
        Physics.Mass(),
        Rendering.PointLight(
            visual.lightColor,
            visual.lightRadius,
            visual.lightIntensity),
        Rendering.Render(render),
        Constructs.Beam(
            config.source,
            config.target,
            effect,
            config.damagePerSecond or 0,
            config.duration or 0,
            config.targetPoint,
            config.visual,
            config.targetPointLocal,
            config.aimAngles,
            config.swayPhase,
            config.swayTime,
            config.swayBasis
        )
    )
end
