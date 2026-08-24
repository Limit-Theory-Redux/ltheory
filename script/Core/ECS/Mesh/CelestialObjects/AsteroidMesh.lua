-- Generator Utilities
local GenUtil = require('Core.ECS.Mesh.Util.GenUtil')

-- Utilities
local QuickProfiler = require('Shared.Tools.QuickProfiler')
local profiler = QuickProfiler("Generate Asteroid", false, false, false)

---@param seed integer
---@param lodRanges table|nil Array of { min, max } raw-unit distance
--- ranges (LOD 0 first); defaults to the shared AsteroidMeshPool.LOD_RANGES
--- when omitted (lazy require avoids a load-order cycle).
---@return Mesh
local function GenerateAsteroidMesh(seed, lodRanges)
    -- Start Profiler
    profiler:start()

    -- Create RNG from seed
    --TODO: Replace use of RNG, If RNG Cache is created
    local rng = RNG.Create(seed)
    -- Create Mesh
    local asteroidMesh = LodMesh.Create()
    -- Create ShaderState
    --TODO: Replace use of Cache.Shader
    local shader = Cache.Shader('identity', 'sdf/asteroid')
    local shaderState = ShaderState.Create(shader)

    -- Set Shader Uniforms
    shaderState:setInt('octaves', 8)
    shaderState:setFloat('seed', rng:getUniformRange(0, 1000))
    shaderState:setFloat('smoothness', 2.5)

    local res = 96 -- resolution
    local lac = 1.5

    -- LOD distances come from AsteroidMeshPool.LOD_RANGES (single source of
    -- truth): each LOD covers a raw-unit range from the camera; LodMesh:add
    -- squares these internally, and get() takes the squared distance.
    lodRanges = lodRanges
        or require("Modules.CelestialObjects.Systems.AsteroidMeshPool").getLodRanges()

    for i = 1, 8 do
        local density = GenUtil.ShaderToTex3D(shaderState, floor(res), TexFormat.R32F)
        local field = SDF.FromTex3D(density)
        field:computeNormals()
        local mesh = field:toMesh()
        mesh:computeOcclusion(density, 0.1)
        mesh:center()
        asteroidMesh:add(mesh, lodRanges[i][1], lodRanges[i][2])
        res = res / lac
    end

    -- Stop profiler
    profiler:stop()
    return asteroidMesh
end

return GenerateAsteroidMesh
