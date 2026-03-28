local Registry = require("Core.ECS.Registry")
local SpaceStationEntity = require('Modules.Constructs.Entities.SpaceStationEntity')
local StationComponents = require("Modules.Constructs.Components")
local Physics = require("Modules.Physics.Components")

-- Legacy Generator Imports
local GenerateStationOld = require('Legacy.Systems.Gen.StationOld')

local Materials = require("Shared.Registries.Materials")

---@class StationGenerator
---@overload fun(): StationGenerator
local StationGenerator = Class("StationGenerator", function()
    -- Station generator initialization
end)

---Generate station mesh
---@param seed integer
---@return Mesh
local function generateStationMesh(seed)
    Log.Debug("@@@ StationGenerator:(create) - seed = %s", seed)

    Profiler.Begin('Gen.Station')
    local result = GenerateStationOld(seed)
    Profiler.End()

    return result
end

---@class StationGenConfig
---@field material Material|nil
---@field position Position|nil
---@field scale number|nil
---@field rotation Quat|nil
---@field isKinematic boolean|nil

---Create a Station with automatic mesh generation
---@param seed integer
---@param config StationGenConfig|nil Configuration {material, stationType, position, scale, rotation, isKinematic}
---@param stats StationStats|nil
---@return Entity
function StationGenerator:createStation(seed, config, stats)
    config = config or {}

    -- Generate mesh
    local mesh = generateStationMesh(seed)
    mesh:computeNormals()
    mesh:computeAO(0.3 * mesh:getRadius())

    -- Get or create material
    local material = config.material or (Materials.Metal and Materials.Metal())

    local meshes = { { mesh = mesh, material = material } }

    -- Create entity
    local entity = SpaceStationEntity(seed, meshes, stats)

    -- Setup physics if configured
    if config.position or config.scale or config.rotation or config.isKinematic ~= nil then
        local rbCmp = entity:get(Physics.RigidBody)
        if rbCmp then
            local rb = RigidBody.CreateConvexDecompositionFromMesh(mesh)

            if config.isKinematic ~= nil then
                rb:setKinematic(config.isKinematic)
            else
                -- Stations are typically kinematic (stationary)
                rb:setKinematic(true)
            end

            if config.position then
                rb:setPos(config.position)
            end

            if config.scale then
                rb:setScale(config.scale)
            end

            if config.rotation then
                rb:setRot(config.rotation)
            end

            rbCmp:setRigidBody(rb)
        end
    end

    return entity
end

---Create multiple stations in a cluster
---@param baseSeed integer
---@param count integer
---@param config StationGenConfig|nil
---@return Entity[]
function StationGenerator:createCluster(baseSeed, count, config)
    config = config or {}
    local stations = {}
    local rng = RNG.Create(baseSeed)

    for i = 1, count do
        local seed = baseSeed + i
        local stationConfig = {
            material = config.material,
            isKinematic = config.isKinematic
        }

        -- Generate positions in a cluster if base position provided
        if config.position then
            local offset = Vec3f(
                rng:getUniformRange(-100, 100),
                rng:getUniformRange(-100, 100),
                rng:getUniformRange(-100, 100)
            )
            stationConfig.position = config.position + offset
        end

        -- Vary scale slightly
        if config.scale then
            local scaleVariation = rng:getUniformRange(0.8, 1.2)
            if type(config.scale) == "number" then
                stationConfig.scale = config.scale * scaleVariation
            else
                stationConfig.scale = config.scale * scaleVariation
            end
        end

        -- Random rotation
        stationConfig.rotation = Quat.FromAxisAngle(
            Vec3f(rng:getUniform(), rng:getUniform(), rng:getUniform()):normalize(),
            rng:getUniformRange(0, math.pi * 2)
        )

        table.insert(stations, self:createStation(seed, stationConfig))
    end

    return stations
end

return StationGenerator()
