local Entity = require('Legacy.GameObjects.Entity')
local Material = require('Legacy.GameObjects.Material')
local AsteroidInstancedRenderer = require('Legacy.Systems.Overlay.AsteroidInstancedRenderer')

-- Route the per-entity render through the instanced collector instead of
-- the legacy per-draw path: asteroids share the 16-mesh pool, so they are
-- grouped by (mesh variant, LOD level) and drawn with ONE
-- DrawInstancedWithData per group (~600 draws -> ~16-128).
local function renderInstanced(self, state)
    if state.mode == BlendMode.Disabled then
        AsteroidInstancedRenderer.collect(self.body, self.mesh, state.eye, self:getScale())
    end
end

-- Pool of distinct asteroid meshes, shared by all asteroids. Each asteroid
-- picks one by seed hash, so the pool keeps visual diversity while meshes
-- are generated once and reused. Before this pool, getMesh did `seed % 1`
-- which collapsed EVERY asteroid onto a single mesh (integer seed % 1 == 0
-- always) - so all asteroids looked identical. N=16 keeps the field varied
-- while generating 16 meshes total instead of one per asteroid.
local POOL_SIZE = 16
local cache = {}

local function getMesh(seed)
    -- Bucket the seed into the pool; keep it deterministic per asteroid.
    local idx = tonumber(seed) % POOL_SIZE
    if idx < 0 then idx = idx + POOL_SIZE end
    if not cache[idx] then
        cache[idx] = Gen.Asteroid(idx)
    end
    return cache[idx]
end

local Asteroid = Subclass("Asteroid", Entity, function(self, seed, scale)
    local mesh = getMesh(seed)
    self:addRigidBody(true, mesh:get(0), Enums.ColliderType.ConvexHull)
    self:addVisibleLodMesh(mesh, Material.Rock())
    self:unregister(OldEvent.Render, Entity.renderVisibleLodMesh)
    self:register(OldEvent.Render, renderInstanced)
    self:addTrackable(true)
    self:addMinable(true)
    self:addClaimable()
    self.zone = nil

    -- NOTE: scale must be set before the radius will be reported correctly
    -- may be updated by the caller
    self:setScale(scale)

    -- TODO: Define asteroid mass as radius ^ asteroid type exponent
    local radius = self:getRadius()
    local mass = 100 + radius ^ Config.gen.massAsteroidExp[1]
    self:setMass(mass)

    self:setDrag(radius / 10, radius / 10) -- fix asteroid in place (unless really tiny)
end)

return Asteroid
