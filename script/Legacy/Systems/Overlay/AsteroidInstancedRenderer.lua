-- AsteroidInstancedRenderer — batched instanced rendering for asteroid
-- fields. Asteroids share a 16-mesh pool (Asteroid.lua); this renderer
-- groups them by (mesh variant, LOD level) and emits ONE
-- DrawInstancedWithData per group instead of one draw per asteroid.
-- ~600 asteroid draws -> ~16-128 instanced draws.
--
-- The instanced shader (wvp_instanced + material/asteroid_instanced) reads
-- the model matrix from instance attributes 4-7, scale from attribute 9
-- (both set up by the executor's DrawInstancedWithData path), and derives
-- normals from mat3(model) (scale cancels in normalize for uniform-scale
-- rigid bodies). mWorld/mWorldIT/scale uniforms are NOT used.
local AsteroidInstancedRenderer = {}

-- Cached LuaJIT ctypes (declared in libphx.lua Basic Typedefs).
local ffi = require('ffi')

-- Per-frame scratch: reused across frames to avoid re-allocating tables.
-- Groups: key = lodMesh object, value = { mesh, instances = cdata array, count }
local groups = {}
local groupOrder = {}

--- Reset per-frame group state (called at the start of the Opaque pass).
function AsteroidInstancedRenderer.beginFrame()
    for i = 1, #groupOrder do
        local g = groups[groupOrder[i]]
        g.count = 0
    end
end

-- Reused out-params: RigidBody_GetPos writes into a passed Position
-- (no allocation per call); the world matrix is only built for asteroids
-- that pass the LOD check (~6 of ~800), instead of for every asteroid.
local scratchPos = Position()

--- Collect one asteroid for instanced rendering (called per asteroid
--- instead of the legacy renderVisibleLodMesh). `body` is the raw rigid
--- body (getPos(out), getToWorldMatrix(eye)), `lodMesh` the shared LodMesh.
---@param body any rigid body
---@param lodMesh any LodMesh (get(distSq) -> Mesh, draw)
---@param eye Vec3f camera position
---@param radius number|nil entity scale (fallback: 1)
function AsteroidInstancedRenderer.collect(body, lodMesh, eye, scale)
    -- LOD selection matches the legacy renderVisibleLodMesh:
    -- mesh:draw(distSq / scale^2) — normalize by scale before selecting.
    local s = scale or 1.0
    body:getPos(scratchPos)
    local distSq = eye:distanceSquared(scratchPos) / (s * s)
    local mesh = lodMesh:get(distSq) -- LOD-selected Mesh (or nil far away)
    if not mesh then return end

    local g = groups[mesh]
    if not g then
        g = { mesh = mesh, capacity = 64, count = 0, instances = ffi.new('InstanceData[64]') }
        groups[mesh] = g
        insert(groupOrder, mesh)
    end

    if g.count >= g.capacity then
        local newCap = g.capacity * 2
        local newArr = ffi.new('InstanceData[?]', newCap)
        ffi.copy(newArr, g.instances, g.count * ffi.sizeof('InstanceData'))
        g.instances = newArr
        g.capacity = newCap
    end

    -- Camera-relative world matrix: matches what the non-instanced path
    -- sends as mWorld (getToWorldMatrix(eye) is eye-relative; the view
    -- matrix is rotation-only). Built only for LOD-surviving asteroids.
    local inst = g.instances[g.count]
    local world = body:getToWorldMatrix(eye)
    ffi.copy(inst.model_matrix, world.m, ffi.sizeof('float') * 16)
    inst.scale = s
    g.count = g.count + 1
end

--- Flush all groups: one instanced draw per (mesh variant, LOD level).
---@param shader any instanced asteroid shader (bound by caller)
function AsteroidInstancedRenderer.flush()
    for i = 1, #groupOrder do
        local g = groups[groupOrder[i]]
        if g.count > 0 then
            g.mesh:drawInstancedWithData(g.instances, g.count)
        end
    end
end

return AsteroidInstancedRenderer
