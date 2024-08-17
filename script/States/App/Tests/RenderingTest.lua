local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path
local Universe = require("_ECS_WIP_TEMP.Systems.Universe")           --!temp path
local Material = require("_ECS_WIP_TEMP.Shared.Rendering.Material")  --!temp path
local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar")  --!temp path
local Asteroid = require("_ECS_WIP_TEMP.Entities.CelestialObjects.Asteroid")         --!temp path
local Log = require("Core.Util.Log")
local Inspect = require("Core.Util.Inspect")

local RenderingTest = require('States.Application')

---@param shaderState ShaderState
---@param uniformInt integer
---@param args table
local setArbValVec3f = function(shaderState, uniformInt, args)
    local val = args[1]:getScale()+args[2]:getScale()
    shaderState:shader():iSetFloat3(uniformInt, val.x, val.y, val.z)
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onInit()

    ---@type GlobalStorage
    GlobalStorage:initStorage();
    -- Mark as initialized
    self.initialized = true

    -- Spawn a Asteroid A
    local A = Asteroid(RNG.Create(0):managed():get64())
    ---@type TransformComponent
    local A_Transform = A:findComponentByName("PhysicsTransform")
    -- Spawn a Asteroid B
    local B = Asteroid(RNG.Create(0):managed():get64())
    ---@type TransformComponent
    local B_Transform = B:findComponentByName("PhysicsTransform")
    
    ---@type Material
    local mat = Material('pulse', 'billboard/axis', 'effect/pulsetail')
    Log.Warn("Material Init Debug: " .. Inspect(mat))
    local autoShaderVar = mat:addAutoShaderVar("color", setArbValVec3f)
    --Log.Warn("Material Added AutoShaderVar Debug")
    --Log.Warn(Inspect(mat))
    if autoShaderVar then autoShaderVar:render(mat.shaderState, {A_Transform, B_Transform}) end
    Log.Warn("Material Added AutoShaderVar Debug: " .. Inspect(mat))




    --[[
    ---@param archetype EntityArchetype
    ---@param entities table<Entity>
    for archetype, entities in ipairs(GlobalStorage:getEntities()) do --!temp fix globalStorage
        ---@param entity Entity
        for _, entity in pairs(entities) do
            local nameComponent = entity:findComponentByName("Name")
            ---@cast nameComponent NameComponent
            print(nameComponent:getName() .. " (" .. Enums.EntityArchetype:getName(archetype) .. ")")

            for component in entity:iterComponents() do
                if component:getComponentName() ~= "NameComponent" then
                     print(" - " .. component:getComponentName())
               end
            end
        end
    end
    --]]

    --Log.Warn(Inspect(GlobalStorage))
end

return RenderingTest