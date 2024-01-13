local Entity = require('GameObjects.Entity')
local SocketType = require('GameObjects.Entities.Ship.SocketType')
local Material = require('GameObjects.Material')

-- TODO : Constraints

local mesh
local material
local shader
local shared
local varCache
local rng = RNG.FromTime()

local Turret = subclass(Entity, function(self)
    if not shared then
        shared = {}
        -- shared.mesh = Gen.ShipBasic.TurretSingle(rng)
        shared.mesh = Gen.ShipFighter.TurretSingle(rng)
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
        mesh = Gen.Primitive.Billboard(-1, -1, 1, 1)
    end

    if not material then
        material = Material.Create(
            'material/metal',
            Cache.Texture('metal/01_d'),
            Cache.Texture('metal/01_n'),
            Cache.Texture('metal/01_s'))
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
    -- TODO: Extend to effects other than Pulse Turret
    self.name       = Config.gen.compTurretPulseStats.name
    self.healthCurr = Config.gen.compTurretPulseStats.healthCurr
    self.healthMax  = Config.gen.compTurretPulseStats.healthMax
    self.projSpread = Config.gen.compTurretPulseStats.spread
    self.projRange  = Config.gen.compTurretPulseStats.range
    self.projSpeed  = Config.gen.compTurretPulseStats.speed
    self.projLife   = self.projRange / self.projSpeed

    self.aim        = Quat.Identity()
    self.mesh       = shared.mesh

    self.firing     = 0
    self.heat       = 0
    self.cooldown   = 0

    --Log.Debug("Register: Turret name = %s, type = %s, handler = %s", self.name, Event.Update, self.updateTurret)
    self:register(Event.Update, self.updateTurret)
end)

function Turret:getSocketType()
    return SocketType.Turret
end

function Turret:addCooldown(cooldown)
    self.cooldown = self.cooldown + cooldown
end

function Turret:aimAt(pos)
    if not GameState.paused then
        local look = pos - self:getPos()
        local up   = self:getParent():getUp()
        self.aim:iLerp(Quat.FromLookUp(look, up), 0.1)
        self.aim = Quat.FromLookUp(look, up)
        -- TODO : Isn't this already normalized?
        self.aim:iNormalize()
    end
end

function Turret:aimAtTarget(target, fallback)
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

function Turret:canFire()
    return not GameState.paused and self.cooldown <= 0 and
        self:getParent():mgrCapacitorGetCharge() >= Config.gen.compTurretPulseStats.charge
end

function Turret:fire()
    if not self:canFire() then return end
    --Log.Debug("%s firing!", self:getParent():getName())

    self:getParent().projColorR = Config.gen.compTurretPulseStats.colorBodyR
    self:getParent().projColorG = Config.gen.compTurretPulseStats.colorBodyG
    self:getParent().projColorB = Config.gen.compTurretPulseStats.colorBodyB

    Config.game.pulseColorBodyR = Config.gen.compTurretPulseStats.colorBodyR
    Config.game.pulseColorBodyG = Config.gen.compTurretPulseStats.colorBodyG
    Config.game.pulseColorBodyB = Config.gen.compTurretPulseStats.colorBodyB

    local projectile = self:getRoot():addProjectile(self:getParent())
    local effect = projectile:getEffect()
    local dir = (self:getForward() + rng:getDir3():scale(self.projSpread * rng:getExp())):normalize()
    effect.pos = self:toWorld(Vec3f(0, 0, 0))
    effect.vel = dir:scale(self.projSpeed) + self:getParent():getVelocity()
    effect.dir = dir
    assert(effect.dir:length() >= 0.9)
    effect.lifeMax = self.projLife
    effect.life = effect.lifeMax

    -- Discharge capacitor if turret holds an energy weapon
    -- TODO: extend to different weapon types
    self:getParent():mgrCapacitorDischarge(Config.gen.compTurretPulseStats.charge)

    -- NOTE : In the future, it may be beneficial to store the actual turret
    -- rather than the parent. It would allow, for example, data-driven
    -- AI threat analysis by keeping track of which weapons have caused
    -- the most real damage to it, allowing for optimal sub-system
    -- targetting.
    --print(self:getParent().name)

    local rpmDeviation = Config.gen.compTurretPulseStats.weaponRPM - Config.gen.compTurretPulseStats.weaponRPM *
        rng:getUniformRange(Config.gen.compTurretPulseStats.weaponRPMDeviation, 0)
    self.cooldown = 60 / rpmDeviation -- 60 seconds / fire rate per minute
    self.heat = self.heat + 1

    -- Event to parent
    self:getParent():send(Event.FiredTurret(self, projectile, effect))
end

function Turret:render(state)
    if state.mode == BlendMode.Additive then
        shader:start()
        Shader.ISetFloat3(varCache.color, 1.0, 1.3, 2.0)
        mesh:drawBind()
        -- TODO : Should this check be done first?
        if self.heat > 1e-3 then
            Shader.ISetFloat(varCache.size, 8)
            Shader.ISetFloat(varCache.alpha, 2.0 * self.heat)
            Shader.ISetMatrix(varCache.mWorld, self:getToWorldMatrix())
            mesh:drawBound()
        end
        mesh:drawUnbind()
        shader:stop()
    end
end

function Turret:updateTurret(state)
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

return Turret
