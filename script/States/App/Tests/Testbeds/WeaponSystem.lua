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
local HullMountDiscovery = require("Modules.Constructs.Managers.Generators.HullMountDiscovery")
local TurretLoadoutGenerator = require("Modules.Constructs.Managers.Generators.TurretLoadoutGenerator")
local CameraEntity = require("Modules.Cameras.Entities").Camera
local SkyboxEntity = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local CameraDataComponent = require("Modules.Cameras.Components.CameraDataComponent")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local LightManager = require("Modules.Rendering.Managers.LightManager")
local CameraSystem = require("Modules.Cameras.Systems.CameraSystem")
local Generator = require("Legacy.Systems.Gen.Generator")
local Starfield = require("Legacy.Systems.Gen.Starfield")
local Pulse = require("Legacy.GameObjects.Entities.Effects.Pulse")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local AIWeaponSystem = require("Modules.Constructs.Systems.AIWeaponSystem")
local ProjectileSystem = require("Modules.Constructs.Systems.ProjectileSystem")
local BeamSystem = require("Modules.Constructs.Systems.BeamSystem")
local WeaponTrackingSystem = require("Modules.Constructs.Systems.WeaponTrackingSystem")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local TrackingModuleGenerator = require("Shared.Content.TrackingModuleGenerator")
local WeaponGenerator = require("Shared.Content.WeaponGenerator")
local ProceduralCatalog = require("Shared.Content.ProceduralCatalog")
local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")
local WeaponActions = require("Input.ActionBindings.WeaponTestbedActions")
local DrawEx = require("UI.DrawEx")

---@class WeaponSystemTestbed: Application
local WeaponSystemTestbed = Subclass("WeaponSystemTestbed", Application)

local MOUNT_SPECS = {
    { mountId = "fore_outer_port", pairId = "fore_outer", zone = "fore", side = "port" },
    { mountId = "fore_outer_starboard", pairId = "fore_outer", zone = "fore", side = "starboard" },
    { mountId = "fore_inner_port", pairId = "fore_inner", zone = "fore", side = "port" },
    { mountId = "fore_inner_starboard", pairId = "fore_inner", zone = "fore", side = "starboard" },
    { mountId = "mid_port", pairId = "mid", zone = "mid", side = "port" },
    { mountId = "mid_starboard", pairId = "mid", zone = "mid", side = "starboard" },
    { mountId = "aft_inner_port", pairId = "aft_inner", zone = "aft", side = "port" },
    { mountId = "aft_inner_starboard", pairId = "aft_inner", zone = "aft", side = "starboard" },
    { mountId = "aft_outer_port", pairId = "aft_outer", zone = "aft", side = "port" },
    { mountId = "aft_outer_starboard", pairId = "aft_outer", zone = "aft", side = "starboard" },
}

local MOUNT_IDS = {}
for _, spec in ipairs(MOUNT_SPECS) do
    table.insert(MOUNT_IDS, spec.mountId)
end

local function removeRigidBody(world, entity)
    if not entity or not entity:isValid() then
        return
    end
    local rbComponent = entity:get(PhysicsComponents.RigidBody)
    if rbComponent and rbComponent:getRigidBody() then
        world:removeRigidBody(rbComponent:getRigidBody())
    end
end

function WeaponSystemTestbed:getActiveTargetMotion()
    local configured = self.targetMotion or {}
    local modes = configured.orbitModes
    local selected = modes and modes[self.targetMotionModeIndex or 1] or nil
    local motion = {}
    for key, value in pairs(configured) do
        if key ~= "orbitModes" then
            motion[key] = value
        end
    end
    for key, value in pairs(selected or {}) do
        motion[key] = value
    end
    motion.phase = (motion.phase or 0) + (self.targetMotionPhaseOffset or 0)
    return motion
end

function WeaponSystemTestbed:getTargetMotionLabel()
    local motion = self:getActiveTargetMotion()
    if motion.mode == "orbit" then
        return string.format(
            "orbit %s plane=%s dir=%+d phase=%.2f",
            motion.name or "unnamed",
            motion.plane or "xz",
            motion.direction or 1,
            motion.phase or 0)
    end
    return string.format(
        "%s axis=%s",
        motion.mode or "linear",
        motion.axis or "y")
end

function WeaponSystemTestbed:resetTargetMotionPosition()
    if not self.targetBody then
        return
    end
    local sample = WeaponSystem:sampleTargetMotion(
        self.targetMotionBasePosition,
        self.targetMotionTime or 0,
        self:getActiveTargetMotion())
    self.targetBody:setPos(Position(
        sample.position.x,
        sample.position.y,
        sample.position.z))
    self.targetMotionVelocity = sample.velocity
    self.targetVelocity = sample.velocity
    if self.targetCandidates and self.targetCandidates[1] then
        self.targetCandidates[1].position = sample.position
        self.targetCandidates[1].velocity = sample.velocity
    end
end

function WeaponSystemTestbed:setTargetOrbitMode(index)
    local modes = self.targetMotion and self.targetMotion.orbitModes or {}
    if #modes == 0 then
        return
    end
    self.targetMotionModeIndex = ((index - 1) % #modes) + 1
    self.targetMotionPhaseOffset = 0
    self.targetMotionTime = 0
    self:resetTargetMotionPosition()
    Log.Info("WeaponSystem target orbit: " .. self:getTargetMotionLabel())
end

function WeaponSystemTestbed:advanceTargetOrbitPhase()
    if not self.targetMotion or not self.targetMotion.orbitModes then
        return
    end
    self.targetMotionPhaseOffset = (self.targetMotionPhaseOffset or 0) + math.pi / 2
    self.targetMotionTime = 0
    self:resetTargetMotionPosition()
    Log.Info("WeaponSystem target orbit phase: " .. self:getTargetMotionLabel())
end

function WeaponSystemTestbed:spawnTarget()
    if self.target and self.target:isValid() then
        return
    end

    self.targetGeneration = (self.targetGeneration or 0) + 1
    self.targetMotionTime = 0
    local motionBasePosition = self.targetOrbitAnchorPosition or self.targetPosition
    self.targetMotionBasePosition = Position(
        motionBasePosition.x,
        motionBasePosition.y,
        motionBasePosition.z)
    self.targetMotionVelocity = Vec3f()
    self.targetVelocity = Vec3f()
    assert(self.targetSeedRng, "testbed target generation requires a deterministic seed stream")
    local targetSeed = self.targetSeedRng:get64()
    local targetPointRng = RNG.Create(targetSeed)
    assert(targetPointRng, "testbed target-point generation could not create an RNG")
    self.target = ShipGenerator:createCapital(targetSeed, {
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
    local targetShipData = self.target:get(ConstructComponents.ShipData)
    assert(targetShipData and targetShipData:getGeneratedMesh(),
        "target ship should retain generated geometry for target-point sampling")
    self.targetSurface = WeaponSystem:buildTargetSurface(targetShipData:getGeneratedMesh())
    self.targetPointSeed = targetPointRng:getInt(0, 2147483646)
    Log.Info(string.format(
        "WeaponSystem target surface: %d triangles, point seed %d",
        #self.targetSurface,
        self.targetPointSeed))
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
            velocity = Vec3f(),
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

    self:clearBeams()
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
    self.targetSurface = nil
    self.targetPointSeed = nil
    self.targetPointLogTime = nil
    self.targetMotionBasePosition = nil
    self.targetMotionVelocity = nil
    self.targetVelocity = nil
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
        self.targetMotionTime = (self.targetMotionTime or 0) + dt
        local motion = self:getActiveTargetMotion()
        local sample = WeaponSystem:sampleTargetMotion(
            self.targetMotionBasePosition,
            self.targetMotionTime,
            motion)
        self.targetBody:setPos(Position(
            sample.position.x,
            sample.position.y,
            sample.position.z))
        self.targetMotionVelocity = sample.velocity
        self.targetVelocity = sample.velocity
        if self.targetCandidates[1] then
            self.targetCandidates[1].position = sample.position
            self.targetCandidates[1].velocity = sample.velocity
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

    self.testbedConfig = Config.weapons.testbed or {}
    self.seed = self.testbedConfig.seed or 1
    local seedRng = RNG.Create(self.seed)
    assert(seedRng, "weapon testbed could not create its master seed RNG")
    self.contentRng = seedRng
    self.targetSeedRng = RNG.Create(seedRng:get64())
    assert(self.targetSeedRng, "weapon testbed could not create its target seed RNG")
    self.world = Physics.Create()
    self.projectiles = {}
    self.beams = {}
    self.turrets = {}
    self.turretsById = {}
    self.mountIds = MOUNT_IDS
    self.mountCount = #MOUNT_IDS
    self.lastShotOrder = {}
    self.lastImpact = nil
    self.weaponId = Enums.Weapon.Type.Plasma
    self.weapon = WeaponRegistry:get(self.weaponId)
    assert(self.weapon, "missing testbed ship weapon: " .. tostring(self.weaponId))
    self.deferredLightingEnabled = self.testbedConfig.deferredLighting ~= false
    self.previousDeferredLightingEnabled = RenderCoreSystem.settings.deferredLighting
    RenderCoreSystem:setDeferredLightingEnabled(self.deferredLightingEnabled)
    LightManager:setDiagnosticsEnabled(self.testbedConfig.pointLightDiagnostics == true)
    self.lastEffectLightCount = -1
    Log.Info(string.format(
        "WeaponSystem deferred lighting: %s",
        self.deferredLightingEnabled and "enabled" or "disabled"))
    self.targetRespawnDelay = self.testbedConfig.targetRespawnDelay or 3.0
    self.targetMaxHealthConfig = self.testbedConfig.targetMaxHealth or 300
    self.targetSizeClass = self.testbedConfig.targetSizeClass or "small"
    self.targetMotion = self.testbedConfig.targetMotion or { enabled = false }
    self.targetMotionModeIndex = self.targetMotion.startMode or 1
    self.targetMotionPhaseOffset = 0
    self.targetPointOptions = self.testbedConfig.targetPoint or {
        motionAmplitude = 0.08,
        motionFrequency = 0.60,
    }
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
        self.weapon.firePolicy.defaultMode,
        MOUNT_IDS))
    self.control:setActive(self.testbedConfig.aiActive == true)
    self.capacitor = self.capital:add(ConstructComponents.WeaponCapacitor(
        self.testbedConfig.capacitors or {}))
    self.weaponCapacitor = self.capacitor
    self.weaponTrackingComponent = self.capital:add(ConstructComponents.WeaponTracking(
        self.testbedConfig.trackingModule or {}))
    local contentRng = self.contentRng
    assert(contentRng, "testbed could not create its deterministic content RNG")
    local trackingContentSeed = contentRng:get64()
    local burstLaserContentSeed = contentRng:get64()
    local fastPlasmaContentSeed = contentRng:get64()
    local missileContentSeed = contentRng:get64()
    local fastBoltContentSeed = contentRng:get64()
    self.trackingModule = TrackingModuleGenerator:generate({
        universeSeed = self.seed,
        contentSeed = trackingContentSeed,
        tier = self.testbedConfig.trackingTier or 4,
    })
    self.generatedWeapons = {
        burstLaser = WeaponGenerator:generate({
            universeSeed = self.seed,
            contentSeed = burstLaserContentSeed,
            family = "laser",
            variant = "short-burst",
            baseWeaponId = Enums.Weapon.Type.Laser,
            burstCount = 3,
        }),
        fastPlasma = WeaponGenerator:generate({
            universeSeed = self.seed,
            contentSeed = fastPlasmaContentSeed,
            family = "plasma",
            variant = "fast-small",
            baseWeaponId = Enums.Weapon.Type.Plasma,
        }),
        fastBolt = WeaponGenerator:generate({
            universeSeed = self.seed,
            contentSeed = fastBoltContentSeed,
            family = "laser",
            variant = "fast-bolt",
            baseWeaponId = Enums.Weapon.Type.LaserBolt,
            burstCount = 3,
        }),
        missile = WeaponGenerator:generate({
            universeSeed = self.seed,
            contentSeed = missileContentSeed,
            family = "missile",
            variant = "guided",
            baseWeaponId = Enums.Weapon.Type.Plasma,
        }),
    }
    local laserCapacitorBank = self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Laser)[1]
    local plasmaCapacitorBank = self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Plasma)[1]
    assert(laserCapacitorBank and plasmaCapacitorBank,
        "testbed requires separate laser and plasma capacitor banks")
    local capacitorTotal = 0
    local capacitorMax = 0
    for _, bank in ipairs(self.capacitor:getBanks()) do
        capacitorTotal = capacitorTotal + bank.charge
        capacitorMax = capacitorMax + bank.maxCharge
    end
    Log.Info(string.format(
        "WeaponSystem testbed capacitor: %.2f/%.2f charge, laser %.2f/%.2f, plasma %.2f/%.2f, %d banks",
        capacitorTotal,
        capacitorMax,
        laserCapacitorBank.charge,
        laserCapacitorBank.maxCharge,
        plasmaCapacitorBank.charge,
        plasmaCapacitorBank.maxCharge,
        #self.capacitor:getBanks()))
    self.targeting = self.capital:add(ConstructComponents.Targeting(self.weapon.range))
    self.targeting:setAutoAcquireEnabled(self.control:isActive())

    local capitalRadius = self.capital:get(PhysicsComponents.RigidBody):getRadius()
    capitalRadius = math.max(capitalRadius, 0.02)
    -- Starboard is +X in the testbed convention. Keep Z exactly zero so the
    -- target is a side-on LOS case rather than another diagonal presentation.
    local targetDistance = math.max(
        capitalRadius + 0.5,
        math.min(capitalRadius * 4.0, self.weapon.range * 0.80))
    self.targetOrbitAnchorPosition = Position(targetDistance, 0, 0)
    self.targetPosition = Position(
        self.targetOrbitAnchorPosition.x,
        self.targetOrbitAnchorPosition.y,
        self.targetOrbitAnchorPosition.z)
    local initialMotion = self:getActiveTargetMotion()
    if initialMotion.mode == "orbit" then
        local initialSample = WeaponSystem:sampleTargetMotion(
            self.targetOrbitAnchorPosition,
            0,
            initialMotion)
        self.targetPosition = Position(
            initialSample.position.x,
            initialSample.position.y,
            initialSample.position.z)
    end
    self.targetScale = capitalScale * (self.testbedConfig.targetScaleMultiplier or 1.0)
    self:spawnTarget()
    Log.Info("WeaponSystem testbed target motion: " .. self:getTargetMotionLabel())
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

    local capitalShipData = self.capital:get(ConstructComponents.ShipData)
    assert(capitalShipData, "capital ship is missing ShipDataComponent")
    local capitalMesh = capitalShipData:getGeneratedMesh()
    assert(capitalMesh, "capital ship did not retain its generated hull mesh")
    local strictDiscoveryOk, mountsOrError = pcall(function()
        return HullMountDiscovery:discover(
            capitalMesh,
            self.seed,
            MOUNT_SPECS,
            {
                minNormalDot = 0.35,
                mirrorTolerance = 0.0001,
            })
    end)
    local mounts
    if strictDiscoveryOk then
        mounts = mountsOrError
    else
        -- A generated hull can legitimately lack one of the authored mirrored
        -- fore/mid/aft zone pairs for a particular deterministic seed. Keep
        -- the seed and generated mesh unchanged for visual verification; use
        -- deterministic unconstrained surface candidates as a testbed-only
        -- fallback rather than changing production mount discovery semantics.
        local fallbackSpecs = {}
        for _, spec in ipairs(MOUNT_SPECS) do
            table.insert(fallbackSpecs, {
                mountId = spec.mountId,
                normal = spec.normal,
                facing = spec.facing,
            })
        end
        Log.Warn(string.format(
            "WeaponSystem mount discovery fallback for seed %d: %s",
            self.seed,
            tostring(mountsOrError)))
        mounts = HullMountDiscovery:discover(
            capitalMesh,
            self.seed,
            fallbackSpecs,
            {
                minNormalDot = 0.35,
                mirrorTolerance = 0.0001,
            })
    end
    local loadoutByMount = {}
    for _, configuredLoadout in ipairs(self.testbedConfig.loadout or {}) do
        assert(configuredLoadout.mountId,
            "testbed loadout entries require a mount ID")
        local loadout = {}
        for key, value in pairs(configuredLoadout) do
            loadout[key] = value
        end
        if loadout.mountId == "fore_inner_port" then
            loadout.weaponId = nil
            loadout.weaponRef = self.generatedWeapons.fastPlasma.weaponRef
        elseif loadout.mountId == "fore_inner_starboard" then
            loadout.weaponId = nil
            loadout.weaponRef = self.generatedWeapons.burstLaser.weaponRef
        elseif loadout.mountId == "aft_inner_port" then
            loadout.weaponId = nil
            loadout.weaponRef = self.generatedWeapons.missile.weaponRef
        elseif loadout.mountId == "mid_port" then
            loadout.weaponId = nil
            loadout.weaponRef = self.generatedWeapons.fastBolt.weaponRef
        end
        assert(loadout.weaponId or loadout.weaponRef,
            "testbed loadout entry has no weapon identity: " .. loadout.mountId)
        local weapon = loadout.weaponId
            and WeaponRegistry:get(loadout.weaponId)
            or ProceduralCatalog:resolve(loadout.weaponRef)
        assert(weapon,
            "testbed loadout references an unregistered weapon: " .. loadout.mountId)
        loadout.weapon = weapon
        loadoutByMount[loadout.mountId] = loadout
    end

    local beamMountCount = 0
    local projectileMountCount = 0
    local laserProfileIds = {}
    local capitalRigidBodyComponent = self.capital:get(PhysicsComponents.RigidBody)
    local hullClearance = math.max(capitalRadius * 0.04, 0.06)
    local hullScale = math.max(capitalRigidBodyComponent:getScale(), 0.0001)
    local localClearance = hullClearance / hullScale
    for _, mount in ipairs(mounts) do
        local loadout = loadoutByMount[mount.mountId]
        assert(loadout, "testbed is missing a loadout for mount: " .. mount.mountId)
        local weapon = loadout.weapon
        assert(weapon and weapon.effect,
            "testbed mount has no resolved effect: " .. mount.mountId)
        assert(weapon.effect.kind == Enums.Weapon.Effect.Beam
            or weapon.effect.kind == Enums.Weapon.Effect.Projectile,
            "testbed mount has an unsupported effect kind: " .. mount.mountId)
        mount.weaponId = loadout.weaponId
        mount.weaponRef = loadout.weaponRef
        mount.trackingModuleRef = self.trackingModule.ref
        mount.trackingModuleStats = self.trackingModule.stats
        if weapon.effect.kind == Enums.Weapon.Effect.Beam then
            beamMountCount = beamMountCount + 1
            local profile = WeaponRegistry:getLaserProfile(weapon)
            assert(profile, "beam mount has no resolved laser profile: " .. mount.mountId)
            laserProfileIds[profile.id] = true
        else
            projectileMountCount = projectileMountCount + 1
        end
        mount.bodyLocalPosition = Position(
            mount.localPosition.x + mount.surfaceNormal.x * localClearance,
            mount.localPosition.y + mount.surfaceNormal.y * localClearance,
            mount.localPosition.z + mount.surfaceNormal.z * localClearance)
        mount.position = capitalRigidBodyComponent:toWorldScaled(mount.bodyLocalPosition)
    end
    assert(beamMountCount > 0 and projectileMountCount > 0,
        "testbed loadout must contain both beam and projectile registry effects")
    Log.Info(string.format(
        "WeaponSystem testbed loadout: %d beam mounts, %d projectile mounts, modern ECS effects",
        beamMountCount,
        projectileMountCount))
    local laserProfileNames = {}
    for _, profileId in ipairs(LaserProfileRegistry:getIds()) do
        if laserProfileIds[profileId] then
            table.insert(laserProfileNames, LaserProfileRegistry:get(profileId).name)
        end
    end
    Log.Info("WeaponSystem testbed laser profiles: " .. table.concat(laserProfileNames, ", "))
    Log.Info("WeaponSystem testbed variants: burst laser=3, fast plasma, fast laser bolt=3, guided missile")

    local generatedTurrets = TurretLoadoutGenerator:create(self.capital, mounts, self.weapon)
    for _, mount in ipairs(generatedTurrets) do
        local component = mount.entity:get(ConstructComponents.Turret)
        local body = mount.entity:get(PhysicsComponents.RigidBody):getRigidBody()
        -- Turret bodies are aim sources, not obstacles in this testbed. Native
        -- non-collidable bodies are excluded from ray casts, and these turret
        -- bodies are not registered in the capital's physics world.
        local record = {
            mountId = mount.mountId,
            entity = mount.entity,
            component = component,
            body = body,
            localPosition = mount.localPosition,
            bodyLocalPosition = mount.bodyLocalPosition,
            localRotation = mount.localRotation or component:getLocalRotation(),
            surfaceNormal = mount.surfaceNormal,
            parentBody = self.capitalBody,
            zoneMatch = mount.zoneMatch,
            sideMatch = mount.sideMatch,
        }
        table.insert(self.turrets, record)
        self.turretsById[mount.mountId] = record
    end

    self.cameraController:setTarget(self.capital)
    Log.Info(string.format(
        "WeaponSystem testbed: capital radius %.4f, target distance %.4f, %d turrets ready",
        capitalRadius,
        targetDistance,
        #generatedTurrets))
end

function WeaponSystemTestbed:removeBeam(index)
    local beam = table.remove(self.beams, index)
    if beam and beam.entity:isValid() then
        Registry:destroyEntity(beam.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
end

function WeaponSystemTestbed:clearBeams()
    for index = #self.beams, 1, -1 do
        self:removeBeam(index)
    end
end

---@param state table|nil
---@return string[]
function WeaponSystemTestbed:collectNonePointDiagnostics(state)
    state = state or self
    local noneMounts = {}
    for mountId, reason in pairs(state.sightReasonByMount or {}) do
        if reason == "none" then
            local pointState = state.targetPointByMount and state.targetPointByMount[mountId]
            local pointPosition = pointState and pointState.position
            table.insert(noneMounts, string.format(
                "%s@(%.3f,%.3f,%.3f)",
                mountId,
                pointPosition and pointPosition.x or 0,
                pointPosition and pointPosition.y or 0,
                pointPosition and pointPosition.z or 0))
        end
    end
    table.sort(noneMounts)
    return noneMounts
end

---@param mount table
---@param solution table
---@param weapon table
---@param shotSerial integer
function WeaponSystemTestbed:spawnBeam(mount, solution, weapon, shotSerial)
    assert(weapon.effect.kind == Enums.Weapon.Effect.Beam,
        "projectile weapons cannot spawn beams")
    local presentation = WeaponRegistry:getPresentation(weapon)
    local beamTargetPoint = solution.targetPoint or solution.position
    local entity = ConstructEntities.Beam(shotSerial, {}, {
        source = mount.entity,
        target = self.target,
        effect = weapon.effect,
        visual = presentation,
        damagePerSecond = WeaponRegistry:getDamagePerSecond(weapon),
        duration = weapon.effect.duration or weapon.cooldown,
        targetPoint = beamTargetPoint,
        targetPointLocal = solution.targetPointLocal,
        aimAngles = solution.aimAngles,
        swayPhase = solution.swayPhase,
        swayTime = solution.swayTime,
        swayBasis = solution.swayBasis,
    })
    table.insert(self.beams, {
        entity = entity,
        component = entity:get(ConstructComponents.Beam),
        sourceBody = mount.body,
        targetBody = self.targetBody,
        targetPoint = beamTargetPoint,
        targetPointLocal = solution.targetPointLocal,
        mountId = mount.mountId,
        shotSerial = shotSerial,
        lightColor = presentation.lightColor,
        lightRadius = presentation.lightRadius,
        lightIntensity = presentation.lightIntensity,
    })
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
    local projectile = weapon.effect
    assert(projectile.kind == Enums.Weapon.Effect.Projectile, "beam weapons cannot spawn projectiles")
    local visual = projectile.visual
    local scale = projectile.speed / length
    local velocity = Vec3f(
        dx * scale + sourceVelocity.x,
        dy * scale + sourceVelocity.y,
        dz * scale + sourceVelocity.z)
    local pulse = Pulse:new()
    pulse.pos = sourcePosition
    pulse.vel = velocity
    pulse.dir = velocity:normalize()
    pulse.lifeMax = projectile.lifetime
    pulse.life = pulse.lifeMax
    pulse.dist = 0

    local entity = ConstructEntities.Projectile(shotSerial, {}, {
        source = mount.entity,
        effect = projectile,
        position = sourcePosition,
        velocity = velocity,
        damage = weapon.damage,
        lifetime = projectile.lifetime,
        scale = projectile.scale,
        guidance = projectile.guidance,
        targetBody = self.targetBody,
        targetEntity = self.target,
        bodyMesh = nil,
    })
    local body = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    table.insert(self.projectiles, {
        entity = entity,
        body = body,
        component = entity:get(ConstructComponents.Projectile),
        mountId = mount.mountId,
        shotSerial = shotSerial,
        effect = pulse,
        pulseDistance = 0,
        lightColor = visual.lightColor,
        lightRadius = visual.lightRadius,
        lightIntensity = visual.lightIntensity,
        pColorR = visual.bodyColor.r,
        pColorG = visual.bodyColor.g,
        pColorB = visual.bodyColor.b,
        pulseHeadSize = visual.headSize,
        pulseTailWidth = visual.tailWidth,
        pulseTailLength = visual.tailLength,
        shaderKey = projectile.shaderKey or (projectile.archetype == "missile" and "missile"),
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
    WeaponActions.Orbit:update(dt)
    WeaponActions.OrbitPhase:update(dt)

    if WeaponActions.Orbit:isPressed() then
        self:setTargetOrbitMode((self.targetMotionModeIndex or 1) + 1)
    elseif WeaponActions.OrbitPhase:isPressed() then
        self:advanceTargetOrbitPhase()
    end

    if WeaponActions.AI:isPressed() then
        local active = not self.control:isActive()
        self.control:setActive(active)
        self.targeting:setAutoAcquireEnabled(active)
        if active then
            self.control:setSequenceIndex(1)
        end
    end

    if not self.control:isActive() and WeaponActions.Volley:isPressed() then
        self.control:setMode(Enums.Weapon.FireMode.Volley)
        self.control:setSequenceIndex(1)
    elseif not self.control:isActive() and WeaponActions.Sequence:isPressed() then
        self.control:setMode(Enums.Weapon.FireMode.Sequence)
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
        self.targetMotionTime = 0
        self.targetMotionPhaseOffset = 0
        self:resetTargetMotionPosition()
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

function WeaponSystemTestbed:syncTurretTransforms()
    local capitalRigidBodyComponent = self.capital:get(PhysicsComponents.RigidBody)
    local parentRotation = self.capitalBody:getRot()
    for _, turret in ipairs(self.turrets) do
        turret.body:setPos(capitalRigidBodyComponent:toWorldScaled(
            turret.bodyLocalPosition or turret.localPosition))
        turret.body:setRot(parentRotation * turret.localRotation)
    end
end

---@param data EventData
function WeaponSystemTestbed:onSim(data)
    local dt = data:deltaTime()
    self:updateTargetLifecycle(dt)
    self:syncTurretTransforms()
    self.world:update(dt)
    AIWeaponSystem:update(self, dt)
    WeaponTrackingSystem:update(self, dt)
    WeaponSystem:update(self, dt)
    if self.lastTargetPoint then
        local pointTime = self.targetPointTime or 0
        if not self.targetPointLogTime or pointTime - self.targetPointLogTime >= 0.5 then
            local point = self.lastTargetPoint.position
            Log.Info(string.format(
                "WeaponSystem target point: surface triangle %d at (%.3f, %.3f, %.3f)",
                self.lastTargetPoint.triangleIndex,
                point.x,
                point.y,
                point.z))
            local sightMounts = {}
            for _, mount in ipairs(self.turrets or {}) do
                if self.sightByMount and self.sightByMount[mount.mountId] then
                    table.insert(sightMounts, mount.mountId)
                end
            end
            local reasons = self.sightReasons or {}
            local noneMounts = self:collectNonePointDiagnostics()
            Log.Info(string.format(
                "WeaponSystem sight: %d/%d ready %d/%d target %d hull %d none %d other %d mounts [%s] nonePoints [%s]",
                self.sightCount or 0,
                self.mountCount or 0,
                self.readyCount or 0,
                self.mountCount or 0,
                reasons.target or 0,
                reasons.hull or 0,
                reasons.none or 0,
                reasons.other or 0,
                table.concat(sightMounts, ","),
                table.concat(noneMounts, ",")))
            self.targetPointLogTime = pointTime
        end
    end
end

---@param data EventData
function WeaponSystemTestbed:onPostSim(data)
    ProjectileSystem:update(self, data:deltaTime())
    BeamSystem:update(self, data:deltaTime())
    for index = #(self.lightEffects or {}), 1, -1 do
        local effect = self.lightEffects[index]
        if not effect or not effect:isValid() then
            table.remove(self.lightEffects, index)
        end
    end
end

---@param data EventData
function WeaponSystemTestbed:onRender(data)
    RenderCoreSystem:render(data)
    self.effectLights = LightManager:getPointLights()
    if #self.effectLights ~= self.lastEffectLightCount then
        Log.Info(string.format(
            "WeaponSystem deferred lighting: %d weapon point lights",
            #self.effectLights))
        self.lastEffectLightCount = #self.effectLights
    end

    self:immediateUI(function()
        local mode = self.control:getMode()
        local health = self.targetHealth and self.targetHealth:getCurrentHealth() or 0
        local targetState = self.target
            and string.format("%.0f/%.0f", health, self.targetMaxHealth)
            or string.format("respawn %.1fs", self.targetRespawnRemaining or 0)
        local lastShot = #self.lastShotOrder > 0 and table.concat(self.lastShotOrder, ", ") or "-"
        local mountCount = self.mountCount or #self.turrets
        local impactState = self.lastImpact or self.lastBeamImpact
        local targetMotionPosition = self.targetBody and self.targetBody:getPos()
        local targetMotionVelocity = self.targetMotionVelocity or Vec3f()
        local targetMotionLabel = self:getTargetMotionLabel()
        local targetPointState = self.lastTargetPoint
        local targetPointPosition = targetPointState and targetPointState.position
        local capacitorCharge = 0
        local capacitorMax = 0
        local laserCapacitorBank = self.capacitor
            and self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Laser)[1]
        local plasmaCapacitorBank = self.capacitor
            and self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Plasma)[1]
        for _, bank in ipairs(self.capacitor and self.capacitor:getBanks() or {}) do
            capacitorCharge = capacitorCharge + bank.charge
            capacitorMax = capacitorMax + bank.maxCharge
        end
        local impact = impactState and string.format(
            "hit %s at (%.3f, %.3f, %.3f)",
            impactState.mountId or "?",
            impactState.position.x,
            impactState.position.y,
            impactState.position.z) or "-"
        local sightMounts = {}
        for _, turret in ipairs(self.turrets) do
            if self.sightByMount and self.sightByMount[turret.mountId] then
                table.insert(sightMounts, turret.mountId)
            end
        end
        local lines = {
            "WeaponSystem Testbed",
            "LMB: fire   1: volley   2: sequence   A: toggle AI   O: next orbit   P: phase   R: reset target",
            string.format(
                "Mode: %s   AI: %s   Target HP: %s   Projectiles: %d   Beams: %d",
                mode,
                self.control:isActive() and "active" or "off",
                targetState,
                #self.projectiles,
                #self.beams),
            string.format(
                "Deferred lighting: %s   Weapon point lights: %d",
                self.deferredLightingEnabled and "enabled" or "disabled",
                self.lastEffectLightCount >= 0 and self.lastEffectLightCount or 0),
            string.format(
                "Target motion: %s pos (%.3f, %.3f, %.3f) vel (%.3f, %.3f, %.3f)",
                targetMotionLabel,
                targetMotionPosition and targetMotionPosition.x or 0,
                targetMotionPosition and targetMotionPosition.y or 0,
                targetMotionPosition and targetMotionPosition.z or 0,
                targetMotionVelocity.x,
                targetMotionVelocity.y,
                targetMotionVelocity.z),
            string.format(
                "Target point: %s (%.3f, %.3f, %.3f) triangle %s",
                targetPointPosition and "surface" or "center",
                targetPointPosition and targetPointPosition.x or (self.targetPosition and self.targetPosition.x or 0),
                targetPointPosition and targetPointPosition.y or (self.targetPosition and self.targetPosition.y or 0),
                targetPointPosition and targetPointPosition.z or (self.targetPosition and self.targetPosition.z or 0),
                targetPointState and tostring(targetPointState.triangleIndex) or "-"),
            string.format(
                "Capacitor: %.2f/%.2f   Laser %.2f/%.2f   Plasma %.2f/%.2f",
                capacitorCharge,
                capacitorMax,
                laserCapacitorBank and laserCapacitorBank.charge or 0,
                laserCapacitorBank and laserCapacitorBank.maxCharge or 0,
                plasmaCapacitorBank and plasmaCapacitorBank.charge or 0,
                plasmaCapacitorBank and plasmaCapacitorBank.maxCharge or 0),
            string.format(
                "Capacity: %s   Inter-shot gap: %.3f",
                mode == Enums.Weapon.FireMode.Volley and "burst" or "sustain",
                self.control.interShotGap or 0),
            string.format(
                "Capital radius: %.2f   Orbit radius: %.2f   Sight: %d/%d   Ready: %d/%d",
                self.capitalRadius or 0,
                self.targetDistance or 0,
                self.sightCount or 0,
                mountCount,
                self.readyCount or 0,
                mountCount),
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
            "Sight mounts: " .. (#sightMounts > 0 and table.concat(sightMounts, ", ") or "-"),
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
    LightManager:clearPointLights()
    LightManager:setDiagnosticsEnabled(false)
    for _, effect in ipairs(self.lightEffects or {}) do
        if effect and effect:isValid() then
            Registry:destroyEntity(effect, Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
    self.lightEffects = {}
    self:clearBeams()
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
    if self.previousDeferredLightingEnabled ~= nil then
        RenderCoreSystem:setDeferredLightingEnabled(self.previousDeferredLightingEnabled)
        self.previousDeferredLightingEnabled = nil
    end
    CameraManager:unregisterCamera("WeaponOrbit")
end

return WeaponSystemTestbed
