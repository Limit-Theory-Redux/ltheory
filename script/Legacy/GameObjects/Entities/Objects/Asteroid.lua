local Entity = require('Legacy.GameObjects.Entity')
local Material = require('Legacy.GameObjects.Material')

local cache = {}

local function getMesh(seed)
    local seed = tonumber(seed) % 1
    if not cache[seed] then
        cache[seed] = Gen.Asteroid(seed)
    end
    return cache[seed]
end

local Asteroid = Subclass("Asteroid", Entity, function(self, seed, scale)
    local mesh = getMesh(seed)
    self:addRigidBody(true, mesh:get(0), Enums.ColliderType.Trimesh)
    self:addVisibleLodMesh(mesh, Material.Rock())
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
