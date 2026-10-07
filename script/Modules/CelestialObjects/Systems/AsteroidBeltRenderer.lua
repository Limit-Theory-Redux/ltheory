local CameraManager     = require("Modules.Cameras.Managers.CameraManager")
local AsteroidFieldSystem = require("Modules.CelestialObjects.Systems.AsteroidFieldSystem")
local AsteroidMeshPool  = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local CoreComponents = require("Modules.Core.Components")
local Pipelines = require("Render.Pipelines")

--- AsteroidBeltRenderer — performant batch renderer for asteroid belts/rings.
--- Chunked instancing (article-derived): asteroids are partitioned into
--- angular chunks at generation time; per frame each chunk is culled by
--- its centroid, survivors are LOD-selected and collected into per-LOD
--- index lists (all in Rust: InstanceField), then flushed with ONE
--- instanced draw per group instead of one draw per
--- asteroid. Producer cost: ~chunk-count cull tests + per-visible-asteroid
--- LOD lookup + matrix fill, no per-asteroid draw/start/stop.
---@class AsteroidBeltRenderer
local AsteroidBeltRenderer = {}

local ffi = require('ffi')

--- Maximum render distance (squared)
local MAX_RENDER_DIST_SQ = 4e12
--- Maximum drawn asteroids per frame per belt/ring. Generous: the ring
--- alone has 1000 rocks and the belt 20000; the per-asteroid work is a
--- single 4-byte index write, so the real limiter is the GPU vertex
--- budget, not this counter. The benchmark lowers it via setMaxDrawnPerFrame
--- to hold frame time constant across scenes.
local MAX_DRAWN_PER_FRAME = 5000
--- Render-distance override (benchmark: camera orbits far outside the belt).
--- nil = derive from belt spread (game default).
local benchRenderDistSq = nil
--- Chunk count (angular sectors)
local CHUNK_COUNT = 32

--- Worker threads for the Rust belt cull: env LTHEORY_BELT_WORKERS wins
--- (testing), else Config.render.belt.workers. 0 or 1 = single-threaded.
local envWorkers = tonumber(os.getenv('LTHEORY_BELT_WORKERS') or '')
local function beltWorkers()
    if envWorkers then return envWorkers end
    local cfg = Config.render.belt
    return cfg and cfg.workers or 0
end

-- LOD distance ranges (RAW units) live in AsteroidMeshPool.LOD_RANGES
-- (single source of truth shared by the pool, generator and this file).

--- Expose the LOD ranges so the belt renderer can compute a stable LOD
--- index per asteroid (the LodMesh get() returns a fresh mesh clone each
--- call - unusable as a group key).
---@return table Array of { min, max } raw-unit ranges (LOD 0 first)
function AsteroidBeltRenderer.getLodRanges()
    return AsteroidMeshPool.getLodRanges()
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

    -- Vertical profile: a uniform offset (flat slab) looks uncanny; real
    -- rings are a gaussian core with a long sparse tail. ~1.5% of rocks
    -- get a 3-8x sigma boost -> a few visible outliers above/below the
    -- plane, the rest hug the mid-plane (engine RNG provides gaussian).
    local sigma = width * 0.08
    for i = 1, count do
        local angle = rng:getUniform() * math.pi * 2
        local radialOffset = (rng:getUniform() + rng:getUniform() - 1.0) * width * 0.5
        local vertOffset = rng:getGaussian() * sigma
        if rng:getUniform() < 0.015 then
            vertOffset = vertOffset * (3 + rng:getUniform() * 5) -- outlier tail
        end
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
--- cull chunks by centroid, collect survivors into LOD-keyed index
--- groups, flush one instanced draw per group.
---@param asteroidData table
---@param lodMesh LodMesh
---@return function renderFn
function AsteroidBeltRenderer.createRenderFn(asteroidData, lodMesh)
    local inst_shader = Cache.Shader('wvp_instanced_tex', 'material/asteroid_instanced')
    local asteroid_tex = Cache.Texture('rock')
    asteroid_tex:genMipmap() -- sampled with `Samplers.LinearMipRepeatAniso`

    -- Texture-fetch instancing: precompute each asteroid's static data ONCE
    -- (rotation*scale mat3 + world position + scale) into a flat float
    -- texture, 4 RGBA32F texels per asteroid (3 rotScale columns + pos/scale
    -- texel). The per-frame producer only writes 4-byte u32 INDICES into the
    -- static texture; the vertex shader texelFetches the transform and
    -- composes wp = rotScale*v + (worldPos - eye) itself. This is what lets
    -- 100k+ asteroids run on the main thread (4 B/instance vs 84 B).
    -- NOTE: cdata arrays are 0-indexed; texel base = (i-1)*4 for 1-based i.
    local nAst = #asteroidData
    local staticTexData = ffi.new('float[?]', nAst * 16)
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
        -- Pack 4 texels per asteroid (each RGBA32F texel = 4 floats):
        --   texel 0 = rotScale column 0 (m.m is column-major: m[0..2])
        --   texel 1 = rotScale column 1 (m[4..6])
        --   texel 2 = rotScale column 2 (m[8..10])
        --   texel 3 = world pos xyz + scale
        local tbase = (i - 1) * 4 * 4  -- float index of texel (i-1)*4
        staticTexData[tbase + 0] = m.m[0]
        staticTexData[tbase + 1] = m.m[1]
        staticTexData[tbase + 2] = m.m[2]
        staticTexData[tbase + 3] = 0
        staticTexData[tbase + 4] = m.m[4]
        staticTexData[tbase + 5] = m.m[5]
        staticTexData[tbase + 6] = m.m[6]
        staticTexData[tbase + 7] = 0
        staticTexData[tbase + 8] = m.m[8]
        staticTexData[tbase + 9] = m.m[9]
        staticTexData[tbase + 10] = m.m[10]
        staticTexData[tbase + 11] = 0
        staticTexData[tbase + 12] = a.px
        staticTexData[tbase + 13] = a.py
        staticTexData[tbase + 14] = a.pz
        staticTexData[tbase + 15] = a.scale
    end
    -- Upload: Bytes.FromData's generated Lua wrapper drops the size arg
    -- (slice bind bug), so call the raw C symbol directly with the byte
    -- view of the float buffer. One copy at generation time.
    --
    -- 2D layout: GL max texture width is 32k; 4 texels * 100k asteroids =
    -- 400k texels would be clamped to a 1D row. Use W x H with W = 4096
    -- (multiple of 4, so an asteroid's 4 texels never straddle a row) and
    -- texel index t -> (t % W, t / W). The shader computes the same via
    -- textureSize(). The byte buffer is zero-padded to W*H*16 so the GL
    -- upload never reads past it (400k texels vs 401,408 allocated).
    local STATIC_TEX_W = 4096
    local staticTexH = math.max(1, math.ceil(nAst * 4 / STATIC_TEX_W))
    local staticTexBytes = ffi.new('uint8_t[?]', STATIC_TEX_W * staticTexH * 16)
    ffi.copy(staticTexBytes, staticTexData, nAst * 64)
    local libphx = require('libphx').lib
    local staticTex = Tex2D.Create(STATIC_TEX_W, staticTexH, TexFormat.RGBA32F)
    staticTex:setDataBytes(
        Core.ManagedObject(libphx.Bytes_FromData(staticTexBytes, STATIC_TEX_W * staticTexH * 16), libphx.Bytes_Free),
        PixelFormat.RGBA, DataFormat.Float)

    -- Bind groups, created once: group 1 has the rock texture, group 2 the
    -- instance data texture (`texelFetch` ignores filtering). The per-draw
    -- block (`InstanceParams`) is allocated from the pass every frame.
    local InstanceParams = inst_shader:blockType('InstanceParams')
    local materialGroup = BindGroupDesc.Create(inst_shader, 1)
    materialGroup:texture('texDiffuse', asteroid_tex:view(), Samplers.LinearMipRepeatAniso)
    local materialBindGroup = Renderer:createBindGroup(materialGroup)
    local instanceGroup = BindGroupDesc.Create(inst_shader, 2)
    instanceGroup:texture('instanceDataTex', staticTex:view(), Samplers.Point)
    local instanceBindGroup = Renderer:createBindGroup(instanceGroup)

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

    -- Generation-time chunks: angular sectors. Structure-of-arrays for the
    -- hot loop: chunk membership as a flat int32 index array + prefix-sum
    -- offsets (no per-asteroid Lua table in the per-frame loop), centroids
    -- as flat float arrays.
    local chunkCounts = ffi.new('int32_t[?]', CHUNK_COUNT + 1)
    local chunkSize = (math.pi * 2) / CHUNK_COUNT
    local chunkOf = ffi.new('int32_t[?]', nAst + 1)
    for i = 1, nAst do
        local a = asteroidData[i]
        local ang = math.atan2(a.pz, a.px) -- -pi..pi
        local c = math.floor((ang + math.pi) / chunkSize) + 1
        c = math.max(1, math.min(CHUNK_COUNT, c))
        chunkOf[i] = c
        chunkCounts[c] = chunkCounts[c] + 1
    end

    -- Prefix-sum offsets into a flat index array, plus centroids
    local chunkOffsets = ffi.new('int32_t[?]', CHUNK_COUNT + 2)
    local chunkCx = ffi.new('float[?]', CHUNK_COUNT + 1)
    local chunkCy = ffi.new('float[?]', CHUNK_COUNT + 1)
    local chunkCz = ffi.new('float[?]', CHUNK_COUNT + 1)
    local total = 0
    for c = 1, CHUNK_COUNT do
        chunkOffsets[c] = total
        total = total + chunkCounts[c]
    end
    chunkOffsets[CHUNK_COUNT + 1] = total
    local chunkIndices = ffi.new('int32_t[?]', total + 1)
    local chunkFill = ffi.new('int32_t[?]', CHUNK_COUNT + 1)
    local chunkSumX = ffi.new('double[?]', CHUNK_COUNT + 1)
    local chunkSumY = ffi.new('double[?]', CHUNK_COUNT + 1)
    local chunkSumZ = ffi.new('double[?]', CHUNK_COUNT + 1)
    for i = 1, nAst do
        local a = asteroidData[i]
        local c = chunkOf[i]
        local p = chunkOffsets[c] + chunkFill[c]
        chunkIndices[p] = i
        chunkFill[c] = chunkFill[c] + 1
        chunkSumX[c] = chunkSumX[c] + a.px
        chunkSumY[c] = chunkSumY[c] + a.py
        chunkSumZ[c] = chunkSumZ[c] + a.pz
    end
    for c = 1, CHUNK_COUNT do
        local n = chunkCounts[c]
        if n > 0 then
            chunkCx[c] = chunkSumX[c] / n
            chunkCy[c] = chunkSumY[c] / n
            chunkCz[c] = chunkSumZ[c] / n
        end
    end

    -- Hand the flat per-asteroid data to the Rust bulk culler (InstanceField,
    -- engine/lib/phx/src/render/instance_field.rs). It copies the arrays
    -- once; the per-frame chunk cull, screen-size LOD selection, sub-pixel
    -- culling and draw cap all run there (optionally on worker threads).
    -- Asteroid and chunk indices are 0-based on the Rust side.
    local fPos = ffi.new('float[?]', nAst * 3 + 1)
    local fScales = ffi.new('float[?]', nAst + 1)
    for i = 1, nAst do
        local a = asteroidData[i]
        fPos[(i - 1) * 3 + 0] = a.px
        fPos[(i - 1) * 3 + 1] = a.py
        fPos[(i - 1) * 3 + 2] = a.pz
        fScales[i - 1] = a.scale
    end
    local fOffsets = ffi.new('uint32_t[?]', CHUNK_COUNT + 1)
    for k = 0, CHUNK_COUNT do fOffsets[k] = chunkOffsets[k + 1] end
    local fIndices = ffi.new('uint32_t[?]', total + 1)
    for p = 0, total - 1 do fIndices[p] = chunkIndices[p] - 1 end
    local fCentroids = ffi.new('float[?]', CHUNK_COUNT * 3)
    for c = 1, CHUNK_COUNT do
        fCentroids[(c - 1) * 3 + 0] = chunkCx[c]
        fCentroids[(c - 1) * 3 + 1] = chunkCy[c]
        fCentroids[(c - 1) * 3 + 2] = chunkCz[c]
    end
    local field = Core.ManagedObject(
        libphx.InstanceField_Create(fPos, nAst * 3, fScales, nAst, fOffsets, CHUNK_COUNT + 1,
            fIndices, total, fCentroids, CHUNK_COUNT * 3),
        libphx.InstanceField_Free)
    -- 0-based spawned indices, refilled each frame
    local spawnedBuf = ffi.new('uint32_t[?]', 64)
    local spawnedCap = 64

    -- groupOrder: LOD indices (0-based) in the order they first had instances.
    local groupOrder = {}
    -- Mesh per LOD index, fetched once per level (LodMesh:get clones; cache
    -- the clone per level instead of per asteroid). Query at the RANGE
    -- MIDPOINT, never at a boundary: LodMesh:get() is inclusive at both
    -- bounds and ranges share boundaries (e.g. LOD0 max 2000^2 == LOD1 min),
    -- so a boundary query would resolve to the PREVIOUS level's mesh.
    local lodMeshes = {}
    -- Screen-size LOD thresholds live in Rust (instance_field.rs): geometric
    -- bands (32, 16, 8, 4, 2, 1, 0.5, 0.25 px), sub-pixel rocks are culled.
    for i = 1, #AsteroidMeshPool.getLodRanges() do
        local r = AsteroidMeshPool.getLodRanges()[i]
        local midRaw = (r[1] + r[2]) * 0.5
        lodMeshes[i] = lodMesh:get(midRaw * midRaw)
    end
    libphx.InstanceField_SetLodCount(field, #lodMeshes)
    local seenLod = {}

    local PhysicsComponents = require("Modules.Physics.Components")

    return function(entity, blendMode)
        if blendMode ~= BlendMode.Disabled then return end
        if not lodMesh then return end

        local eye = CameraManager:getEye()
        if not eye then return end
        local eyeX, eyeY, eyeZ = eye.x, eye.y, eye.z

        -- Angular-size LOD factor: projected pixels per game-unit at
        -- distance 1, computed at a FIXED reference height (720p). LOD
        -- selection is then a pure function of the object's angular size
        -- (scale/dist), independent of the actual window resolution -
        -- rendering at 1080p does NOT push every asteroid one LOD band
        -- higher (2.3x more verts each) and does not let more rocks pass
        -- the sub-pixel cull. This keeps GPU vertex load ~constant across
        -- resolutions (same technique as GPU-driven LOD selection, done
        -- on CPU since we're GL 3.3). Squared: no sqrt/div in the loop.
        local fovRad = (Config.render.camera.fov or 70) * 0.5 * (math.pi / 180)
        local REF_H = 720
        local pxPerUnitSq = (REF_H / (2 * math.tan(fovRad))) ^ 2

        -- Origin = entity world position. Rings/belts attach to a parent
        -- body (planet) that ORBITS, so the render origin must track the
        -- parent's CURRENT transform every frame - the entity's own
        -- transform was set once at generation and goes stale as the
        -- parent moves (a static origin visibly detaches the ring from a
        -- moving planet). Standalone belts (no parent) use their own.
        local entPosX, entPosY, entPosZ = 0, 0, 0
        local originEntity = entity
        local parentCmp = entity:get(CoreComponents.Parent)
        if parentCmp then
            local p = parentCmp:getParent()
            if p and p:get(PhysicsComponents.Transform) then
                originEntity = p
            end
        end
        local transform = originEntity:get(PhysicsComponents.Transform)
        if transform then
            local p = transform:getPos()
            entPosX, entPosY, entPosZ = p.x, p.y, p.z
        end

        -- Spawned asteroids are real entities, not drawn here (0-based
        -- indices into the field). O(spawned) per frame.
        local spawnedIdx = AsteroidFieldSystem:getSpawnedIndices(entity)
        local nSpawned = #spawnedIdx
        if nSpawned > spawnedCap then
            while spawnedCap < nSpawned do spawnedCap = spawnedCap * 2 end
            spawnedBuf = ffi.new('uint32_t[?]', spawnedCap)
        end
        for s = 1, nSpawned do
            spawnedBuf[s - 1] = spawnedIdx[s] - 1
        end

        local fwdX, fwdY, fwdZ = 0, 0, -1
        local camForward = CameraManager:getForward()
        if camForward then fwdX, fwdY, fwdZ = camForward.x, camForward.y, camForward.z end

        -- Chunk cull (distance + view cone), screen-size LOD, draw cap:
        -- all in Rust, in f64, in chunk order. Worker threads fill their
        -- own lists; the join happens inside this call.
        libphx.InstanceField_SetWorkers(field, beltWorkers())
        -- The FFI rejects empty slices, so an empty spawned set is a clear
        if nSpawned > 0 then
            libphx.InstanceField_SetSpawned(field, spawnedBuf, nSpawned)
        else
            libphx.InstanceField_ClearSpawned(field)
        end
        libphx.InstanceField_Cull(field, eyeX, eyeY, eyeZ, fwdX, fwdY, fwdZ,
            entPosX, entPosY, entPosZ, pxPerUnitSq, renderDistSq, MAX_DRAWN_PER_FRAME)

        -- Groups keep the order in which a LOD first produced instances.
        for k = 0, libphx.InstanceField_GetLodOrderLen(field) - 1 do
            local lod = libphx.InstanceField_GetLodOrder(field, k)
            if not seenLod[lod] then
                seenLod[lod] = true
                groupOrder[#groupOrder + 1] = lod
            end
        end

        -- Flush: one instanced draw per (mesh variant, LOD level), all
        -- instances pulled from the static data texture by index
        if #groupOrder > 0 then
            local pass = Renderer:currentPass()
            pass:setPipeline(Pipelines.get(inst_shader, Pipelines.Opaque))
            pass:setBindGroup(1, materialBindGroup)
            pass:setBindGroup(2, instanceBindGroup)
            -- Camera-relative origin, subtracted in DOUBLE precision here
            -- (Lua numbers are f64): at AU-scale coordinates the origin
            -- and eye are ~1e7 GU, and float32 cannot represent
            -- (1e7 - 1e7 + 1000) - the ULP at 1e7 is ~1 GU, so the shader
            -- doing origin - eye itself would jitter every asteroid by up
            -- to a GU per frame (larger than a 1 GU rock). The shader
            -- receives ONE small-magnitude vec3 and adds the baked
            -- position on top - no cancellation.
            local params = pass:alloc(InstanceParams)
            params.originRelEye.x = entPosX - eyeX
            params.originRelEye.y = entPosY - eyeY
            params.originRelEye.z = entPosZ - eyeZ
            for i = 1, #groupOrder do
                local lod = groupOrder[i]
                if libphx.InstanceField_GetCount(field, lod) > 0 then
                    libphx.InstanceField_Draw(field, pass, Renderer, lod, lodMeshes[lod + 1])
                end
            end
        end
    end
end

return AsteroidBeltRenderer
