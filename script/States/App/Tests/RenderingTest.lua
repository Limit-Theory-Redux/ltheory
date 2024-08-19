-- Entities
local Camera = require("_ECS_WIP_TEMP.Entities.Rendering.Camera")            --!temp path
local Asteroid = require("_ECS_WIP_TEMP.Entities.CelestialObjects.Asteroid") --!temp path
-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage")         --!temp path
local Universe = require("_ECS_WIP_TEMP.Systems.Universe")                   --!temp path
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
    GlobalStorage:storeEntity(camera)

    local rng = RNG.Create(0):managed()

    -- Spawn a Asteroid A
    local A = Asteroid(rng:get64())
    ---@type TransformComponent
    local A_Transform = A:findComponentByName("PhysicsTransform")
    --Log.Warn("Asteroid A Entity: " .. Inspect(A) .. "\nTransform Component: " .. Inspect(A_Transform) .. "\n\n")

    -- Spawn a Asteroid B
    local B = Asteroid(rng:get64())
    ---@type TransformComponent
    local B_Transform = B:findComponentByName("PhysicsTransform")
    --Log.Warn("Asteroid B Entity: " .. Inspect(B) .. "\nTransform Component: " .. Inspect(B_Transform))

    print(A, B, A_Transform, B_Transform)

    ---@type Material
    local mat = Material('pulse', 'billboard/axis', 'effect/pulsetail')
    local autoShaderVar = mat:addAutoShaderVar("color", setArbValVec3f)
    if autoShaderVar then autoShaderVar:setShaderVar(mat.shaderState, A_Transform, B_Transform) end
end

return RenderingTest
