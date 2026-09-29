local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")

local beamMesh
local beamShader

local function getPosition(entity)
    if not entity or not entity:isValid() then
        return nil
    end

    local rigidBody = entity:get(Physics.RigidBody)
    if rigidBody and rigidBody:getRigidBody() then
        local position = rigidBody:getRigidBody():getPos()
        return Vec3f(position.x, position.y, position.z)
    end

    local transform = entity:get(Physics.Transform)
    local position = transform and transform:getPos() or nil
    if not position then
        return nil
    end

    return Vec3f(position.x, position.y, position.z)
end

local function render(entity, blendMode)
    if blendMode ~= BlendMode.Additive then
        return
    end

    local beam = entity:get(Constructs.Beam)
    local startPosition = getPosition(beam:getSource())
    local targetPoint = beam:getTargetPoint()
    local endPosition = targetPoint
        and Vec3f(targetPoint.x, targetPoint.y, targetPoint.z)
        or getPosition(beam:getTarget())
    if not startPosition or not endPosition then
        return
    end

    local axis = endPosition - startPosition
    local length = axis:length()
    if length <= 1e-6 then
        return
    end

    if not beamMesh then
        beamMesh = Gen.Primitive.Billboard(-1, 0, 1, 1)
        beamShader = Cache.Shader("billboard/axis", "effect/beam")
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

    beamShader:start()
    beamShader:setFloat3("color", color.r, color.g, color.b)
    beamShader:setFloat("alpha", 1.0)
    beamShader:setFloat2("size", visual.beamWidth or 0.008, length)
    beamShader:setFloat("seed", 0.0)
    beamShader:setMatrix("mWorld", matrix)
    beamMesh:draw()
    beamShader:stop()
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
