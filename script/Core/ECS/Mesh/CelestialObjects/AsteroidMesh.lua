-- Generator Utilities
local GenUtil = require('Core.ECS.Mesh.Util.GenUtil')

-- Utilities
local QuickProfiler = require('Shared.Tools.QuickProfiler')
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
    local lac = 1.5

    -- LOD distances (squared) — scaled for asteroids at game scale (50-500 units)
    -- Each LOD covers a range of squared distances from camera
    local lodRanges = {
        { 0,      500 * 500 },         -- LOD 0: highest detail, < 500 units
        { 500^2,  2000^2 },            -- LOD 1
        { 2000^2, 8000^2 },            -- LOD 2
        { 8000^2, 30000^2 },           -- LOD 3
        { 30000^2, 100000^2 },         -- LOD 4
        { 100000^2, 500000^2 },        -- LOD 5
        { 500000^2, 2000000^2 },       -- LOD 6
        { 2000000^2, 1e16 },           -- LOD 7: lowest detail, very far
    }

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
