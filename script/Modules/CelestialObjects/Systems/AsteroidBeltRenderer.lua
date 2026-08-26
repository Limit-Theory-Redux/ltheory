local CameraManager     = require("Modules.Cameras.Managers.CameraManager")
local AsteroidFieldSystem = require("Modules.CelestialObjects.Systems.AsteroidFieldSystem")
local AsteroidMeshPool  = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local CoreComponents = require("Modules.Core.Components")

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
--- cull chunks by centroid, collect survivors into LOD-keyed InstanceData
--- groups, flush one instanced draw per group.
---@param asteroidData table
---@param lodMesh LodMesh
---@return function renderFn
function AsteroidBeltRenderer.createRenderFn(asteroidData, lodMesh)
    local inst_shader = Cache.Shader('wvp_instanced_tex', 'material/asteroid_instanced')
    local asteroid_tex = Cache.Texture('rock')

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
    -- Linear filter (Nearest doesn't exist; texelFetch uses integer coords
    -- so filtering is irrelevant anyway)
    staticTex:setMinFilter(TexFilter.Linear)
    staticTex:setMagFilter(TexFilter.Linear)

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

    -- Flat per-asteroid SoA (hot loop reads these, never the Lua tables):
    -- position, scale, and a spawned flag array synced once per frame.
    local posX = ffi.new('float[?]', nAst + 1)
    local posY = ffi.new('float[?]', nAst + 1)
    local posZ = ffi.new('float[?]', nAst + 1)
    local scales = ffi.new('float[?]', nAst + 1)
    local spawnedFlags = ffi.new('uint8_t[?]', nAst + 1)
    for i = 1, nAst do
        local a = asteroidData[i]
        posX[i] = a.px
        posY[i] = a.py
        posZ[i] = a.pz
        scales[i] = a.scale
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
    -- Screen-size LOD thresholds (projected pixels). Bands are geometric
    -- (halving each level) to match the ~2.3x vertex reduction per LOD.
    -- LOD 0 = res 96 (16k verts) down to LOD 7 (34 verts). Sub-pixel
    -- asteroids (below the LOD-7 threshold) are culled entirely.
    local lodPxMin = ffi.new('float[8]', { 32, 16, 8, 4, 2, 1, 0.5, 0.25 })
    -- Squared thresholds: the hot loop compares px^2 = s^2 * pxPerUnit^2
    -- / distSq, avoiding a sqrt + division per asteroid (113K of them).
    local lodPxMinSq = ffi.new('float[8]')
    for i = 0, 7 do lodPxMinSq[i] = lodPxMin[i] * lodPxMin[i] end
    for i = 1, #AsteroidMeshPool.getLodRanges() do
        local r = AsteroidMeshPool.getLodRanges()[i]
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

        -- Reset per-frame group counts
        for i = 1, #groupOrder do
            groups[groupOrder[i]].count = 0
        end

        -- Sync spawned flags once per frame (small set, O(spawned) not
        -- O(n): read the field system's spawned indices into the flat
        -- byte array so the hot loop never touches the Lua asteroid tables)
        local spawnedIdx = AsteroidFieldSystem:getSpawnedIndices(entity)
        for s = 1, #spawnedIdx do
            spawnedFlags[spawnedIdx[s]] = 1
        end

        local drawn = 0
        local done = false

        -- Chunk pass: cull by centroid distance + view cone, then collect
        -- survivors. The cone test (KSA article: frustum culling before
        -- LOD) skips whole angular sectors behind or far beside the
        -- camera - the fly-through only sees ~180deg of the ring, so
        -- ~half the chunks are dropped before any per-asteroid work.
        -- All per-asteroid data is flat cdata (posX/Y/Z, scales,
        -- spawnedFlags) - no Lua table lookups in the loop.
        local fwdX, fwdY, fwdZ = 0, 0, -1
        local camForward = CameraManager:getForward()
        if camForward then fwdX, fwdY, fwdZ = camForward.x, camForward.y, camForward.z end
        -- Cone half-angle + margin (sectors are wide; the chunk's radius
        -- is culled by the distance test, this catches the plane split)
        local fwdLenSq = fwdX * fwdX + fwdY * fwdY + fwdZ * fwdZ
        for c = 1, CHUNK_COUNT do
            if done then break end
            local nIdx = chunkCounts[c]
            if nIdx == 0 then goto next_chunk end

            -- Centroid eye-distance (world coords = entity pos + centroid)
            local rcx = entPosX + chunkCx[c] - eyeX
            local rcy = entPosY + chunkCy[c] - eyeY
            local rcz = entPosZ + chunkCz[c] - eyeZ
            local cDistSq = rcx * rcx + rcy * rcy + rcz * rcz
            if cDistSq > renderDistSq then goto next_chunk end

            -- View-cone test: dot(centroidDir, forward) < -0.3 -> chunk is
            -- more than ~107deg behind the camera plane, cull it. (The
            -- ring band is ~2x belt radius from the eye on the fly path,
            -- so this is a conservative but safe cutoff.)
            if fwdLenSq > 1e-9 and cDistSq > 1e-9 then
                local invDist = 1.0 / math.sqrt(cDistSq)
                local dot = (rcx * fwdX + rcy * fwdY + rcz * fwdZ) * invDist / math.sqrt(fwdLenSq)
                if dot < -0.3 then goto next_chunk end
            end

            local base = chunkOffsets[c]
            for j = 0, nIdx - 1 do
                if drawn >= MAX_DRAWN_PER_FRAME then done = true break end
                local i = chunkIndices[base + j]
                if spawnedFlags[i] == 0 then
                    local rx = entPosX + posX[i] - eyeX
                    local ry = entPosY + posY[i] - eyeY
                    local rz = entPosZ + posZ[i] - eyeZ
                    local distSq = rx * rx + ry * ry + rz * rz

                    -- Screen-size LOD selection (GameUnit-agnostic):
                    -- projected pixel height ≈ scale / dist * pxPerUnit.
                    -- Compared via multiplication (pxSq >= t  <=>
                    -- s^2 * pxPerUnitSq >= distSq * t) so the hot loop
                    -- needs no sqrt and no division per asteroid.
                    -- LOD 0 (16k verts) only for big/near rocks; sub-pixel
                    -- rocks are culled entirely.
                    local s2ppu = scales[i] * scales[i] * pxPerUnitSq
                    local li = 1
                    while li < 8 and distSq * lodPxMinSq[li - 1] > s2ppu do
                        li = li + 1
                    end
                    if distSq * lodPxMinSq[li - 1] <= s2ppu then
                        local mesh = lodMeshes[li]
                        if mesh then
                            -- Texture-fetch instancing: the ONLY per-frame
                            -- per-asteroid write is a 4-byte u32 index into
                            -- the static data texture. The transform lives
                            -- on the GPU; the vertex shader texelFetches it
                            -- and composes wp = rotScale*v + (pos - eye).
                            local g = groups[li]
                            if not g then
                                g = { mesh = mesh, capacity = 64, count = 0, indices = ffi.new('uint32_t[64]') }
                                groups[li] = g
                                groupOrder[#groupOrder + 1] = li
                            end
                            if g.count >= g.capacity then
                                local newCap = g.capacity * 2
                                local newArr = ffi.new('uint32_t[?]', newCap)
                                ffi.copy(newArr, g.indices, g.count * ffi.sizeof('uint32_t'))
                                g.indices = newArr
                                g.capacity = newCap
                            end

                            g.indices[g.count] = i - 1  -- 0-based for the shader
                            g.count = g.count + 1
                            drawn = drawn + 1
                        end
                    end
                end
            end

            ::next_chunk::
        end

        -- Clear the spawned flags we set this frame (next frame re-syncs)
        for s = 1, #spawnedIdx do
            spawnedFlags[spawnedIdx[s]] = 0
        end

        -- Flush: one instanced draw per (mesh variant, LOD level), all
        -- instances pulled from the static data texture by index
        if #groupOrder > 0 then
            inst_shader:start()
            inst_shader:setTex2D('texDiffuse', asteroid_tex)
            inst_shader:setTex2D('instanceDataTex', staticTex)
            -- Camera-relative origin, subtracted in DOUBLE precision here
            -- (Lua numbers are f64): at AU-scale coordinates the origin
            -- and eye are ~1e7 GU, and float32 cannot represent
            -- (1e7 - 1e7 + 1000) - the ULP at 1e7 is ~1 GU, so the shader
            -- doing origin - eye itself would jitter every asteroid by up
            -- to a GU per frame (larger than a 1 GU rock). The shader
            -- receives ONE small-magnitude vec3 and adds the baked
            -- position on top - no cancellation.
            inst_shader:setFloat3('originRelEye', entPosX - eyeX, entPosY - eyeY, entPosZ - eyeZ)
            for i = 1, #groupOrder do
                local g = groups[groupOrder[i]]
                if g.count > 0 then
                    g.mesh:drawInstancedIndices(g.indices, g.count)
                end
            end
            inst_shader:stop()
        end
    end
end

return AsteroidBeltRenderer
