-- Entities
local Camera = require("_ECS_WIP_TEMP.Entities.Rendering.Camera")            --!temp path
local Asteroid = require("_ECS_WIP_TEMP.Entities.CelestialObjects.Asteroid") --!temp path
-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage")         --!temp path
---@type CameraSystem
local CameraSystem = require("_ECS_WIP_TEMP.Systems.Rendering.CameraSystem")
-- Utilities
local Material = require("_ECS_WIP_TEMP.Shared.Rendering.Material")           --!temp path
local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar") --!temp path
local Log = require("Core.Util.Log")
local Inspect = require("Core.Util.Inspect")

local RenderingTest = require('States.Application')

---@param shaderState ShaderState
---@param uniformInt integer
---@param transfromA TransformComponent
---@param transformB TransformComponent
local setArbValVec3f = function(shaderState, uniformInt, transfromA, transformB)
    local val = transfromA:getScale() + transformB:getScale()
    shaderState:shader():iSetFloat3(uniformInt, val.x, val.y, val.z)
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onInit()
    -- Mark as initialized
    self.initialized = true

    -- Spawn CameraEntity
    local camera = Camera()
    local entityInfo = GlobalStorage:storeEntity(camera)
    CameraSystem:setCamera(entityInfo)

    local rng = RNG.Create(0):managed()

    -- Spawn a Asteroid A
    local a = Asteroid(rng:get64())
    local aTransform = a:findComponentByName("PhysicsTransform")
    ---@cast aTransform TransformComponent
    --Log.Warn("Asteroid A Entity: " .. Inspect(A) .. "\nTransform Component: " .. Inspect(A_Transform) .. "\n\n")

    -- Spawn a Asteroid B
    local b = Asteroid(rng:get64())
    local bTransform = b:findComponentByName("PhysicsTransform")
    ---@cast bTransform TransformComponent
    --Log.Warn("Asteroid B Entity: " .. Inspect(B) .. "\nTransform Component: " .. Inspect(B_Transform))

    print(a, b, aTransform, bTransform)

    ---@type Material
    local mat = Material('pulse', 'billboard/axis', 'effect/pulsetail')
    local autoShaderVar = mat:addAutoShaderVar("color", setArbValVec3f)
    if autoShaderVar then autoShaderVar:setShaderVar(mat.shaderState, aTransform, bTransform) end
end

return RenderingTest
