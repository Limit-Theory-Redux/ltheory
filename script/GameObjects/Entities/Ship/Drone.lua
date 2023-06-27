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

local Drone = subclass(Entity, function(self)
    if not shared then
        shared = {}
        --    shared.mesh = Gen.ShipBasic.DroneSingle(rng)
        shared.mesh = Gen.ShipFighter.DroneSingle(rng)
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

    self.name         = Config.gen.compDroneStats.name
    self.healthCurr   = Config.gen.compDroneStats.healthCurr
    self.healthMax    = Config.gen.compDroneStats.healthMax
    self.droneType    = Config.gen.compDroneStats.droneType -- each drone rack launches a specific type of drone
    self.dronesCurr   = Config.gen.compDroneStats.dronesCurr
    self.dronesActive = Config.gen.compDroneStats.dronesActive
    self.dronesMax    = Config.gen.compDroneStats.dronesMax
    self.reloadTime   = Config.gen.compDroneStats.reloadTime
    self.projRange    = Config.gen.compDroneStats.droneRange
    self.projSpeed    = Config.gen.compDroneStats.droneSpeed
    self.projLife     = self.projRange / self.projSpeed -- TEMP: remove when actual drone functionality is added

    self.aim          = Quat.Identity()
    self.mesh         = shared.mesh

    self.firing       = 0
    self.cooldown     = 0
    self.heat         = 0

    --printf("Register: Drone name = %s, type = %s, handler = %s", self.name, Event.Update, self.updateDrone)
    self:register(Event.Update, self.updateDrone)
end)

function Drone:getSocketType()
    return SocketType.Drone
end

function Drone:addCooldown(cooldown)
    self.cooldown = self.cooldown + cooldown
end

function Drone:aimAt(pos)
    if not GameState.paused then
        local look = pos - self:getPos()
        local up   = self:getParent():getUp()
        self.aim:iLerp(Quat.FromLookUp(look, up), 0.1)
        self.aim = Quat.FromLookUp(look, up)
        -- TODO : Isn't this already normalized?
        self.aim:iNormalize()
    end
end

function Drone:aimAtTarget(target, fallback)
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

function Drone:canFire()
    return not GameState.paused and self.cooldown <= 0
end

function Drone:fire()
    if not self:canFire() then return end
    printf("%s launching drone!", self:getParent():getName())

    local projectile, effect = self:getRoot():addProjectile(self:getParent())
    local dir = (self:getForward() + rng:getDir3():scale(self.projSpread * rng:getExp())):normalize()
    effect.pos = self:toWorld(Vec3f(0, 0, 0))
    effect.vel = dir:scale(self.projSpeed) + self:getParent():getVelocity()
    effect.dir = dir
    assert(effect.dir:length() >= 0.9)
    effect.lifeMax = self.projLife
    effect.life = effect.lifeMax

    if projectile then
        projectile.pos  = effect.pos
        projectile.vel  = effect.vel
        projectile.dir  = effect.dir
        projectile.dist = 0
        --printf("DRONE: %s pos %s", projectile:getName(), projectile.pos)
    end

    -- NOTE : In the future, it may be beneficial to store the actual drone
    --        rather than the parent. It would allow, for example, data-driven
    --        AI threat analysis by keeping track of which weapons have caused
    --        the most real damage to it, allowing for optimal sub-system
    --        targetting.
    self.cooldown = 1.0
    self.heat = self.heat + 1
end

function Drone:render(state)
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

function Drone:updateDrone(state)
    --printf("name = %s", self.name)
    local decay = exp(-16.0 * state.dt)
    self:setRotLocal(self:getParent():getRot():inverse() * self.aim)
    if self.firing > 0 then
        self.firing = 0
        if self.cooldown <= 0 then self:fire() end
    end
    self.cooldown = max(0, self.cooldown - state.dt * Config.gen.compDroneStats.rateOfFire)
    self.heat = self.heat * decay
end

return Drone
