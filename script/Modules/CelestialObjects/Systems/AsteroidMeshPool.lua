--- AsteroidMeshPool — shared pool of asteroid LodMeshes.
--- Loads pre-generated meshes from res/mesh/asteroid/.
--- If not found on disk, generates and saves them for next time.
---@class AsteroidMeshPool
local AsteroidMeshPool = {}

local pool = {}
local poolSize = 0

local MESH_DIR = "res/mesh/asteroid"
local LOD_COUNT = 8

--- Initialize the pool with N asteroid mesh variants
---@param count number Number of variants (default 8)
---@param baseSeed number Base seed for generation (default 42)
function AsteroidMeshPool:init(count, baseSeed)
    count = count or 8
    baseSeed = baseSeed or 42

    if poolSize >= count then return end

    Log.Info("AsteroidMeshPool: loading %d asteroid mesh variants...", count)

    for i = 1, count do
        local lodMesh = LodMesh.Create()
        local allLoaded = true

        -- Try loading pre-cached LOD meshes from disk
        for lod = 0, LOD_COUNT - 1 do
            local path = string.format("%s/asteroid_%02d_lod%d.mesh", MESH_DIR, i, lod)
            if File.Exists(path) then
                local mesh = Mesh.Load(path)
                -- LOD distance ranges (RAW units; LodMesh:add squares them
                -- internally, matching AsteroidMesh.lua). LOD 0 = highest
                -- detail, used up to 2000 units from the camera.
                local lodRanges = {
                    { 0,       2000 },
                    { 2000,    8000 },
                    { 8000,    30000 },
                    { 30000,   100000 },
                    { 100000,  500000 },
                    { 500000,  2000000 },
                    { 2000000, 10000000 },
                    { 10000000, 1e16 },
                }
                local r = lodRanges[lod + 1]
                lodMesh:add(mesh, r[1], r[2])
            else
                allLoaded = false
                break
            end
        end

        if allLoaded then
            pool[i] = lodMesh
            Log.Info("  Variant %d: loaded from cache", i)
        else
            -- Generate and save
            Log.Info("  Variant %d: generating (first run)...", i)
            local seed = baseSeed + i * 7919
            local GenerateAsteroidMesh = require("Core.ECS.Mesh.CelestialObjects.AsteroidMesh")
            pool[i] = GenerateAsteroidMesh(seed)

            -- Save each LOD level to disk for next time
            -- Query by the MIDPOINT of each RAW distance range, squared
            -- (get() takes squared distance; add() squares the range ends).
            local lodRanges = {
                { 0,       2000 },
                { 2000,    8000 },
                { 8000,    30000 },
                { 30000,   100000 },
                { 100000,  500000 },
                { 500000,  2000000 },
                { 2000000, 10000000 },
                { 10000000, 1e16 },
            }
            for lod = 0, LOD_COUNT - 1 do
                local r = lodRanges[lod + 1]
                local mid = (r[1] + r[2]) * 0.5
                local mesh = pool[i]:get(mid * mid)
                if mesh then
                    local path = string.format("%s/asteroid_%02d_lod%d.mesh", MESH_DIR, i, lod)
                    mesh:save(path)
                end
            end
            Log.Info("  Variant %d: saved to cache", i)
        end
    end

    poolSize = count
    Log.Info("AsteroidMeshPool: %d variants ready", poolSize)
end

--- Get a mesh variant by index (1-based, wraps around)
---@param index number
---@return LodMesh
function AsteroidMeshPool:get(index)
    if poolSize == 0 then self:init() end
    return pool[((index - 1) % poolSize) + 1]
end

--- Get a mesh variant based on a seed (deterministic selection)
---@param seed number
---@return LodMesh
function AsteroidMeshPool:getFromSeed(seed)
    if poolSize == 0 then self:init() end
    local idx = (math.abs(tonumber(seed) or 0) % poolSize) + 1
    return pool[idx]
end

--- Get pool size
---@return number
function AsteroidMeshPool:getSize()
    return poolSize
end

--- Clear the pool
function AsteroidMeshPool:clear()
    pool = {}
    poolSize = 0
end

return AsteroidMeshPool
