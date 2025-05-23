local Entity = require('Legacy.GameObjects.Entity')
local SocketType = require('Legacy.GameObjects.Entities.Ship.SocketType')
local Material = require('Legacy.GameObjects.Material')

-- TODO : Constraints

local mesh
local material
local shader
local shared
local varCache
local rng = RNG.FromTime()

local Bay = Subclass("Bay", Entity, function(self)
    if not shared then
        shared = {}
        -- shared.mesh = Gen.ShipBasic.BaySingle(rng)
        shared.mesh = Gen.ShipFighter.BaySingle(rng)
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
        mesh = Gen.Primitive.Billboard(-1, -1, 1, 1)
    end

    if not material then
        material = Material.Create(
            'material/metal',
            Cache.Texture('metal/03_d'),
            Cache.Texture('metal/03_n'),
            Cache.Texture('metal/03_s'))
    end

    if not shader then
        shader = Cache.Shader('billboard/quad', 'effect/pulsehead')
    end

    if not varCache then
        varCache = ShaderVarCache(shader, { 'color', 'size', 'alpha', 'mWorld', })
    end

    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, material)

    -- TODO : Tracking Component
    -- TODO: Extend to effects other than Pulse Bay
    self.name       = Config.gen.compBayPulseStats.name
    self.healthCurr = Config.gen.compBayPulseStats.healthCurr
    self.healthMax  = Config.gen.compBayPulseStats.healthMax
    self.projSpread = Config.gen.compBayPulseStats.spread
    self.projRange  = Config.gen.compBayPulseStats.range
    self.projSpeed  = Config.gen.compBayPulseStats.speed
    self.projLife   = self.projRange / self.projSpeed

    self.aim        = Quat.Identity()
    self.mesh       = shared.mesh

    self.firing     = 0
    self.cooldown   = 0
    self.heat       = 0

    --Log.Debug("Register: Bay name = %s, type = %s, handler = %s", self.name, OldEvent.Update, self.updateBay)
    self:register(OldEvent.Update, self.updateBay)
end)

function Bay:getSocketType()
    return SocketType.Bay
end

function Bay:addCooldown(cooldown)
    self.cooldown = self.cooldown + cooldown
end

function Bay:aimAt(pos)
    if not GameState.paused then
        local look = pos:relativeTo(self:getPos())
        local up   = self:getParent():getUp()
        self.aim:iLerp(Quat.FromLookUp(look, up), 0.1)
        self.aim = Quat.FromLookUp(look, up)
        -- TODO : Isn't this already normalized?
        self.aim:iNormalize()
    end
end

function Bay:aimAtTarget(target, fallback)
    local tHit, pHit = Math.Impact(
        self:getPos(),
        target:getPos(),
        self:getParent():getVelocity(),
        target:getVelocity(),
        self.projSpeed)

    if tHit and tHit < self.projLife then
        self:aimAt(pHit)
        return true
    elseif fallback then
        self:aimAt(fallback)
    end

    return false
end

function Bay:canFire()
    return not GameState.paused and self.cooldown <= 0 and
        self:getParent():mgrCapacitorGetCharge() >= Config.gen.compBayPulseStats.charge
end

function Bay:fire()
    if not self:canFire() then return end
    --Log.Debug("%s firing!", self:getParent():getName())

    self:getParent().projColorR = Config.gen.compBayPulseStats.colorBodyR
    self:getParent().projColorG = Config.gen.compBayPulseStats.colorBodyG
    self:getParent().projColorB = Config.gen.compBayPulseStats.colorBodyB

    Config.game.pulseColorBodyR = Config.gen.compBayPulseStats.colorBodyR
    Config.game.pulseColorBodyG = Config.gen.compBayPulseStats.colorBodyG
    Config.game.pulseColorBodyB = Config.gen.compBayPulseStats.colorBodyB

    local projectile = self:getRoot():addProjectile(self:getParent())
    local effect = projectile:getEffect()
    local dir = (self:getForward() + rng:getDir3():scale(self.projSpread * rng:getExp())):normalize()
    effect.pos = self:toWorld(Vec3f(0, 0, 0))
    effect.vel = dir:scale(self.projSpeed) + self:getParent():getVelocity()
    effect.dir = dir
    assert(effect.dir:length() >= 0.9)
    effect.lifeMax = self.projLife
    effect.life = effect.lifeMax

    -- Discharge capacitor if bay holds an energy weapon
    self:getParent():mgrCapacitorDischarge(Config.gen.compBayPulseStats.charge)

    -- NOTE : In the future, it may be beneficial to store the actual bay
    -- rather than the parent. It would allow, for example, data-driven
    -- AI threat analysis by keeping track of which weapons have caused
    -- the most real damage to it, allowing for optimal sub-system
    -- targetting.
    local rpmDeviation = Config.gen.compBayPulseStats.weaponRPM - Config.gen.compBayPulseStats.weaponRPM *
        rng:getUniformRange(Config.gen.compTurretPulseStats.weaponRPMDeviation, 0)
    self.cooldown = 60 / rpmDeviation -- 60 seconds / fire rate per minute
    self.heat = self.heat + 1
end

function Bay:render(state)
    if state.mode == BlendMode.Additive then
        shader:start()
        shader:iSetFloat3(varCache.color, 1.0, 1.3, 2.0)
        mesh:drawBind()
        -- TODO : Should this check be done first?
        if self.heat > 1e-3 then
            shader:iSetFloat(varCache.size, 8)
            shader:iSetFloat(varCache.alpha, 2.0 * self.heat)
            shader:iSetMatrix(varCache.mWorld, self:getToWorldMatrix(state.eye))
            mesh:drawBound()
        end
        mesh:drawUnbind()
        shader:stop()
    end
end

function Bay:updateBay(state)
    --Log.Debug("name = %s", self.name)
    local decay = exp(-16.0 * state.dt)
    self:setRotLocal(self:getParent():getRot():inverse() * self.aim)
    if self.firing > 0 then
        self.firing = 0
        if self.cooldown <= 0 then self:fire() end
    end
    self.cooldown = max(0, self.cooldown - state.dt)
    self.heat = self.heat * decay
end

return Bay
