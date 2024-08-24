-- Generator Utilities
local GenUtil = require('Systems.Gen.GenUtil')

-- Utilities
local QuickProfiler = require('_ECS_WIP_TEMP.Shared.Tools.QuickProfiler')
local profiler = QuickProfiler("Generate Asteroid", false, false, false)

---@param seed integer
---@return Mesh
local function GenerateAsteroidMesh(seed)
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
    local dMin = 0
    local dMax = 1
    local lac = 1.5

    for i = 1, 8 do
        -- Create Mesh from Tex3D
        local density = GenUtil.ShaderToTex3D(shaderState, floor(res), TexFormat.R32F)
        -- Get Signed Distance Field (SDF)
        local field = SDF.FromTex3D(density)
        field:computeNormals()
        -- Get Mesh from SDF
        local mesh = field:toMesh()
        mesh:computeOcclusion(density, 0.1)
        mesh:center()
        -- Add Mesh to AsteroidMesh
        asteroidMesh:add(mesh, dMin, dMax)
        
        -- Free Memory Used for creation of Mesh
        field:free()
        density:free()

        --TODO: Math?
        res = res / lac
        dMin = dMax
        dMax = dMax * lac * sqrt(2.0)
    end

    --TODO: Replace use of RNG, If RNG Cache is created
    -- Free RNG from memory
    rng:free()
    -- Stop profiler
    profiler:stop()
    return asteroidMesh
end

return GenerateAsteroidMesh
