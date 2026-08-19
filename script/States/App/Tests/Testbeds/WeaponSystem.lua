Namespace.LoadInline("Legacy")
Namespace.LoadInline("Legacy.Systems")

local Application = require("States.Application")

local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local PhysicsComponents = require("Modules.Physics.Components")
local RenderingComponents = require("Modules.Rendering.Components")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ConstructEntities = require("Modules.Constructs.Entities")
local ShipGenerator = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local TurretLoadoutGenerator = require("Modules.Constructs.Managers.Generators.TurretLoadoutGenerator")
local CameraEntity = require("Modules.Cameras.Entities").Camera
local SkyboxEntity = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local CameraDataComponent = require("Modules.Cameras.Components.CameraDataComponent")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local CameraSystem = require("Modules.Cameras.Systems.CameraSystem")
local Generator = require("Legacy.Systems.Gen.Generator")
local Starfield = require("Legacy.Systems.Gen.Starfield")
local Pulse = require("Legacy.GameObjects.Entities.Effects.Pulse")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local AIWeaponSystem = require("Modules.Constructs.Systems.AIWeaponSystem")
local ProjectileSystem = require("Modules.Constructs.Systems.ProjectileSystem")
local ShipWeaponRegistry = require("Shared.Registries.ShipWeaponRegistry")
local WeaponActions = require("Input.ActionBindings.WeaponTestbedActions")
local DrawEx = require("UI.DrawEx")

---@class WeaponSystemTestbed: Application
local WeaponSystemTestbed = Subclass("WeaponSystemTestbed", Application)

local MOUNT_IDS = {
    "fore_port",
    "fore_starboard",
    "mid_port",
    "mid_starboard",
    "aft_port",
    "aft_starboard",
}

local function removeRigidBody(world, entity)
    if not entity or not entity:isValid() then
        return
    end
    local rbComponent = entity:get(PhysicsComponents.RigidBody)
    if rbComponent and rbComponent:getRigidBody() then
        world:removeRigidBody(rbComponent:getRigidBody())
    end
end

local function copyMountPosition(capitalPosition, x, y, z, capitalScale)
    return {
        position = Position(
            capitalPosition.x + x,
            capitalPosition.y + y,
            capitalPosition.z + z),
        localPosition = Position(
            x / capitalScale,
            y / capitalScale,
            z / capitalScale),
    }
end

function WeaponSystemTestbed:spawnTarget()
    if self.target and self.target:isValid() then
        return
    end

    self.targetGeneration = (self.targetGeneration or 0) + 1
    self.target = ShipGenerator:createCapital(self.seed + self.targetGeneration, {
        position = self.targetPosition,
        scale = self.targetScale,
        isKinematic = true,
    }, {
        maxHealth = self.targetMaxHealthConfig,
        engine = {},
        defense = {
            maxHealth = self.targetMaxHealthConfig,
            maxShield = 0,
            armor = 0,
            shieldRegen = 0,
        },
    })
    self.target:add(ConstructComponents.Targetable("debug", self.targetSizeClass))
    self.targetBody = self.target:get(PhysicsComponents.RigidBody):getRigidBody()
    self.targetBody:setCollidable(true)
    self.targetHealth = self.target:get(CoreComponents.Health)
    self.targetRadius = self.target:get(PhysicsComponents.RigidBody):getRadius()
    self.targetMaxHealth = self.targetHealth:getMaxHealth()
    self.world:addRigidBody(self.targetBody)
    self.targetCandidates = {
        {
            id = self.target.id,
            entity = self.target,
            body = self.targetBody,
            position = self.targetBody:getPos(),
            enabled = true,
            sizeClass = self.targetSizeClass,
        },
    }
    self.targetRespawnRemaining = nil
    if self.targeting then
        self.targeting:setTarget(nil)
    end
end

function WeaponSystemTestbed:destroyTarget()
    if not self.target then
        return
    end

    local target = self.target
    removeRigidBody(self.world, target)
    if target:isValid() then
        Registry:destroyEntity(target, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end

    self.target = nil
    self.targetBody = nil
    self.weaponTargetEntity = nil
    self.weaponTargetBody = nil
    self.targetHealth = nil
    self.targetRadius = nil
    self.targetCandidates = {}
    self.targetRespawnRemaining = self.targetRespawnDelay
    if self.targeting then
        self.targeting:setTarget(nil)
    end
end

function WeaponSystemTestbed:onTargetDestroyed()
    self:destroyTarget()
end

function WeaponSystemTestbed:updateTargetLifecycle(dt)
    if self.target and self.targetBody then
        if self.targetCandidates[1] then
            self.targetCandidates[1].position = self.targetBody:getPos()
        end
        return
    end

    if self.target then
        self:destroyTarget()
    end

    self.targetRespawnRemaining = math.max(0, (self.targetRespawnRemaining or 0) - dt)
    if self.targetRespawnRemaining <= 0 then
        self:spawnTarget()
    end
end

function WeaponSystemTestbed:onInit()
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = 20260819
    self.world = Physics.Create()
    self.projectiles = {}
    self.turrets = {}
    self.turretsById = {}
    self.mountIds = MOUNT_IDS
    self.lastShotOrder = {}
    self.lastImpact = nil
    self.weaponKey = "debugPulseTurret"
    self.weapon = ShipWeaponRegistry:get(self.weaponKey)
    assert(self.weapon, "missing testbed ship weapon: " .. self.weaponKey)
    self.testbedConfig = Config.weapons.testbed or {}
    self.targetRespawnDelay = self.testbedConfig.targetRespawnDelay or 3.0
    self.targetMaxHealthConfig = self.testbedConfig.targetMaxHealth or 300
    self.targetSizeClass = self.testbedConfig.targetSizeClass or "small"
    self.targetGeneration = 0
    self.targetRespawnRemaining = 0

    self.skybox = SkyboxEntity(self.seed, function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty)
        if not placeholder then
            placeholder = entity:add(CoreComponents.Empty)
        end

        if not placeholder.envMap then
            require("Legacy.Systems.Gen.Nebula.Nebula1")
            local nebulaRNG = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle = nebulaRNG:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            placeholder.envMap = Generator.Get("Nebula", nebulaRNG)(
                nebulaRNG,
                Config.gen.nebulaRes,
                placeholder.starDir)
            placeholder.irMap = placeholder.envMap:genIRMap(256)
            placeholder.stars = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
            CameraManager:setStarDir(placeholder.starDir)
            ShaderVar.PushTexCube("envMap", placeholder.envMap)
            ShaderVar.PushTexCube("irMap", placeholder.irMap)
        end

        if blendMode == BlendMode.Disabled then
            RenderState.PushDepthWritable(false)
            local shader = Cache.Shader("farplane", "skybox")
            RenderState.PushCullFace(CullFace.None)
            shader:start()
            Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
            shader:stop()
            RenderState.PopCullFace()
            RenderState.PopDepthWritable()
        elseif blendMode == BlendMode.Additive then
            local shader = Cache.Shader("farplane", "starbg")
            shader:start()
            shader:setFloat("brightnessScale", 3)
            shader:setTexCube("irMap", placeholder.irMap)
            shader:setTexCube("envMap", placeholder.envMap)
            placeholder.stars:draw()
            shader:stop()
        end
    end)

    self.pulseRenderEntity = Entity.Create(
        "WeaponPulseEffects",
        RenderingComponents.Render(function(_, blendMode)
            if #self.projectiles > 0 then
                Pulse.Render(self.projectiles, {
                    mode = blendMode,
                    eye = CameraManager:getEye(),
                })
            end
        end))

    -- CameraSystem owns controller input/pre-render updates through its event subscriptions.
    local _ = CameraSystem
    local camera = CameraEntity()
    CameraManager:registerCamera("WeaponOrbit", camera)
    self.cameraController = OrbitCameraController(camera, {
        distance = 6.0,
        minDistance = 0.3,
        maxDistance = 14.0,
        initialYaw = 0.0,
        initialPitch = 0.22,
        zoomSpeed = 0.35,
    })
    camera:get(CameraDataComponent):setController(self.cameraController)
    CameraManager:setActiveCamera("WeaponOrbit")

    local capitalScale = Config.game.shipHulls.scale[6] * 4.0
    self.capitalScale = capitalScale
    self.capital = ShipGenerator:createCapital(self.seed, {
        position = Position(0, 0, 0),
        scale = capitalScale,
        isKinematic = true,
    })
    self.capitalBody = self.capital:get(PhysicsComponents.RigidBody):getRigidBody()
    self.capitalBody:setCollidable(true)
    self.world:addRigidBody(self.capitalBody)
    self.capitalBodyInWorld = true

    self.control = self.capital:add(ConstructComponents.WeaponControl(
        self.weapon.ai.defaultMode,
        MOUNT_IDS))
    self.control:setActive(self.testbedConfig.aiActive == true)
    self.targeting = self.capital:add(ConstructComponents.Targeting(self.weapon.range))
    self.targeting:setAutoAcquireEnabled(self.control:isActive())

    local capitalRadius = self.capital:get(PhysicsComponents.RigidBody):getRadius()
    capitalRadius = math.max(capitalRadius, 0.02)
    -- Starboard is +X in the testbed convention. Keep Z exactly zero so the
    -- target is a side-on LOS case rather than another diagonal presentation.
    local targetDistance = math.max(
        capitalRadius + 0.5,
        math.min(capitalRadius * 4.0, self.weapon.range * 0.80))
    self.targetPosition = Position(targetDistance, 0, 0)
    self.targetScale = capitalScale * (self.testbedConfig.targetScaleMultiplier or 1.0)
    self:spawnTarget()
    self.capitalRadius = capitalRadius
    self.targetDistance = targetDistance
    self.losObstacles = {
        {
            body = self.capitalBody,
            kind = "hull",
            radius = capitalRadius,
            position = self.capitalBody:getPos(),
        },
    }

    local mounts = {}
    local function addMount(id, x, y, z)
        local position = copyMountPosition(Position(0, 0, 0), x, y, z, capitalScale)
        position.mountId = id
        table.insert(mounts, position)
    end

    addMount("fore_port", -capitalRadius * 0.45, capitalRadius * 0.18, capitalRadius * 0.38)
    addMount("fore_starboard", capitalRadius * 0.45, capitalRadius * 0.18, capitalRadius * 0.38)
    addMount("mid_port", -capitalRadius * 0.55, 0, 0)
    addMount("mid_starboard", capitalRadius * 0.55, 0, 0)
    addMount("aft_port", -capitalRadius * 0.45, -capitalRadius * 0.18, -capitalRadius * 0.38)
    addMount("aft_starboard", capitalRadius * 0.45, -capitalRadius * 0.18, -capitalRadius * 0.38)

    for _, mount in ipairs(mounts) do
        mount.position = Position(
            self.capitalBody:getPos().x + mount.position.x,
            self.capitalBody:getPos().y + mount.position.y,
            self.capitalBody:getPos().z + mount.position.z)
    end

    local generatedTurrets = TurretLoadoutGenerator:create(self.capital, mounts, self.weapon)
    for _, mount in ipairs(generatedTurrets) do
        local component = mount.entity:get(ConstructComponents.Turret)
        local body = mount.entity:get(PhysicsComponents.RigidBody):getRigidBody()
        -- Turret bodies are aim sources, not obstacles in this testbed. Native
        -- ray queries do not honor setCollidable(false) as an exclusion filter.
        local record = {
            mountId = mount.mountId,
            entity = mount.entity,
            component = component,
            body = body,
        }
        table.insert(self.turrets, record)
        self.turretsById[mount.mountId] = record
    end

    self.cameraController:setTarget(self.capital)
    Log.Info(string.format(
        "WeaponSystem testbed: capital radius %.4f, target distance %.4f, six turrets ready",
        capitalRadius,
        targetDistance))
end

---@param mount table
---@param solution table
---@param weapon table
---@param shotSerial integer
function WeaponSystemTestbed:spawnProjectile(mount, solution, weapon, shotSerial)
    local sourcePosition = mount.body:getPos()
    local sourceVelocity = mount.body:getVelocity()
    local dx = solution.position.x - sourcePosition.x
    local dy = solution.position.y - sourcePosition.y
    local dz = solution.position.z - sourcePosition.z
    local length = math.sqrt(dx * dx + dy * dy + dz * dz)
    if length < 0.000001 then
        return
    end

    local direction = Vec3f(dx / length, dy / length, dz / length)
    local scale = weapon.projectileSpeed / length
    local velocity = Vec3f(
        dx * scale + sourceVelocity.x,
        dy * scale + sourceVelocity.y,
        dz * scale + sourceVelocity.z)
    local pulse = Pulse:new()
    pulse.pos = sourcePosition
    pulse.vel = velocity
    pulse.dir = velocity:normalize()
    pulse.lifeMax = weapon.projectileLifetime
    pulse.life = pulse.lifeMax
    pulse.dist = 0

    local entity = ConstructEntities.Projectile(shotSerial, {}, {
        source = mount.entity,
        position = sourcePosition,
        velocity = velocity,
        damage = weapon.damage,
        lifetime = weapon.projectileLifetime,
        scale = weapon.projectileScale,
        bodyMesh = nil,
    })
    local body = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    local pulseStats = Config.gen.compTurretPulseStats
    table.insert(self.projectiles, {
        entity = entity,
        body = body,
        component = entity:get(ConstructComponents.Projectile),
        mountId = mount.mountId,
        shotSerial = shotSerial,
        effect = pulse,
        pulseDistance = 0,
        pColorR = pulseStats.colorBodyR,
        pColorG = pulseStats.colorBodyG,
        pColorB = pulseStats.colorBodyB,
        pulseHeadSize = weapon.pulseHeadSize,
        pulseTailWidth = weapon.pulseTailWidth,
        pulseTailLength = weapon.pulseTailLength,
        inWorld = false,
    })
end

function WeaponSystemTestbed:removeProjectile(index)
    local projectile = table.remove(self.projectiles, index)
    if not projectile then
        return
    end
    if projectile.inWorld then
        removeRigidBody(self.world, projectile.entity)
    end
    Registry:destroyEntity(projectile.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
end

---@param data EventData
function WeaponSystemTestbed:onPreSim(data)
    local dt = data:deltaTime()
    WeaponActions.Fire:update(dt)
    WeaponActions.Volley:update(dt)
    WeaponActions.Sequence:update(dt)
    WeaponActions.AI:update(dt)
    WeaponActions.Reset:update(dt)

    if WeaponActions.AI:isPressed() then
        local active = not self.control:isActive()
        self.control:setActive(active)
        self.targeting:setAutoAcquireEnabled(active)
        if active then
            self.control:setSequenceIndex(1)
        end
    end

    if not self.control:isActive() and WeaponActions.Volley:isPressed() then
        self.control:setMode("volley")
        self.control:setSequenceIndex(1)
    elseif not self.control:isActive() and WeaponActions.Sequence:isPressed() then
        self.control:setMode("sequence")
        self.control:setSequenceIndex(1)
    end

    if WeaponActions.Reset:isPressed() then
        if self.target then
            self.targetHealth:setCurrentHealth(self.targetMaxHealth)
        else
            self.targetRespawnRemaining = 0
        end
        self.control:setSequenceIndex(1)
        self.lastImpact = nil
    end

    if not self.control:isActive() and WeaponActions.Fire:isReleased() then
        self.control:setSequenceIndex(1)
    end
    if not self.control:isActive() then
        self.control:setTriggerHeld(WeaponActions.Fire:isDown())
    else
        self.control:setTriggerHeld(false)
    end
end

---@param data EventData
function WeaponSystemTestbed:onSim(data)
    local dt = data:deltaTime()
    self:updateTargetLifecycle(dt)
    self.world:update(dt)
    AIWeaponSystem:update(self, dt)
    WeaponSystem:update(self, dt)
end

---@param data EventData
function WeaponSystemTestbed:onPostSim(data)
    ProjectileSystem:update(self, data:deltaTime())
end

---@param data EventData
function WeaponSystemTestbed:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local mode = self.control:getMode()
        local health = self.targetHealth and self.targetHealth:getCurrentHealth() or 0
        local targetState = self.target
            and string.format("%.0f/%.0f", health, self.targetMaxHealth)
            or string.format("respawn %.1fs", self.targetRespawnRemaining or 0)
        local lastShot = #self.lastShotOrder > 0 and table.concat(self.lastShotOrder, ", ") or "-"
        local impact = self.lastImpact and string.format(
            "hit %s at (%.3f, %.3f, %.3f)",
            self.lastImpact.mountId or "?",
            self.lastImpact.position.x,
            self.lastImpact.position.y,
            self.lastImpact.position.z) or "-"
        local lines = {
            "WeaponSystem Testbed",
            "LMB: fire   1: volley   2: sequence   A: toggle AI   R: reset target",
            string.format(
                "Mode: %s   AI: %s   Target HP: %s   Projectiles: %d",
                mode,
                self.control:isActive() and "active" or "off",
                targetState,
                #self.projectiles),
            string.format(
                "Capital radius: %.2f   Target starboard: %.2f   Sight: %d/6   Ready: %d/6",
                self.capitalRadius or 0,
                self.targetDistance or 0,
                self.sightCount or 0,
                self.readyCount or 0),
            string.format(
                "LOS ray results: target %d  hull %d  none %d  other %d  source %d",
                self.sightReasons and self.sightReasons.target or 0,
                self.sightReasons and self.sightReasons.hull or 0,
                self.sightReasons and self.sightReasons.none or 0,
                self.sightReasons and self.sightReasons.other or 0,
                self.sightReasons and self.sightReasons.source or 0),
            string.format(
                "Ray hit position: %s",
                self.sightHitPosition
                    and string.format(
                        "%.2f, %.2f, %.2f",
                        self.sightHitPosition.x,
                        self.sightHitPosition.y,
                        self.sightHitPosition.z)
                    or "-"),
            "Last fire order: " .. lastShot,
            "Last impact: " .. impact,
        }
        local y = 32
        for _, line in ipairs(lines) do
            DrawEx.TextAdditive("Unageo-Medium", line, 11, 32, y, 40, 20, 0.9, 0.9, 0.9, 0.95, 0.0, 0.5)
            y = y + 22
        end
    end)
end

function WeaponSystemTestbed:onExit()
    for index = #self.projectiles, 1, -1 do
        self:removeProjectile(index)
    end
    for _, turret in ipairs(self.turrets) do
        removeRigidBody(self.world, turret.entity)
    end
    if self.capitalBodyInWorld then
        removeRigidBody(self.world, self.capital)
    end

    if self.target and self.target:isValid() then
        removeRigidBody(self.world, self.target)
        Registry:destroyEntity(self.target, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    if self.capital and self.capital:isValid() then
        Registry:destroyEntity(self.capital, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    if self.pulseRenderEntity and self.pulseRenderEntity:isValid() then
        Registry:destroyEntity(self.pulseRenderEntity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    if self.skybox and self.skybox:isValid() then
        Registry:destroyEntity(self.skybox, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    CameraManager:unregisterCamera("WeaponOrbit")
end

return WeaponSystemTestbed
