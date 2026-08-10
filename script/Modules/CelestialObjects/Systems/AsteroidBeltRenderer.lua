local CameraManager     = require("Modules.Cameras.Managers.CameraManager")

--- AsteroidBeltRenderer — performant batch renderer for asteroid belts/rings.
--- Chunked instancing (article-derived): asteroids are partitioned into
--- angular chunks at generation time; per frame each chunk is culled by
--- its centroid, survivors are LOD-selected and collected into reusable
--- InstanceData groups keyed by (mesh variant, LOD level), then flushed
--- with ONE DrawInstancedWithData per group instead of one draw per
--- asteroid. Producer cost: ~chunk-count cull tests + per-visible-asteroid
--- LOD lookup + matrix fill, no per-asteroid draw/start/stop.
---@class AsteroidBeltRenderer
local AsteroidBeltRenderer = {}

local ffi = require('ffi')

--- Maximum render distance (squared)
local MAX_RENDER_DIST_SQ = 4e12
--- Maximum drawn asteroids per frame per belt/ring
local MAX_DRAWN_PER_FRAME = 200
--- Render-distance override (benchmark: camera orbits far outside the belt).
--- nil = derive from belt spread (game default).
local benchRenderDistSq = nil
--- Chunk count (angular sectors)
local CHUNK_COUNT = 32

--- LOD distance ranges (RAW units) shared by the pool, generator and
--- belt renderer. LodMesh:add() squares these internally; get() takes
--- the squared distance. LOD 0 = highest detail (res 96, 16k verts).
local LOD_RANGES = {
    { 0,       2000 },
    { 2000,    8000 },
    { 8000,    30000 },
    { 30000,   100000 },
    { 100000,  500000 },
    { 500000,  2000000 },
    { 2000000, 10000000 },
    { 10000000, 1e16 },
}

--- Expose the LOD ranges so the belt renderer can compute a stable LOD
--- index per asteroid (the LodMesh get() returns a fresh mesh clone each
--- call - unusable as a group key).
---@return table Array of { min, max } raw-unit ranges (LOD 0 first)
function AsteroidBeltRenderer.getLodRanges()
    return LOD_RANGES
end

--- Allow benchmarks/tests to raise the per-frame draw cap (module-local).
---@param n number
function AsteroidBeltRenderer.setMaxDrawnPerFrame(n)
    MAX_DRAWN_PER_FRAME = n or 200
end

--- Allow benchmarks/tests to override the render-distance cutoff.
---@param distSq number Squared distance; asteroids beyond this are not drawn
function AsteroidBeltRenderer.setRenderDistSq(distSq)
    benchRenderDistSq = distSq
end

--- Generate asteroid transforms for a belt
---@param params table { orbitRadius, width, count, inclination, seed, minScale, maxScale }
---@return table asteroids Array of { px, py, pz, rotSeed, scale }
function AsteroidBeltRenderer.generateBeltAsteroids(params)
    local rng = RNG.Create(params.seed or 12345)
    local asteroids = {}

    local orbitRadius = params.orbitRadius
    local width = params.width or orbitRadius * 0.1
    local count = params.count or 500
    local inclination = params.inclination or 0
    local minScale = params.minScale or 10
    local maxScale = params.maxScale or 200

    for i = 1, count do
        local angle = rng:getUniform() * math.pi * 2
        local radialOffset = (rng:getUniform() + rng:getUniform() - 1.0) * width * 0.5
        local vertOffset = (rng:getUniform() - 0.5) * width * 0.3
        vertOffset = vertOffset + math.sin(angle) * math.sin(inclination) * (orbitRadius + radialOffset)

        local r = orbitRadius + radialOffset
        local px = math.cos(angle) * r
        local py = vertOffset
        local pz = math.sin(angle) * r

        local scale = minScale + (maxScale - minScale) * rng:getUniform() * rng:getUniform()
        local rotSeed = rng:get31()

        table.insert(asteroids, {
            px = px, py = py, pz = pz,
            rotSeed = rotSeed,
            scale = scale,
        })
    end

    return asteroids
end

--- Create a render function for a belt entity.
--- Partition asteroids into angular chunks at generation time; per frame
--- cull chunks by centroid, collect survivors into LOD-keyed InstanceData
--- groups, flush one instanced draw per group.
---@param asteroidData table
---@param lodMesh LodMesh
---@return function renderFn
function AsteroidBeltRenderer.createRenderFn(asteroidData, lodMesh)
    local inst_shader = Cache.Shader('wvp_instanced', 'material/asteroid_instanced')
    local asteroid_tex = Cache.Texture('rock')

    -- Precompute each asteroid's static rotation*scale matrix ONCE (the
    -- rotation and scale never change; only the translation depends on
    -- the eye each frame). Stored as a flat float[16] per asteroid; per
    -- frame we memcpy it and patch the translation column - no Matrix
    -- allocation in the hot loop. NOTE: cdata arrays are 0-indexed, so
    -- allocate nAst+1 to keep 1-based Lua indexing (asteroidData[i]).
    local nAst = #asteroidData
    local staticMats = ffi.new('float[?][16]', nAst + 1)
    for i = 1, nAst do
        local a = asteroidData[i]
        local rng = RNG.Create(a.rotSeed)
        local ax = rng:getUniform() - 0.5
        local ay = rng:getUniform() - 0.5
        local az = rng:getUniform() - 0.5
        local len = math.sqrt(ax * ax + ay * ay + az * az)
        if len > 0.001 then ax, ay, az = ax / len, ay / len, az / len end
        local rot = Quat.FromAxisAngle(Vec3f(ax, ay, az), rng:getUniform() * math.pi * 2)
        local m = Matrix.FromPosRotScale(Vec3f(0, 0, 0), rot, a.scale)
        ffi.copy(staticMats[i], m.m, ffi.sizeof('float') * 16)
    end

    -- Render distance proportional to belt spread (capped), or the
    -- benchmark override when set (camera orbits far outside the belt)
    local renderDistSq = benchRenderDistSq
    if not renderDistSq then
        local maxOrbitR = 0
        for i = 1, math.min(100, #asteroidData) do
            local a = asteroidData[i]
            local r = math.sqrt(a.px * a.px + a.pz * a.pz)
            if r > maxOrbitR then maxOrbitR = r end
        end
        renderDistSq = math.min(MAX_RENDER_DIST_SQ, (maxOrbitR * 3) ^ 2)
    end

    -- Generation-time chunks: angular sectors. Each chunk holds indices +
    -- a centroid (world-space offset) used for cheap per-frame culling.
    local chunks = {}
    for c = 1, CHUNK_COUNT do
        chunks[c] = { indices = {}, cx = 0, cy = 0, cz = 0 }
    end
    local chunkSize = (math.pi * 2) / CHUNK_COUNT
    for i, a in ipairs(asteroidData) do
        local ang = math.atan2(a.pz, a.px) -- -pi..pi
        local c = math.floor((ang + math.pi) / chunkSize) + 1
        c = math.max(1, math.min(CHUNK_COUNT, c))
        local ch = chunks[c]
        ch.indices[#ch.indices + 1] = i
        ch.cx = ch.cx + a.px
        ch.cy = ch.cy + a.py
        ch.cz = ch.cz + a.pz
    end
    for c = 1, CHUNK_COUNT do
        local ch = chunks[c]
        local n = #ch.indices
        if n > 0 then
            ch.cx = ch.cx / n
            ch.cy = ch.cy / n
            ch.cz = ch.cz / n
        end
    end

    -- Per-frame group state. Keyed by LOD INDEX (stable) not the mesh
    -- object: LodMesh:get() returns a fresh mesh clone per call, which
    -- would leak a new group (and instance array) every frame.
    -- groups[lodIndex] = { mesh, capacity, count, instances }
    local groups = {}
    local groupOrder = {}
    -- Mesh per LOD index, fetched once per level (LodMesh:get clones; cache
    -- the clone per level instead of per asteroid). Query at the RANGE
    -- MIDPOINT, never at a boundary: LodMesh:get() is inclusive at both
    -- bounds and ranges share boundaries (e.g. LOD0 max 2000^2 == LOD1 min),
    -- so a boundary query would resolve to the PREVIOUS level's mesh.
    local lodMeshes = {}
    -- Flat squared upper bounds (float[8], 0-indexed cdata) for a JIT-
    -- friendly LOD scan. Ranges are contiguous in ascending order and share
    -- boundaries, so "first li with distNorm <= lodMaxSq[li]" selects the
    -- same level as the inclusive-both-bounds table scan (and matches
    -- LodMesh:get's first-match-wins semantics at shared boundaries).
    local lodMaxSq = ffi.new('float[8]')
    for i = 1, #LOD_RANGES do
        local r = LOD_RANGES[i]
        lodMaxSq[i - 1] = r[2] * r[2]
        local midRaw = (r[1] + r[2]) * 0.5
        lodMeshes[i] = lodMesh:get(midRaw * midRaw)
    end

    local PhysicsComponents = require("Modules.Physics.Components")

    return function(entity, blendMode)
        if blendMode ~= BlendMode.Disabled then return end
        if not lodMesh then return end

        local eye = CameraManager:getEye()
        if not eye then return end
        local eyeX, eyeY, eyeZ = eye.x, eye.y, eye.z

        -- Entity world position (rings: planet position)
        local entPosX, entPosY, entPosZ = 0, 0, 0
        local transform = entity:get(PhysicsComponents.Transform)
        if transform then
            local p = transform:getPos()
            entPosX, entPosY, entPosZ = p.x, p.y, p.z
        end

        -- Reset per-frame group counts
        for i = 1, #groupOrder do
            groups[groupOrder[i]].count = 0
        end

        local drawn = 0
        local done = false

        -- Chunk pass: cull by centroid, then collect survivors
        for c = 1, CHUNK_COUNT do
            if done then break end
            local ch = chunks[c]
            local nIdx = #ch.indices
            if nIdx == 0 then goto next_chunk end

            -- Centroid eye-distance (world coords = entity pos + centroid)
            local rcx = entPosX + ch.cx - eyeX
            local rcy = entPosY + ch.cy - eyeY
            local rcz = entPosZ + ch.cz - eyeZ
            local cDistSq = rcx * rcx + rcy * rcy + rcz * rcz
            if cDistSq > renderDistSq then goto next_chunk end

            for j = 1, nIdx do
                if drawn >= MAX_DRAWN_PER_FRAME then done = true break end
                local i = ch.indices[j]
                local a = asteroidData[i]
                if not a.spawned then
                    local rx = entPosX + a.px - eyeX
                    local ry = entPosY + a.py - eyeY
                    local rz = entPosZ + a.pz - eyeZ
                    local distSq = rx * rx + ry * ry + rz * rz

                    -- LOD selection, normalized by scale (matches legacy path)
                    -- Flat-array scan: first level whose squared upper bound
                    -- holds distNorm (ranges are contiguous ascending, so the
                    -- lower bound is implied). JIT-friendly: no table lookups.
                    local s = a.scale
                    local distNorm = distSq / (s * s)
                    local li = 1
                    while li < 8 and distNorm > lodMaxSq[li - 1] do
                        li = li + 1
                    end
                    if distNorm <= lodMaxSq[li - 1] then
                        local mesh = lodMeshes[li]
                        if mesh then
                            -- Camera-relative world matrix: static
                            -- rotation*scale (precomputed) + eye-relative
                            -- translation patched in. Unrolled writes keep
                            -- the loop inside the LuaJIT trace (ffi.copy is
                            -- a C call that aborts it).
                            local g = groups[li]
                            if not g then
                                g = { mesh = mesh, capacity = 64, count = 0, instances = ffi.new('InstanceData[64]') }
                                groups[li] = g
                                groupOrder[#groupOrder + 1] = li
                            end
                            if g.count >= g.capacity then
                                local newCap = g.capacity * 2
                                local newArr = ffi.new('InstanceData[?]', newCap)
                                ffi.copy(newArr, g.instances, g.count * ffi.sizeof('InstanceData'))
                                g.instances = newArr
                                g.capacity = newCap
                            end

                            local inst = g.instances[g.count]
                            local sm = staticMats[i]
                            local mm = inst.model_matrix
                            mm[0] = sm[0]; mm[1] = sm[1]; mm[2] = sm[2]; mm[3] = sm[3]
                            mm[4] = sm[4]; mm[5] = sm[5]; mm[6] = sm[6]; mm[7] = sm[7]
                            mm[8] = sm[8]; mm[9] = sm[9]; mm[10] = sm[10]; mm[11] = sm[11]
                            mm[12] = rx; mm[13] = ry; mm[14] = rz; mm[15] = sm[15]
                            inst.scale = s
                            g.count = g.count + 1
                            drawn = drawn + 1
                        end
                    end
                end
            end

            ::next_chunk::
        end

        -- Flush: one instanced draw per (mesh variant, LOD level)
        if #groupOrder > 0 then
            inst_shader:start()
            inst_shader:setTex2D('texDiffuse', asteroid_tex)
            for i = 1, #groupOrder do
                local g = groups[groupOrder[i]]
                if g.count > 0 then
                    g.mesh:drawInstancedWithData(g.instances, g.count)
                end
            end
            inst_shader:stop()
        end
    end
end

return AsteroidBeltRenderer
