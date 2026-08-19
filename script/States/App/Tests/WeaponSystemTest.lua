local Application = require("States.Application")
local Registry = require("Core.ECS.Registry")
local ffi = require("ffi")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local WeaponTrackingSystem = require("Modules.Constructs.Systems.WeaponTrackingSystem")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local ProjectileRegistry = require("Shared.Registries.ProjectileRegistry")
local BeamRegistry = require("Shared.Registries.BeamRegistry")
local BeamSystem = require("Modules.Constructs.Systems.BeamSystem")
local BeamAimHelper = require("Shared.Helpers.BeamAimHelper")
local WeaponSystemTestbed = require("States.App.Tests.Testbeds.WeaponSystem")
local ConstructComponents = require("Modules.Constructs.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents = require("Modules.Core.Components")
local RenderingComponents = require("Modules.Rendering.Components")
local PointLightSystem = require("Modules.Rendering.Systems.PointLightSystem")
local LightManager = require("Modules.Rendering.Managers.LightManager")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")
local RaycastHelper = require("Shared.Helpers.RaycastHelper")
local HullMountDiscovery = require("Modules.Constructs.Managers.Generators.HullMountDiscovery")
local ShipCapital = require("Legacy.Systems.Gen.ShipCapital")
local TrackingModuleGenerator = require("Shared.Content.TrackingModuleGenerator")
local WeaponGenerator = require("Shared.Content.WeaponGenerator")
local ProjectileSystem = require("Modules.Constructs.Systems.ProjectileSystem")

---@class WeaponSystemTest: Application
local WeaponSystemTest = Subclass("WeaponSystemTest", Application)

local function assertNear(actual, expected, epsilon, label)
    assert(math.abs(actual - expected) <= epsilon,
        string.format("%s: expected %.6f, got %.6f", label, expected, actual))
end

local function assertVec3Near(actual, expected, epsilon, label)
    assertNear(actual.x, expected.x, epsilon, label .. ".x")
    assertNear(actual.y, expected.y, epsilon, label .. ".y")
    assertNear(actual.z, expected.z, epsilon, label .. ".z")
end

local function vecDistance(a, b)
    local dx = a.x - b.x
    local dy = a.y - b.y
    local dz = a.z - b.z
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

local function assertPointLightShaderContract()
    local pointShaderFile = assert(io.open("res/shader/fragment/light/point.glsl", "r"),
        "point-light shader should be available for the deferred-lighting contract")
    local pointShaderSource = pointShaderFile:read("*a")
    pointShaderFile:close()
    assert(pointShaderSource:find("depth <= kMinDistance", 1, true),
        "point lights should reject invalid zero-depth environment pixels")
    assert(pointShaderSource:find("mat != Material_NoShade", 1, true),
        "point lights should affect every shaded environment material")
end

local function runImpactDissipationRegression(lightProjectileEntity, targetEntity, projectileBody)
    local impactProjectileComponent = lightProjectileEntity:get(ConstructComponents.Projectile)
    local impactTargetBody = targetEntity:get(PhysicsComponents.RigidBody):getRigidBody()
    impactTargetBody:setPos(Position(0, 0, 5))
    projectileBody:setPos(Position(0, 0, 0))
    impactProjectileComponent.previousPosition = Position(0, 0, 0)
    impactProjectileComponent:setVelocity(Vec3f(0, 0, 10))
    impactProjectileComponent.remainingLifetime = 3
    local impactProjectile = {
        entity = lightProjectileEntity,
        body = projectileBody,
        component = impactProjectileComponent,
        effect = { lifeMax = 3, life = 3, vel = Vec3f(0, 0, 10) },
        pulseDistance = 0,
        shotSerial = 8011,
        mountId = "impact_position_test",
    }
    local impactState = {
        projectiles = { impactProjectile },
        targetHealth = CoreComponents.Health(100),
        targetBody = impactTargetBody,
        targetRadius = 0.5,
        removeProjectile = function(state, index)
            table.remove(state.projectiles, index)
        end,
    }
    ProjectileSystem:update(impactState, 0.6)
    assert(#impactState.projectiles == 1 and impactState.lastImpact,
        "impact projectile should enter dissipation instead of overshooting and disappearing")
    assertNear(impactState.lastImpact.position.z, 4.5, 0.0001,
        "impact should record the actual segment hit position")
    assertNear(projectileBody:getPos().z, 4.5, 0.0001,
        "dissipating projectile body should remain at the impact position")
    assertNear(impactProjectileComponent.previousPosition.z, 4.5, 0.0001,
        "dissipating projectile previous position should remain at impact")
    assertNear(impactProjectile.effect.pos.z, 4.5, 0.0001,
        "dissipating projectile visual should remain at impact")
    local impactPointLights = PointLightSystem:update()
    local impactPointLight
    for _, light in ipairs(impactPointLights) do
        if light.entityId == lightProjectileEntity.id then
            impactPointLight = light
            break
        end
    end
    assert(impactPointLight, "dissipating projectile should retain its point light")
    assertNear(impactPointLight.pos.z, 4.5, 0.0001,
        "dissipating projectile point light should remain at impact")
    ProjectileSystem:update(impactState, 0.35)
    impactTargetBody:setPos(Position(0, 0, 50))
    projectileBody:setPos(Position(4, 5, 6))
    impactProjectileComponent.previousPosition = Position(4, 5, 6)
    local pointLight = lightProjectileEntity:get(RenderingComponents.PointLight)
    pointLight:setIntensity(0.20)
    pointLight:setEnabled(true)
end

local function runTrackingRegressions()
    local trackingConfig = {
        predictionHorizon = 12,
        predictionDamping = 0.25,
        turnRateFloor = 0.0001,
        turnRateLimit = 4,
    }
    local curvedTrack = {
        position = Vec3f(100, 0, 0),
        velocity = Vec3f(0, 10, 0),
        acceleration = Vec3f(-1, 0, 0),
        rotation = Quat.Identity(),
        angularVelocity = Vec3f(0, 0, 0),
        confidence = 1,
    }
    local curvedSolution = WeaponTrackingSystem:solveIntercept(
        Vec3f(0, 0, 0),
        Vec3f(0, 0, 0),
        curvedTrack,
        Vec3f(0, 0, 0),
        1,
        50,
        trackingConfig,
        500)
    assert(curvedSolution and curvedSolution.model == "intercept",
        "coordinated-turn tracking should bracket a curved-motion intercept")
    assertNear(
        vecDistance(curvedSolution.position, Vec3f(0, 0, 0)),
        curvedSolution.time * 50,
        0.001,
        "curved intercept range")

    local contentRng = RNG.Create(42)
    assert(contentRng, "procedural-generation regression could not create its master RNG")
    local trackingContentSeed = contentRng:get64()
    local trackingDifferentSeed = contentRng:get64()
    local burstLaserContentSeed = contentRng:get64()
    local defaultBurstContentSeed = contentRng:get64()
    local fastPlasmaContentSeed = contentRng:get64()
    local fastPlasmaDifferentContentSeed = contentRng:get64()
    local missileContentSeed = contentRng:get64()
    local fastBoltContentSeed = contentRng:get64()
    local generatedModuleA = TrackingModuleGenerator:generate({
        universeSeed = 42,
        contentSeed = trackingContentSeed,
        tier = 4,
    })
    local generatedModuleB = TrackingModuleGenerator:generate({
        universeSeed = 42,
        contentSeed = trackingContentSeed,
        tier = 4,
    })
    local generatedModuleC = TrackingModuleGenerator:generate({
        universeSeed = 42,
        contentSeed = trackingDifferentSeed,
        tier = 4,
    })
    local generatedModuleD = TrackingModuleGenerator:generate({
        universeSeed = 43,
        contentSeed = trackingContentSeed,
        tier = 4,
    })
    assert(generatedModuleA == generatedModuleB,
        "same procedural tracking-module identity must resolve to one catalog record")
    assert(generatedModuleA.fingerprint == generatedModuleB.fingerprint,
        "same procedural tracking-module seed must reproduce its fingerprint")
    assert(generatedModuleA.ref.canonicalKey ~= generatedModuleC.ref.canonicalKey,
        "different procedural tracking-module seeds must not collide")
    assert(generatedModuleA.fingerprint ~= generatedModuleC.fingerprint,
        "different procedural tracking-module seeds must change generated stats")
    assert(generatedModuleA.fingerprint ~= generatedModuleD.fingerprint,
        "different universe seeds must change generated tracking-module stats")
    assert(generatedModuleA.stats.predictionHorizon >= 3
        and generatedModuleA.stats.predictionHorizon <= 10,
        "generated tracking-module prediction horizon must stay bounded")

    local burstLaser = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = burstLaserContentSeed,
        family = "laser",
        variant = "short-burst",
        baseWeaponId = Enums.Weapon.Type.Laser,
        burstCount = 3,
    })
    local defaultBurstLaser = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = defaultBurstContentSeed,
        family = "laser",
        variant = "short-burst",
        baseWeaponId = Enums.Weapon.Type.Laser,
    })
    local fastPlasma = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = fastPlasmaContentSeed,
        family = "plasma",
        variant = "fast-small",
        baseWeaponId = Enums.Weapon.Type.Plasma,
    })
    local fastPlasmaSameSeed = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = fastPlasmaContentSeed,
        family = "plasma",
        variant = "fast-small",
        baseWeaponId = Enums.Weapon.Type.Plasma,
    })
    local fastPlasmaDifferentSeed = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = fastPlasmaDifferentContentSeed,
        family = "plasma",
        variant = "fast-small",
        baseWeaponId = Enums.Weapon.Type.Plasma,
    })
    local fastBolt = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = fastBoltContentSeed,
        family = "laser",
        variant = "fast-bolt",
        baseWeaponId = Enums.Weapon.Type.LaserBolt,
        burstCount = 3,
    })
    local fastBoltSameSeed = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = fastBoltContentSeed,
        family = "laser",
        variant = "fast-bolt",
        baseWeaponId = Enums.Weapon.Type.LaserBolt,
        burstCount = 3,
    })
    local missile = WeaponGenerator:generate({
        universeSeed = 42,
        contentSeed = missileContentSeed,
        family = "missile",
        variant = "guided",
        baseWeaponId = Enums.Weapon.Type.Plasma,
    })
    assert(burstLaser.effect.burst and burstLaser.effect.burst.count == 3,
        "generated short-burst laser must carry burst cadence data")
    assert(defaultBurstLaser.effect.burst
        and defaultBurstLaser.effect.burst.count >= 2
        and defaultBurstLaser.effect.burst.count <= 4,
        "default generated burst count must come from the seeded RNG range")
    assert(fastPlasma.effect.speed > WeaponRegistry:get(Enums.Weapon.Type.Plasma).effect.speed
        and fastPlasma.effect.scale < WeaponRegistry:get(Enums.Weapon.Type.Plasma).effect.scale,
        "generated fast plasma must be smaller and faster than base plasma")
    local laserBoltBase = WeaponRegistry:get(Enums.Weapon.Type.LaserBolt)
    assert(laserBoltBase and laserBoltBase.effect.kind == Enums.Weapon.Effect.Projectile,
        "laser bolt base weapon must resolve through the projectile registry")
    assert(fastBolt.effect.kind == Enums.Weapon.Effect.Projectile
        and fastBolt.effect.speed > laserBoltBase.effect.speed
        and fastBolt.effect.scale < laserBoltBase.effect.scale,
        "generated fast laser bolt must be smaller and faster than its base projectile")
    assert(fastBolt.effect.burst and fastBolt.effect.burst.count == 3,
        "fast laser bolt must support an explicit three-shot burst")
    assert(fastBolt.effect.shaderKey == "laserbolt",
        "fast laser bolt must select its dedicated presentation shader")
    assert(fastBolt.effect.archetype == "laser-bolt",
        "fast laser bolt must preserve its projectile archetype")
    assert(fastBolt == fastBoltSameSeed
        and fastBolt.fingerprint == fastBoltSameSeed.fingerprint,
        "same procedural seed must resolve to the same fast laser bolt")
    assert(fastPlasma == fastPlasmaSameSeed
        and fastPlasma.fingerprint == fastPlasmaSameSeed.fingerprint,
        "same procedural weapon seed must resolve to the same generated record")
    assert(fastPlasma.fingerprint ~= fastPlasmaDifferentSeed.fingerprint,
        "different procedural weapon seeds must change generated stats")
    local proceduralTurret = ConstructComponents.Turret(
        "generated_fast_plasma",
        Position(),
        {
            weaponRef = fastPlasma.weaponRef,
            trackingModuleStats = generatedModuleA.stats,
        })
    assert(proceduralTurret.weaponRef.canonicalKey == fastPlasma.weaponRef.canonicalKey,
        "turret components must preserve procedural weapon references")
    assert(missile.effect.guidance and missile.effect.guidance.kind == "missile",
        "generated missile must carry guidance data")
    assert(burstLaser.weaponRef.kind == "procedural"
        and fastPlasma.weaponRef.kind == "procedural"
        and fastBolt.weaponRef.kind == "procedural"
        and missile.weaponRef.kind == "procedural",
        "generated weapons must use procedural refs instead of closed enum IDs")

    local missileVelocity = Vec3f(0, 0, -10)
    local missileComponent = {
        guidance = missile.effect.guidance,
        targetBody = {
            getPos = function()
                return Position(10, 0, 0)
            end,
            getVelocity = function()
                return Vec3f(0, 0, 0)
            end,
        },
        guidanceFuel = 1,
        guidanceState = {},
        getVelocity = function(self)
            return self.velocity
        end,
        setVelocity = function(self, velocity)
            self.velocity = velocity
        end,
        velocity = missileVelocity,
    }
    ProjectileSystem:updateGuidance(missileComponent, Position(0, 0, 0), 0.1)
    assert(missileComponent.guidanceState.turnRate <= missile.effect.guidance.maximumTurnRate + 0.0001,
        "missile guidance must respect its maximum turn rate")

    local antiParallelTarget = {
        getPos = function()
            return Position(0, 0, 10)
        end,
        getVelocity = function()
            return Vec3f(0, 0, 0)
        end,
    }
    local antiParallelComponent = {
        guidance = {
            maximumAcceleration = 0,
            maximumSpeed = 10,
            maximumTurnRate = math.rad(30),
            fuelLifetime = 10,
        },
        targetBody = antiParallelTarget,
        guidanceFuel = 10,
        guidanceState = {},
        getVelocity = function(self)
            return self.velocity
        end,
        setVelocity = function(self, velocity)
            self.velocity = velocity
        end,
        velocity = Vec3f(0, 0, -10),
    }
    ProjectileSystem:updateGuidance(antiParallelComponent, Position(0, 0, 0), 1)
    assert(antiParallelComponent.guidanceState.turnRate <= math.rad(30) + 0.0001
        and antiParallelComponent.guidanceState.turnRate > 0,
        "anti-parallel missile guidance must obey its turn-rate limit")

    local fuelComponent = {
        guidance = missile.effect.guidance,
        targetBody = missileComponent.targetBody,
        guidanceFuel = 0.05,
        guidanceState = {},
        getVelocity = function(self)
            return self.velocity
        end,
        setVelocity = function(self, velocity)
            self.velocity = velocity
        end,
        velocity = Vec3f(0, 0, -10),
    }
    ProjectileSystem:updateGuidance(fuelComponent, Position(0, 0, 0), 0.1)
    assert(fuelComponent.guidanceFuel == 0 and fuelComponent.guidanceState.fuelExpired,
        "missile guidance must mark fuel exhaustion")
    local ballisticVelocity = fuelComponent.velocity
    ProjectileSystem:updateGuidance(fuelComponent, Position(0, 0, 0), 0.1)
    assertVec3Near(fuelComponent.velocity, ballisticVelocity, 0.000001,
        "fuel-exhausted missile should continue ballistically")

    local guidedImpactHealth = {
        current = 100,
        isDestroyed = function(self)
            return self.current <= 0
        end,
        getCurrentHealth = function(self)
            return self.current
        end,
        setCurrentHealth = function(self, value)
            self.current = value
        end,
    }
    local guidedImpactTargetBody = {
        getPos = function()
            return Position(0, 0, -1)
        end,
        getVelocity = function()
            return Vec3f(0, 0, 0)
        end,
    }
    local guidedImpactComponent = {
        guidance = {
            maximumAcceleration = 0,
            maximumSpeed = 20,
            maximumTurnRate = math.pi,
            proximityRadius = 0.05,
            fuelLifetime = 2,
        },
        targetBody = guidedImpactTargetBody,
        guidanceFuel = 2,
        guidanceState = {},
        previousPosition = nil,
        remainingLifetime = 2,
        damage = 25,
        getVelocity = function(self)
            return self.velocity
        end,
        setVelocity = function(self, velocity)
            self.velocity = velocity
        end,
        getDamage = function(self)
            return self.damage
        end,
        velocity = Vec3f(0, 0, -10),
    }
    local guidedImpactBody = {
        position = Position(0, 0, 0),
        getPos = function(self)
            return self.position
        end,
        setPos = function(self, position)
            self.position = position
        end,
    }
    local guidedProjectileState = {
        projectiles = {
            {
                component = guidedImpactComponent,
                body = guidedImpactBody,
                effect = {},
                shotSerial = 17,
                mountId = "guided_test",
            },
        },
        targetHealth = guidedImpactHealth,
        targetBody = guidedImpactTargetBody,
        targetRadius = 0.05,
        removeProjectile = function(state, index)
            table.remove(state.projectiles, index)
        end,
    }
    ProjectileSystem:update(guidedProjectileState, 0.1)
    assert(#guidedProjectileState.projectiles == 0
        and guidedImpactHealth.current == 75
        and guidedProjectileState.lastImpact
        and guidedProjectileState.lastImpact.shotSerial == 17,
        "guided projectile should collide, apply damage, and complete its lifecycle")

    local staleBurstComponent = { burstRemaining = 2, burstGap = 0.1 }
    local staleBurstState = {
        control = { interShotGap = 0.5 },
        weaponTrackingComponent = {},
        turrets = { { component = staleBurstComponent } },
        weaponTargetBody = nil,
        targetBody = nil,
    }
    WeaponSystem:update(staleBurstState, 0.1)
    assert(staleBurstComponent.burstRemaining == 0
        and staleBurstComponent.burstGap == 0
        and staleBurstState.control.interShotGap == 0,
        "pending weapon bursts must clear when their target disappears")

    local burstTurret = {}
    local burstWeapon = {
        effect = { burst = { count = 3, gap = 0.05 } },
        interShotGap = 0.05,
    }
    assert(WeaponSystem:advanceBurst(burstTurret, burstWeapon) == 2
        and burstTurret.burstRemaining == 2
        and burstTurret.burstGap == 0.05,
        "first shot of a three-shot burst must leave two shots pending")
    assert(WeaponSystem:advanceBurst(burstTurret, burstWeapon) == 1
        and burstTurret.burstRemaining == 1,
        "second shot of a three-shot burst must leave one shot pending")
    assert(WeaponSystem:advanceBurst(burstTurret, burstWeapon) == 0
        and burstTurret.burstRemaining == 0
        and burstTurret.burstGap == 0,
        "third shot of a three-shot burst must complete the burst")

    local limitTurret = {
        yaw = 0,
        pitch = 0,
        yawMin = -0.1,
        yawMax = 0.1,
        pitchMin = -0.1,
        pitchMax = 0.1,
        traverseRate = 10,
    }
    local limitAim = WeaponSystem:aimTurret(
        limitTurret,
        Vec3f(0, 0, 0),
        Vec3f(1, 0, 0),
        1,
        { tracking = { traverseRate = 10, aimTolerance = 0.01 } })
    assert(not limitAim.withinLimits and not limitAim.ready,
        "turret at a mechanical limit must not report a false ready state")
end

function WeaponSystemTest:onInit()
    self.finished = false
end

function WeaponSystemTest:onSim()
    if self.finished then return end

    local solution = WeaponSystem:solveIntercept(
        Vec3f(0, 0, 0),
        Vec3f(0, 0, 0),
        Vec3f(100, 0, 0),
        Vec3f(0, 0, 0),
        50)

    assert(solution, "stationary target should produce an intercept")
    assertNear(solution.time, 2.0, 0.0001, "intercept time")
    assertVec3Near(solution.position, Vec3f(100, 0, 0), 0.0001, "intercept position")

    runTrackingRegressions()

    local motionOrigin = Vec3f(5, 2, 0)
    local motionConfig = {
        enabled = true,
        axis = "y",
        amplitude = 0.42,
        frequency = 0.55,
        phase = 0.25,
    }
    local motionA = WeaponSystem:sampleTargetMotion(motionOrigin, 1.75, motionConfig)
    local motionB = WeaponSystem:sampleTargetMotion(motionOrigin, 1.75, motionConfig)
    assertVec3Near(motionA.position, motionB.position, 0.0001,
        "target motion should be repeatable")
    assertVec3Near(motionA.velocity, motionB.velocity, 0.0001,
        "target motion velocity should be repeatable")
    assertNear(motionA.position.x, motionOrigin.x, 0.0001,
        "target motion should preserve the non-motion x axis")
    assertNear(motionA.position.z, motionOrigin.z, 0.0001,
        "target motion should preserve the non-motion z axis")
    assert(math.abs(motionA.position.y - motionOrigin.y) <= motionConfig.amplitude + 0.0001,
        "target motion should remain within its configured amplitude")
    assert(math.abs(motionA.velocity.y) <= motionConfig.amplitude
        * 2 * math.pi * motionConfig.frequency + 0.0001,
        "target motion velocity should remain bounded by its profile")

    local orbitOrigin = Vec3f(5, 0, 0)
    local orbitConfig = {
        enabled = true,
        mode = "orbit",
        plane = "xz",
        angularSpeed = 0.5,
        direction = 1,
        phase = 0,
    }
    local orbitStart = WeaponSystem:sampleTargetMotion(orbitOrigin, 0, orbitConfig)
    assertVec3Near(orbitStart.position, Vec3f(5, 0, 0), 0.0001,
        "horizontal orbit should start on the starboard side")
    local orbitQuarter = WeaponSystem:sampleTargetMotion(
        orbitOrigin,
        math.pi / (2 * orbitConfig.angularSpeed),
        orbitConfig)
    assertVec3Near(orbitQuarter.position, Vec3f(0, 0, 5), 0.0001,
        "horizontal orbit should expose the fore side")
    assertNear(orbitQuarter.velocity.x, -2.5, 0.0001,
        "horizontal orbit velocity should be analytic")
    local orbitHalf = WeaponSystem:sampleTargetMotion(
        orbitOrigin,
        math.pi / orbitConfig.angularSpeed,
        orbitConfig)
    assertVec3Near(orbitHalf.position, Vec3f(-5, 0, 0), 0.0001,
        "horizontal orbit should expose the port side")
    local reverseOrbit = WeaponSystem:sampleTargetMotion(
        orbitOrigin,
        math.pi / (2 * orbitConfig.angularSpeed),
        {
            enabled = true,
            mode = "orbit",
            plane = "xz",
            angularSpeed = orbitConfig.angularSpeed,
            direction = -1,
            phase = 0,
        })
    assertVec3Near(reverseOrbit.position, Vec3f(0, 0, -5), 0.0001,
        "reverse orbit should expose the aft side")
    local verticalOrbit = WeaponSystem:sampleTargetMotion(
        orbitOrigin,
        math.pi / (2 * orbitConfig.angularSpeed),
        {
            enabled = true,
            mode = "orbit",
            plane = "xy",
            angularSpeed = orbitConfig.angularSpeed,
            direction = 1,
            phase = 0,
        })
    assertVec3Near(verticalOrbit.position, Vec3f(0, 5, 0), 0.0001,
        "vertical orbit should expose the top side")
    local verticalRadius = math.sqrt(
        verticalOrbit.position.x * verticalOrbit.position.x
        + verticalOrbit.position.y * verticalOrbit.position.y
        + verticalOrbit.position.z * verticalOrbit.position.z)
    assertNear(verticalRadius, 5, 0.0001,
        "orbit modes should preserve the capital-centered radius")

    local mountIds = { "fore_port", "fore_starboard", "aft_port" }
    local volley = WeaponSystem:planFire(
        Enums.Weapon.FireMode.Volley,
        mountIds,
        { fore_port = true, fore_starboard = true, aft_port = true },
        1)
    assert(#volley.shots == 3, "volley should fire all eligible mounts together")
    assert(volley.shots[1] == "fore_port", "volley order should remain deterministic")
    assert(volley.shots[3] == "aft_port", "volley should preserve configured order")

    local blockedVolley = WeaponSystem:planFire(
        Enums.Weapon.FireMode.Volley,
        mountIds,
        { fore_port = true, fore_starboard = false, aft_port = true },
        1)
    assert(#blockedVolley.shots == 2, "volley should fire every ready unobstructed mount")
    assert(blockedVolley.shots[1] == "fore_port" and blockedVolley.shots[2] == "aft_port",
        "volley should exclude only mounts that are not ready")

    local sequence = WeaponSystem:planFire(
        Enums.Weapon.FireMode.Sequence,
        mountIds,
        { fore_port = true, fore_starboard = true, aft_port = true },
        2)
    assert(#sequence.shots == 1, "sequence should fire one mount per step")
    assert(sequence.shots[1] == "fore_starboard", "sequence should use its configured index")
    assert(sequence.nextIndex == 3, "sequence should advance to the next configured mount")

    local blockedSequence = WeaponSystem:planFire(
        Enums.Weapon.FireMode.Sequence,
        mountIds,
        { fore_port = false, fore_starboard = true, aft_port = true },
        1)
    assert(#blockedSequence.shots == 1, "sequence should skip a blocked mount")
    assert(blockedSequence.shots[1] == "fore_starboard", "sequence should select the next ready mount")
    assert(blockedSequence.nextIndex == 3, "sequence should advance after skipping a blocked mount")

    local capacitorBanks = {
        { charge = 1.5, maxCharge = 2.0, chargeRate = 1.0 },
    }
    WeaponSystem:rechargeCapacitors(capacitorBanks, 0.75)
    assertNear(capacitorBanks[1].charge, 2.0, 0.0001,
        "capacitor recharge should clamp at maximum charge")
    local capacitorWeapons = {
        fore_port = {
            capacitorCost = 1.0,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Burst },
                [Enums.Weapon.FireMode.Sequence] = { id = Enums.Weapon.CapacityPolicy.Sustain },
            },
        },
        fore_starboard = {
            capacitorCost = 1.0,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Burst },
                [Enums.Weapon.FireMode.Sequence] = { id = Enums.Weapon.CapacityPolicy.Sustain },
            },
        },
        aft_port = {
            capacitorCost = 1.0,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Burst },
                [Enums.Weapon.FireMode.Sequence] = { id = Enums.Weapon.CapacityPolicy.Sustain },
            },
        },
    }
    local burst = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = { "fore_port", "fore_starboard", "aft_port" }, nextIndex = 1 },
        capacitorWeapons,
        capacitorBanks)
    assert(#burst.shots == 2, "burst capacity should admit every affordable ready shot")
    assertNear(capacitorBanks[1].charge, 0, 0.0001,
        "burst capacity should discharge accepted shot costs")
    capacitorBanks[1].charge = 1.5
    local sustain = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Sequence,
        { shots = { "fore_port", "fore_starboard" }, nextIndex = 2 },
        capacitorWeapons,
        capacitorBanks)
    assert(#sustain.shots == 1 and sustain.shots[1] == "fore_port",
        "sustain capacity should admit only one shot per sequence step")
    assertNear(capacitorBanks[1].charge, 0.5, 0.0001,
        "sustain capacity should discharge one accepted shot cost")
    capacitorBanks[1].charge = 1.0
    local firingWithExistingProjectile = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Sequence,
        { shots = { "fore_port" }, nextIndex = 1 },
        capacitorWeapons,
        capacitorBanks)
    assert(#firingWithExistingProjectile.shots == 1,
        "existing projectiles must not globally block a new capacitor-funded shot")

    local laserCapacitorGroup = Enums.Weapon.CapacitorGroup.Laser
    local plasmaCapacitorGroup = Enums.Weapon.CapacitorGroup.Plasma
    local groupedCapacitor = ConstructComponents.WeaponCapacitor {
        banks = {
            {
                groupId = laserCapacitorGroup,
                charge = 0,
                maxCharge = 0.18,
                chargeRate = 0,
            },
            {
                groupId = plasmaCapacitorGroup,
                charge = 1.0,
                maxCharge = 1.0,
                chargeRate = 1.0,
            },
        },
    }
    local groupedWeapons = {
        laser = {
            capacitorGroup = laserCapacitorGroup,
            capacitorCost = 0.18,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Burst },
            },
        },
        plasma = {
            capacitorGroup = plasmaCapacitorGroup,
            capacitorCost = 1.0,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Burst },
            },
        },
    }
    local laserCannotDrainPlasma = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = { "laser" }, nextIndex = 1 },
        groupedWeapons,
        groupedCapacitor)
    assert(#laserCannotDrainPlasma.shots == 0,
        "a depleted laser bank must not consume another weapon group's charge")
    assertNear(groupedCapacitor:getBanks()[2].charge, 1.0, 0.0001,
        "a laser denial must preserve the plasma bank")
    local plasmaShot = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = { "plasma" }, nextIndex = 1 },
        groupedWeapons,
        groupedCapacitor)
    assert(#plasmaShot.shots == 1,
        "a full plasma bank must admit a plasma shot")
    assertNear(groupedCapacitor:getBanks()[2].charge, 0, 0.0001,
        "a plasma shot must discharge only the plasma bank")
    WeaponSystem:rechargeCapacitors(groupedCapacitor, 1.0)
    local recoveredPlasmaShot = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = { "plasma" }, nextIndex = 1 },
        groupedWeapons,
        groupedCapacitor)
    assert(#recoveredPlasmaShot.shots == 1,
        "a depleted plasma bank must recharge and admit a later shot")

    local fullVolleyCapacitor = ConstructComponents.WeaponCapacitor {
        banks = {
            {
                groupId = plasmaCapacitorGroup,
                charge = 4.0,
                maxCharge = 5.0,
                chargeRate = 1.0,
            },
        },
    }
    local fullVolleyWeapons = {}
    local fullVolleyMounts = {}
    for index = 1, 5 do
        local mountId = "plasma_" .. index
        table.insert(fullVolleyMounts, mountId)
        fullVolleyWeapons[mountId] = groupedWeapons.plasma
    end
    local deniedFullVolley = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = fullVolleyMounts, nextIndex = 1 },
        fullVolleyWeapons,
        fullVolleyCapacitor)
    assert(#deniedFullVolley.shots == 0,
        "a burst must wait until the entire Plasma group volley is affordable")
    assertNear(fullVolleyCapacitor:getBanks()[1].charge, 4.0, 0.0001,
        "a denied group volley must not partially discharge its bank")
    WeaponSystem:rechargeCapacitors(fullVolleyCapacitor, 1.0)
    local fullVolley = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = fullVolleyMounts, nextIndex = 1 },
        fullVolleyWeapons,
        fullVolleyCapacitor)
    assert(#fullVolley.shots == 5,
        "a full Plasma bank must discharge the complete group volley")
    assertNear(fullVolleyCapacitor:getBanks()[1].charge, 0, 0.0001,
        "a complete group volley must empty the Plasma bank")
    WeaponSystem:rechargeCapacitors(fullVolleyCapacitor, 5.0)
    local repeatedFullVolley = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = fullVolleyMounts, nextIndex = 1 },
        fullVolleyWeapons,
        fullVolleyCapacitor)
    assert(#repeatedFullVolley.shots == 5,
        "a recharged Plasma bank must support the next complete group volley")

    local configuredCapacitor = ConstructComponents.WeaponCapacitor {
        banks = {
            { charge = 2.0, maxCharge = 2.0, chargeRate = 0 },
        },
        policies = {
            [Enums.Weapon.FireMode.Volley] = {
                id = Enums.Weapon.CapacityPolicy.Burst,
            },
            [Enums.Weapon.FireMode.Sequence] = {
                id = Enums.Weapon.CapacityPolicy.Sustain,
            },
        },
    }
    local installationPolicyWeapons = {
        fore_port = {
            capacitorCost = 1.0,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Sustain },
            },
        },
        fore_starboard = {
            capacitorCost = 1.0,
            capacityPolicy = {
                [Enums.Weapon.FireMode.Volley] = { id = Enums.Weapon.CapacityPolicy.Sustain },
            },
        },
    }
    local configuredBurst = WeaponSystem:gateFirePlan(
        Enums.Weapon.FireMode.Volley,
        { shots = { "fore_port", "fore_starboard" }, nextIndex = 1 },
        installationPolicyWeapons,
        configuredCapacitor)
    assert(#configuredBurst.shots == 2,
        "capacitor installation policy should override per-weapon capacity metadata")

    local sourceBody = {}
    local targetBodyForSight = {}
    local blockingBody = {}
    local raycastCalls = 0
    local helperWorld = {
        rayCast = function(_, ray)
            raycastCalls = raycastCalls + 1
            if raycastCalls == 1 then
                return {
                    body = sourceBody,
                    posx = 1,
                    posy = 0,
                    posz = 0,
                    normx = 1,
                    normy = 0,
                    normz = 0,
                    t = 0.1,
                }
            end
            return {
                body = targetBodyForSight,
                posx = 4,
                posy = 5,
                posz = 6,
                normx = -1,
                normy = 0,
                normz = 0,
                t = 0.5,
            }
        end,
    }
    local helperHit = RaycastHelper:castSegment(
        helperWorld,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        { ignoredBodies = { sourceBody } })
    assert(helperHit and helperHit.body == targetBodyForSight,
        "raycast helper should advance past an ignored rigid body")
    assert(raycastCalls == 2, "raycast helper should perform one follow-up cast after an ignored hit")
    assertVec3Near(helperHit.position, Vec3f(4, 5, 6), 0.0001,
        "raycast helper hit position")
    assertVec3Near(helperHit.normal, Vec3f(-1, 0, 0), 0.0001,
        "raycast helper hit normal")
    local clearSightWorld = {
        rayCast = function(_, ray)
            assertNear(ray.px, 0, 0.0000001, "sight ray should preserve exact source x")
            assertNear(ray.py, 0, 0.0000001, "sight ray should preserve exact source y")
            assertNear(ray.pz, 0, 0.0000001, "sight ray should preserve exact source z")
            assert(ray.tMin == 0 and ray.tMax == 1, "sight ray should use normalized bounds")
            return { body = targetBodyForSight }
        end,
    }
    local clearSight, clearReason = WeaponSystem:hasLineOfSight(
        clearSightWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0))
    assert(clearSight, "sight ray should see the target when it is the first hit")
    assert(clearReason == "target", "clear sight should report a target hit")
    local hitPositionSightWorld = {
        rayCast = function()
            return {
                body = targetBodyForSight,
                posx = 4,
                posy = 5,
                posz = 6,
            }
        end,
    }
    local _, _, reportedHitPosition = WeaponSystem:hasLineOfSight(
        hitPositionSightWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0))
    assertVec3Near(reportedHitPosition, Vec3f(4, 5, 6), 0.0001,
        "sight ray should preserve the native hit position")
    local blockedSightWorld = {
        rayCast = function() return { body = blockingBody } end,
    }
    assert(not WeaponSystem:hasLineOfSight(
        blockedSightWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0)), "sight ray should reject an intervening body")
    local emptySightWorld = {
        rayCast = function() return { body = nil } end,
    }
    assert(not WeaponSystem:hasLineOfSight(
        emptySightWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0)), "sight ray should reject a target it cannot hit")

    local capitalBody = {}
    function capitalBody:getPos() return Vec3f(0, 0, 0) end
    function capitalBody:getBoundingRadius() return 2 end
    local hullVisibleCalls = 0
    local hullVisibleWorld = {
        rayCast = function()
            hullVisibleCalls = hullVisibleCalls + 1
            if hullVisibleCalls == 1 then
                return {
                    body = capitalBody,
                    posx = 2,
                    posy = 0,
                    posz = 0,
                    normx = 1,
                    normy = 0,
                    normz = 0,
                    t = 0.2,
                }
            end
            return { body = targetBodyForSight, posx = 10, posy = 0, posz = 0, t = 0.8 }
        end,
    }
    local hullBlockedWorld = {
        rayCast = function()
            return {
                body = capitalBody,
                posx = -2,
                posy = 0,
                posz = 0,
                normx = -1,
                normy = 0,
                normz = 0,
                t = 0.1,
            }
        end,
    }
    local hullVisible, hullVisibleReason = WeaponSystem:hasLineOfSight(
        hullVisibleWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(1, 0, 0),
        Vec3f(10, 0, 0),
        { { body = capitalBody, kind = "hull" } })
    assert(hullVisible and hullVisibleReason == "target",
        "a ray leaving the target-facing side of the hull should remain visible")
    local hullBlocked, hullBlockedReason = WeaponSystem:hasLineOfSight(
        hullBlockedWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(-1, 0, 0),
        Vec3f(10, 0, 0),
        { { body = capitalBody, kind = "hull" } })
    assert(not hullBlocked and hullBlockedReason == "hull",
        "a ray aimed through the capital hull should be blocked")

    local downstreamObstacle = {}
    local hullThenObstacleCalls = 0
    local hullThenObstacleWorld = {
        rayCast = function()
            hullThenObstacleCalls = hullThenObstacleCalls + 1
            if hullThenObstacleCalls == 1 then
                return {
                    body = capitalBody,
                    posx = 2,
                    posy = 0,
                    posz = 0,
                    normx = 1,
                    normy = 0,
                    normz = 0,
                    t = 0.2,
                }
            end
            return {
                body = downstreamObstacle,
                posx = 4,
                posy = 0,
                posz = 0,
                normx = -1,
                normy = 0,
                normz = 0,
                t = 0.2,
            }
        end,
    }
    local hullThenObstacleVisible, hullThenObstacleReason = WeaponSystem:hasLineOfSight(
        hullThenObstacleWorld,
        sourceBody,
        targetBodyForSight,
        Vec3f(1, 0, 0),
        Vec3f(10, 0, 0),
        {
            { body = capitalBody, kind = "hull" },
            { body = downstreamObstacle, kind = "obstacle" },
        })
    assert(not hullThenObstacleVisible and hullThenObstacleReason == "obstacle",
        "a downstream obstacle after the source hull must remain blocked")
    assert(hullThenObstacleCalls == 2,
        "hull self-hit handling should recast before accepting target sight")

    local nullBody = ffi.cast("RigidBody*", nil)
    local nullSight, nullReason = WeaponSystem:hasLineOfSight(
        {
            rayCast = function() return { body = nullBody } end,
        },
        sourceBody,
        targetBodyForSight,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0))
    assert(not nullSight and nullReason == "none",
        "typed-null ray results should be treated as misses")

    local targetable = ConstructComponents.Targetable("debug")
    assert(targetable:isEnabled(), "targetable components should start enabled")
    assert(targetable:getTeam() == "debug", "targetable should preserve its team")
    local capitalTargetable = ConstructComponents.Targetable("debug", "capital")
    assert(capitalTargetable:getSizeClass() == "capital", "targetable should preserve size class")

    local turret = ConstructComponents.Turret("fore_port", Position(1, 2, 3), {
        weaponId = Enums.Weapon.Type.Plasma,
    })
    assert(turret:getMountId() == "fore_port", "turret mount ID should be stable")
    assertVec3Near(turret:getLocalPosition(), Vec3f(1, 2, 3), 0.0001, "turret local position")
    assert(turret:getWeaponId() == Enums.Weapon.Type.Plasma, "turret should preserve its explicit weapon ID")
    local missingWeaponSucceeded, missingWeaponError = pcall(function()
        ConstructComponents.Turret("missing_weapon", Position(), {})
    end)
    assert(not missingWeaponSucceeded and tostring(missingWeaponError):find("weaponId") ~= nil,
        "turret construction should reject a missing weapon ID")
    assertNear(turret:getYaw(), 0, 0.0001, "turret initial yaw")
    assertNear(turret:getPitch(), 0, 0.0001, "turret initial pitch")

    local targeting = ConstructComponents.Targeting(250)
    assert(targeting:getTarget() == nil, "targeting should start without a target")
    assertNear(targeting:getRange(), 250, 0.0001, "target acquisition range")

    local control = ConstructComponents.WeaponControl(
        Enums.Weapon.FireMode.Sequence,
        { "fore_port", "fore_starboard", "aft_port" })
    assert(control:getMode() == Enums.Weapon.FireMode.Sequence, "weapon control should preserve its mode")
    assert(control:getSequence()[2] == "fore_starboard", "weapon control should preserve sequence order")
    assert(control:getSequenceIndex() == 1, "weapon control should start at sequence index one")
    assert(not control:isTriggerHeld(), "weapon control should start idle")
    control:setActive(true)
    assert(control:isActive(), "weapon control should expose active AI state")

    local candidates = {
        { id = 9, position = Vec3f(40, 0, 0), enabled = true },
        { id = 2, position = Vec3f(40, 0, 0), enabled = true },
        { id = 1, position = Vec3f(20, 0, 0), enabled = false },
        { id = 3, position = Vec3f(300, 0, 0), enabled = true },
    }
    local selected = WeaponSystem:selectNearestTarget(Vec3f(0, 0, 0), candidates, 100)
    assert(selected and selected.id == 2, "target selection should use range then entity ID")
    assert(WeaponSystem:selectNearestTarget(Vec3f(0, 0, 0), candidates, 10) == nil,
        "target selection should return nil when no target is in range")

    local ConstructEntities = require("Modules.Constructs.Entities")
    local PhysicsComponents = require("Modules.Physics.Components")
    local CoreComponents = require("Modules.Core.Components")
    local turretEntity = ConstructEntities.Turret("test_mount", Position(1, 2, 3), {}, {
        weaponId = Enums.Weapon.Type.Plasma,
    })
    assert(turretEntity:get(PhysicsComponents.RigidBody):getRigidBody(),
        "turret entity should have a rigid body")
    assert(turretEntity:get(ConstructComponents.Turret):getMountId() == "test_mount",
        "turret entity should carry its mount component")

    local targetEntity = ConstructEntities.DebugTarget(7, {}, {
        position = Position(0, 0, 50),
        scale = 4,
        maxHealth = 250,
    })
    assert(targetEntity:get(PhysicsComponents.RigidBody):getRigidBody(),
        "debug target should have a rigid body")
    assert(targetEntity:get(ConstructComponents.Targetable):isEnabled(),
        "debug target should be targetable")
    assertNear(targetEntity:get(CoreComponents.Health):getCurrentHealth(), 250, 0.0001,
        "debug target health")

    local projectileEntity = ConstructEntities.Projectile(11, {}, {
        source = targetEntity,
        effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        position = Position(0, 0, 10),
        velocity = Vec3f(0, 0, 10),
        damage = 12,
    })
    assert(projectileEntity:get(ConstructComponents.Projectile),
        "projectile entity should carry the projectile component")

    local beamEntity = ConstructEntities.Beam(12, {}, {
        source = targetEntity,
        target = targetEntity,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 18,
        duration = 1,
        targetPoint = Vec3f(1, 2, 3),
    })
    assert(beamEntity:get(ConstructComponents.Beam),
        "beam entity should carry the beam component")
    assertVec3Near(beamEntity:get(ConstructComponents.Beam):getTargetPoint(),
        Vec3f(1, 2, 3), 0.0001, "beam should retain its sampled target point")

    local laserWeaponForBeam = WeaponRegistry:get(Enums.Weapon.Type.Laser)
    local beamSpawnState = {
        target = targetEntity,
        targetBody = targetEntity:get(PhysicsComponents.RigidBody):getRigidBody(),
        beams = {},
    }
    WeaponSystemTestbed.spawnBeam(
        beamSpawnState,
        { entity = targetEntity, body = beamSpawnState.targetBody, mountId = "beam_test" },
        { position = Vec3f(20, 21, 22), targetPoint = Vec3f(7, 8, 9),
            targetPointLocal = Vec3f(0.25, 0, 0) },
        laserWeaponForBeam,
        120)
    local spawnedBeamComponent = beamSpawnState.beams[1].component
    assertVec3Near(spawnedBeamComponent:getTargetPoint(), Vec3f(7, 8, 9), 0.0001,
        "spawned beam should use the geometry-valid target point")
    assertVec3Near(spawnedBeamComponent:getTargetPointLocal(), Vec3f(0.25, 0, 0), 0.0001,
        "spawned beam should retain the local target point for tracking")
    Registry:destroyEntity(beamSpawnState.beams[1].entity,
        Enums.Registry.EntityDestroyMode.DestroyChildren)

    local beamAimRegression = {
        effect = {
            sway = {
                amplitude = math.rad(0.5),
                frequency = 1.0,
                secondaryFrequency = 0.5,
                secondaryAmplitude = 0.5,
            },
        },
        origin = Vec3f(0, 0, 10),
        target = Vec3f(0, 0, 100),
    }
    beamAimRegression.a = BeamSystem:computeSwayedTargetPoint(
        beamAimRegression.origin,
        beamAimRegression.target,
        beamAimRegression.effect,
        0.25,
        0.3)
    beamAimRegression.aRepeat = BeamSystem:computeSwayedTargetPoint(
        beamAimRegression.origin,
        beamAimRegression.target,
        beamAimRegression.effect,
        0.25,
        0.3)
    assertVec3Near(beamAimRegression.a, beamAimRegression.aRepeat, 0.000001,
        "beam sway should be deterministic for a fixed time and phase")
    beamAimRegression.b = BeamSystem:computeSwayedTargetPoint(
        beamAimRegression.origin,
        beamAimRegression.target,
        beamAimRegression.effect,
        0.75,
        0.3)
    beamAimRegression.delta = beamAimRegression.b - beamAimRegression.a
    assert(beamAimRegression.delta.x * beamAimRegression.delta.x
        + beamAimRegression.delta.y * beamAimRegression.delta.y
        + beamAimRegression.delta.z * beamAimRegression.delta.z > 0.000001,
        "beam sway should move smoothly over time")
    beamAimRegression.offset = beamAimRegression.a - beamAimRegression.target
    beamAimRegression.offsetLength = math.sqrt(
        beamAimRegression.offset.x * beamAimRegression.offset.x
        + beamAimRegression.offset.y * beamAimRegression.offset.y
        + beamAimRegression.offset.z * beamAimRegression.offset.z)
    beamAimRegression.limit = 90 * math.tan(beamAimRegression.effect.sway.amplitude)
        * (1 + beamAimRegression.effect.sway.secondaryAmplitude) + 0.0001
    assert(beamAimRegression.offsetLength <= beamAimRegression.limit,
        "beam sway should remain within its configured angular envelope")
    local basisA = BeamAimHelper.getBasis(
        Vec3f(0, 0, 0),
        Vec3f(0.45, 0.89, 0))
    local basisB = BeamAimHelper.getBasis(
        Vec3f(0, 0, 0),
        Vec3f(0.43, 0.90, 0),
        basisA)
    local transportedRightDot = basisA.rightX * basisB.rightX
        + basisA.rightY * basisB.rightY
        + basisA.rightZ * basisB.rightZ
    assert(transportedRightDot > 0.99,
        "beam basis transport should not flip near the vertical orbit threshold")

    local beamHitRegression
    beamHitRegression = {
        sourceBody = {},
        targetBody = {},
        blockerBody = {},
        targetWorld = {
            rayCast = function()
                return { body = beamHitRegression.targetBody }
            end,
        },
        blockedWorld = {
            rayCast = function()
                return { body = beamHitRegression.blockerBody }
            end,
        },
    }
    beamHitRegression.targetHit, beamHitRegression.targetHitReason = BeamSystem:validateHit(
        beamHitRegression.targetWorld,
        beamHitRegression.sourceBody,
        beamHitRegression.targetBody,
        Vec3f(0, 0, 0),
        Vec3f(0, 0, 10))
    assert(beamHitRegression.targetHit and beamHitRegression.targetHitReason == "target",
        "beam hit validation should accept a target-first segment")
    beamHitRegression.blockedHit, beamHitRegression.blockedHitReason = BeamSystem:validateHit(
        beamHitRegression.blockedWorld,
        beamHitRegression.sourceBody,
        beamHitRegression.targetBody,
        Vec3f(0, 0, 0),
        Vec3f(0, 0, 10))
    assert(not beamHitRegression.blockedHit
        and beamHitRegression.blockedHitReason == "blocked",
        "beam hit validation should reject a blocker-first segment")

    local trackedTarget = ConstructEntities.DebugTarget(20, {}, {
        position = Position(0, 0, 90),
        scale = 4,
        maxHealth = 100,
    })
    local trackedBody = trackedTarget:get(PhysicsComponents.RigidBody):getRigidBody()
    beamHitRegression.sourceEntity = targetEntity
    beamHitRegression.sourceBody = targetEntity:get(PhysicsComponents.RigidBody):getRigidBody()
    beamHitRegression.targetEntity = trackedTarget
    beamHitRegression.targetBody = trackedBody
    beamHitRegression.blockedBeamEntity = ConstructEntities.Beam(24, {}, {
        source = beamHitRegression.sourceEntity,
        target = beamHitRegression.targetEntity,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 100,
        duration = 0.2,
        targetPoint = Vec3f(0, 0, 90),
    })
    beamHitRegression.blockedState = {
        world = beamHitRegression.blockedWorld,
        beams = {
            {
                component = beamHitRegression.blockedBeamEntity:get(ConstructComponents.Beam),
                sourceBody = beamHitRegression.sourceBody,
                targetBody = beamHitRegression.targetBody,
            },
        },
        removeBeam = function(state, index)
            table.remove(state.beams, index)
        end,
    }
    BeamSystem:update(beamHitRegression.blockedState, 0.08)
    assertNear(trackedTarget:get(CoreComponents.Health):getCurrentHealth(), 100, 0.0001,
        "a blocker-first beam segment must not apply damage")
    beamHitRegression.validBeamEntity = ConstructEntities.Beam(25, {}, {
        source = beamHitRegression.sourceEntity,
        target = beamHitRegression.targetEntity,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 100,
        duration = 0.2,
        targetPoint = Vec3f(0, 0, 90),
    })
    beamHitRegression.validState = {
        world = beamHitRegression.targetWorld,
        beams = {
            {
                component = beamHitRegression.validBeamEntity:get(ConstructComponents.Beam),
                sourceBody = beamHitRegression.sourceBody,
                targetBody = beamHitRegression.targetBody,
            },
        },
        removeBeam = function(state, index)
            table.remove(state.beams, index)
        end,
    }
    BeamSystem:update(beamHitRegression.validState, 0.08)
    assertNear(trackedTarget:get(CoreComponents.Health):getCurrentHealth(), 92, 0.0001,
        "a target-first beam segment should apply damage")

    local trackedLocalPoint = Vec3f(0.25, 0, 0)
    local trackedScale = trackedBody:getScale()
    local trackedInitialPosition = trackedBody:getPos()
    local trackedInitialPoint = Vec3f(
        trackedInitialPosition.x + trackedLocalPoint.x * trackedScale,
        trackedInitialPosition.y,
        trackedInitialPosition.z)
    local trackedBeamEntity = ConstructEntities.Beam(21, {}, {
        source = targetEntity,
        target = trackedTarget,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 1,
        duration = 1,
        targetPoint = trackedInitialPoint,
        targetPointLocal = trackedLocalPoint,
    })
    local trackedBeamComponent = trackedBeamEntity:get(ConstructComponents.Beam)
    local trackedBeamState = {
        beams = { { component = trackedBeamComponent } },
        removeBeam = function(state, index)
            table.remove(state.beams, index)
        end,
    }
    trackedBody:setPos(Position(
        trackedInitialPosition.x + 5,
        trackedInitialPosition.y,
        trackedInitialPosition.z))
    BeamSystem:update(trackedBeamState, 0.01)
    assertNear(trackedBeamComponent:getTargetPoint().x,
        trackedInitialPoint.x + 5, 0.0001,
        "beam endpoint should follow target body motion")

    local tickTarget = ConstructEntities.DebugTarget(22, {}, {
        position = Position(0, 0, 100),
        scale = 4,
        maxHealth = 100,
    })
    local tickBeamEntity = ConstructEntities.Beam(23, {}, {
        source = targetEntity,
        target = tickTarget,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 100,
        duration = 0.2,
        targetPoint = Vec3f(0, 0, 100),
    })
    local tickBeamState = {
        beams = { { component = tickBeamEntity:get(ConstructComponents.Beam) } },
        removeBeam = function(state, index)
            table.remove(state.beams, index)
        end,
    }
    BeamSystem:update(tickBeamState, 0.04)
    assertNear(tickTarget:get(CoreComponents.Health):getCurrentHealth(), 100, 0.0001,
        "beam tick cadence should defer damage before the configured interval")
    BeamSystem:update(tickBeamState, 0.04)
    assertNear(tickTarget:get(CoreComponents.Health):getCurrentHealth(), 92, 0.0001,
        "beam tick cadence should apply one interval of damage")

    local secondBeamTarget = ConstructEntities.DebugTarget(8, {}, {
        position = Position(0, 0, 60),
        scale = 4,
        maxHealth = 250,
    })
    local beamForFirstTarget = ConstructEntities.Beam(13, {}, {
        source = targetEntity,
        target = targetEntity,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 10,
        duration = 1,
        targetPoint = Vec3f(1, 2, 3),
    })
    local beamForSecondTarget = ConstructEntities.Beam(14, {}, {
        source = targetEntity,
        target = secondBeamTarget,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 10,
        duration = 1,
        targetPoint = Vec3f(4, 5, 6),
    })
    local beamState = {
        beams = {
            { component = beamForFirstTarget:get(ConstructComponents.Beam) },
            { component = beamForSecondTarget:get(ConstructComponents.Beam) },
        },
        removeBeam = function(state, index)
            table.remove(state.beams, index)
        end,
        targetHealth = targetEntity:get(CoreComponents.Health),
    }
    BeamSystem:update(beamState, 1)
    assertNear(targetEntity:get(CoreComponents.Health):getCurrentHealth(), 240, 0.0001,
        "beam system should damage the component target instead of global target state")
    assertNear(secondBeamTarget:get(CoreComponents.Health):getCurrentHealth(), 240, 0.0001,
        "beam system should damage each beam's own target")
    targetEntity:get(CoreComponents.Health):setCurrentHealth(250)

    local lethalBeamTarget = ConstructEntities.DebugTarget(9, {}, {
        position = Position(0, 0, 70),
        scale = 4,
        maxHealth = 10,
    })
    local secondLethalBeamTarget = ConstructEntities.DebugTarget(10, {}, {
        position = Position(0, 0, 80),
        scale = 4,
        maxHealth = 10,
    })
    local lethalBeamA = ConstructEntities.Beam(15, {}, {
        source = targetEntity,
        target = lethalBeamTarget,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 20,
        duration = 1,
        targetPoint = Vec3f(7, 8, 9),
    })
    local lethalBeamB = ConstructEntities.Beam(16, {}, {
        source = targetEntity,
        target = secondLethalBeamTarget,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        damagePerSecond = 20,
        duration = 1,
        targetPoint = Vec3f(10, 11, 12),
    })
    local lifecycleState = {
        beams = {
            { component = lethalBeamA:get(ConstructComponents.Beam) },
            { component = lethalBeamB:get(ConstructComponents.Beam) },
        },
        removeBeam = function(state, index)
            table.remove(state.beams, index)
        end,
        onTargetDestroyed = function(state, target)
            state.destroyedTargets = state.destroyedTargets or {}
            table.insert(state.destroyedTargets, target)
            state.beams = {}
        end,
    }
    BeamSystem:update(lifecycleState, 1)
    assert(#(lifecycleState.destroyedTargets or {}) == 2,
        "beam system should notify every distinct destroyed target after iteration")
    assert((lifecycleState.destroyedTargets[1] == lethalBeamTarget
            and lifecycleState.destroyedTargets[2] == secondLethalBeamTarget)
        or (lifecycleState.destroyedTargets[1] == secondLethalBeamTarget
            and lifecycleState.destroyedTargets[2] == lethalBeamTarget),
        "beam lifecycle callbacks should identify both destroyed targets")
    assert(#lifecycleState.beams == 0,
        "beam lifecycle callbacks should be allowed to clear all beams safely")

    local previousDeferredLightingEnabled = RenderCoreSystem.settings.deferredLighting
    local exitCleanupProbe = {
        beams = { { entity = {} } },
        projectiles = {},
        turrets = {},
        capitalBodyInWorld = false,
        previousDeferredLightingEnabled = previousDeferredLightingEnabled,
        clearBeams = function(state)
            state.beamsCleared = true
            state.beams = {}
        end,
        removeProjectile = function() end,
    }
    RenderCoreSystem:setDeferredLightingEnabled(not exitCleanupProbe.previousDeferredLightingEnabled)
    WeaponSystemTestbed.onExit(exitCleanupProbe)
    assert(exitCleanupProbe.beamsCleared,
        "testbed exit should clear active beam entities before destroying owners")
    assert(#exitCleanupProbe.beams == 0,
        "testbed exit should leave no active beam records")
    assert(RenderCoreSystem.settings.deferredLighting == previousDeferredLightingEnabled,
        "testbed exit should restore the prior deferred-lighting setting")

    local nonePointDiagnostics = WeaponSystemTestbed:collectNonePointDiagnostics({
        sightReasonByMount = {
            fore_port = "none",
            mid_starboard = "target",
        },
        targetPointByMount = {
            fore_port = { position = Vec3f(1, 2, 3) },
        },
    })
    assert(#nonePointDiagnostics == 1
        and nonePointDiagnostics[1] == "fore_port@(1.000,2.000,3.000)",
        "LOS diagnostics should report none reasons with their mount target points")

    local projectile = ConstructComponents.Projectile(
        targetEntity,
        Vec3f(0, 0, 10),
        12,
        3)
    assert(projectile:getSource() == targetEntity, "projectile should retain its source entity")
    assertVec3Near(projectile:getVelocity(), Vec3f(0, 0, 10), 0.0001, "projectile velocity")
    assertNear(projectile:getDamage(), 12, 0.0001, "projectile damage")
    assertNear(projectile:getRemainingLifetime(), 3, 0.0001, "projectile lifetime")

    local ProjectileSystem = require("Modules.Constructs.Systems.ProjectileSystem")
    local hit = ProjectileSystem:segmentSphereHit(
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        Vec3f(5, 0, 0),
        1)
    assert(hit, "projectile segment should hit an intersected target")
    assertNear(hit.t, 0.4, 0.0001, "earliest segment hit fraction")
    assertVec3Near(hit.position, Vec3f(4, 0, 0), 0.0001, "earliest segment hit position")
    assert(ProjectileSystem:segmentSphereHit(
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        Vec3f(5, 5, 0),
        1) == nil, "projectile segment should miss a distant target")

    local lethalProjectileTarget = ConstructEntities.DebugTarget(17, {}, {
        position = Position(0, 0, 80),
        scale = 4,
        maxHealth = 5,
    })
    local lethalProjectileA = ConstructEntities.Projectile(18, {}, {
        source = targetEntity,
        effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        position = Position(0, 0, 80),
        velocity = Vec3f(),
        damage = 10,
        lifetime = 1,
    })
    local lethalProjectileB = ConstructEntities.Projectile(19, {}, {
        source = targetEntity,
        effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        position = Position(0, 0, 80),
        velocity = Vec3f(),
        damage = 10,
        lifetime = 1,
    })
    local lethalProjectileState = {
        projectiles = {
            {
                entity = lethalProjectileA,
                component = lethalProjectileA:get(ConstructComponents.Projectile),
                body = lethalProjectileA:get(PhysicsComponents.RigidBody):getRigidBody(),
            },
            {
                entity = lethalProjectileB,
                component = lethalProjectileB:get(ConstructComponents.Projectile),
                body = lethalProjectileB:get(PhysicsComponents.RigidBody):getRigidBody(),
            },
        },
        targetHealth = lethalProjectileTarget:get(CoreComponents.Health),
        targetBody = lethalProjectileTarget:get(PhysicsComponents.RigidBody):getRigidBody(),
        targetRadius = lethalProjectileTarget:get(PhysicsComponents.RigidBody):getRadius(),
        removeProjectile = function(state, index)
            table.remove(state.projectiles, index)
        end,
        onTargetDestroyed = function(state)
            state.destroyed = true
            state.projectiles = {}
        end,
    }
    ProjectileSystem:update(lethalProjectileState, 1)
    assert(lethalProjectileState.destroyed,
        "projectile system should defer target lifecycle until update iteration completes")
    assert(#lethalProjectileState.projectiles == 0,
        "projectile lifecycle callback should be allowed to clear all projectiles safely")

    local plasmaWeapon = WeaponRegistry:get(Enums.Weapon.Type.Plasma)
    assert(plasmaWeapon, "plasma turret should be registered")
    local laserWeapon = WeaponRegistry:get(Enums.Weapon.Type.Laser)
    assert(laserWeapon and laserWeapon.id == Enums.Weapon.Type.Laser,
        "laser turret should be registered by its enum ID")
    assert(laserWeapon.effect == BeamRegistry:get(Enums.Weapon.Beam.Laser),
        "laser turret should reference the registered laser beam definition")
    assert(laserWeapon.effect.kind == Enums.Weapon.Effect.Beam,
        "laser turret should reference a beam effect")
    local laserProfiles = {
        LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Red),
        LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Green),
        LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Blue),
        LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Violet),
    }
    for _, profile in ipairs(laserProfiles) do
        assert(profile, "every built-in laser profile should be registered")
        assert(profile.wavelengthNm > 0 and profile.photonEnergyEv > 0,
            "laser profile should define positive wavelength and photon energy")
        assertNear(
            profile.wavelengthNm * profile.photonEnergyEv,
            1239.841984,
            0.01,
            "laser photon energy relationship")
        assertNear(
            profile.damagePerSecond,
            profile.baseDamagePerSecond * profile.strength,
            0.0001,
            "laser profile strength should scale beam damage")
        assert(profile.presentation and profile.presentation.bodyColor
            and profile.presentation.lightColor,
            "laser profile should define presentation and future-lighting colors")
    end
    assert(laserProfiles[1].presentation.bodyColor.r
        > laserProfiles[1].presentation.bodyColor.g,
        "red laser profile should be red-dominant")
    assert(laserProfiles[2].presentation.bodyColor.g
        > laserProfiles[2].presentation.bodyColor.r,
        "green laser profile should be green-dominant")
    assert(laserProfiles[3].presentation.bodyColor.b
        > laserProfiles[3].presentation.bodyColor.r,
        "blue laser profile should be blue-dominant")
    assert(laserProfiles[4].wavelengthNm < laserProfiles[3].wavelengthNm,
        "violet laser should have a shorter wavelength than blue")
    local greenLaser = WeaponRegistry:get(Enums.Weapon.Type.LaserGreen)
    local blueLaser = WeaponRegistry:get(Enums.Weapon.Type.LaserBlue)
    local violetLaser = WeaponRegistry:get(Enums.Weapon.Type.LaserViolet)
    assert(greenLaser and blueLaser and violetLaser,
        "all built-in laser weapon variants should be registered")
    assert(WeaponRegistry:getLaserProfile(greenLaser)
        == LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Green),
        "green laser should resolve its registry profile")
    assert(WeaponRegistry:getPresentation(blueLaser)
        == laserProfiles[3].presentation,
        "blue laser should resolve profile presentation without per-mount branching")
    assert(WeaponRegistry:getDamagePerSecond(violetLaser)
        > WeaponRegistry:getDamagePerSecond(greenLaser),
        "stronger violet profile should resolve higher beam damage")
    local configuredLaserProfiles = {}
    for _, loadout in ipairs(Config.weapons.testbed.loadout) do
        if loadout.weaponId == Enums.Weapon.Type.LaserGreen
            or loadout.weaponId == Enums.Weapon.Type.LaserBlue
            or loadout.weaponId == Enums.Weapon.Type.LaserViolet then
            configuredLaserProfiles[loadout.weaponId] = true
        end
    end
    assert(configuredLaserProfiles[Enums.Weapon.Type.LaserGreen]
        and configuredLaserProfiles[Enums.Weapon.Type.LaserBlue]
        and configuredLaserProfiles[Enums.Weapon.Type.LaserViolet],
        "testbed loadout should exercise each added laser profile")
    assert(laserWeapon.key == nil and plasmaWeapon.key == nil,
        "weapon definitions should not carry duplicate string keys")
    assert(plasmaWeapon.effect == ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        "plasma turret should reference the registered plasma projectile definition")
    assert(plasmaWeapon.effect.kind == Enums.Weapon.Effect.Projectile,
        "plasma turret should reference a projectile effect")
    assert(plasmaWeapon.effect.speed > 0 and plasmaWeapon.cooldown > 0,
        "registered plasma turret should define projectile timing")
    assert(plasmaWeapon.firePolicy.modeBySizeClass.capital == Enums.Weapon.FireMode.Volley,
        "registered AI policy should use volley mode for capital targets")
    assert(plasmaWeapon.firePolicy.modeBySizeClass.small == Enums.Weapon.FireMode.Sequence,
        "registered AI policy should use sequence mode for small targets")
    assert(Config.weapons.testbed.aiActive == true,
        "weapon testbed should enable automatic AI firing by default")
    assert(WeaponSystem:selectFireMode("capital", plasmaWeapon.firePolicy) == Enums.Weapon.FireMode.Volley,
        "AI mode selection should use the target size class")
    assert(WeaponSystem:selectFireMode("small", plasmaWeapon.firePolicy) == Enums.Weapon.FireMode.Sequence,
        "AI mode selection should use sequence for small targets")
    local yaw, pitch = WeaponSystem:directionToAngles(Vec3f(10, 0, -10))
    assertNear(yaw, -math.pi / 4, 0.0001, "turret yaw convention")
    assertNear(pitch, 0, 0.0001, "turret level pitch")
    local elevatedYaw, elevatedPitch = WeaponSystem:directionToAngles(Vec3f(0, 5, -10))
    assertNear(elevatedYaw, 0, 0.0001, "turret forward yaw")
    assertNear(elevatedPitch, math.atan2(5, 10), 0.0001, "turret elevation pitch")
    local beamDirection = Vec3f(1, 0, 0)
    local beamMatrix = Matrix.LookUp(
        Vec3f(),
        -beamDirection,
        Math.OrthoVector(beamDirection))
    assertVec3Near(
        beamMatrix:mulDir(Vec3f(0, 0, 1)),
        beamDirection,
        0.0001,
        "beam shader axis direction")

    local aimingTurret = ConstructComponents.Turret("aim_test", Position(), {
        weaponId = Enums.Weapon.Type.Plasma,
        traverseRate = math.pi,
        yawMin = -math.pi / 2,
        yawMax = math.pi / 2,
        pitchMin = -math.pi / 4,
        pitchMax = math.pi / 4,
    })
    local aimResult = WeaponSystem:aimTurret(
        aimingTurret,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 10),
        1,
        { aimTolerance = 0.01 })
    assertNear(aimingTurret:getYaw(), -math.pi / 2, 0.0001, "turret articulated yaw")
    assert(not aimResult.withinLimits and not aimResult.ready,
        "turret outside its mechanical limits must not report ready")
    local validAimResult = WeaponSystem:aimTurret(
        aimingTurret,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, -10),
        1,
        { aimTolerance = 0.01 })
    assert(validAimResult.ready, "turret should be ready after reaching a valid aim solution")
    local localFrameAim = WeaponSystem:aimTurret(
        aimingTurret,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        1,
        { aimTolerance = 0.01 },
        Quat.FromEuler(math.pi / 2, 0, 0))
    assertNear(localFrameAim.desiredYaw, -math.pi / 2, 0.0001,
        "turret aim should be solved in the mount local frame")

    local mountFrameRotation = Quat.FromLookUp(
        Vec3f(0, 0, 1),
        Vec3f(0, 1, 0))
    local starboardAimTurret = ConstructComponents.Turret("starboard_aim_test", Position(), {
        weaponId = Enums.Weapon.Type.Plasma,
        traverseRate = math.pi,
        yawMin = -math.pi,
        yawMax = math.pi,
        pitchMin = -math.pi / 4,
        pitchMax = math.pi / 4,
    })
    local starboardAim = WeaponSystem:aimTurret(
        starboardAimTurret,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        1,
        { aimTolerance = 0.01 },
        mountFrameRotation)
    local starboardRotation = mountFrameRotation
        * Quat.FromEuler(starboardAim.yaw, starboardAim.pitch, 0)
    assertVec3Near(
        starboardRotation:getForward(),
        Vec3f(1, 0, 0),
        0.0001,
        "articulated turret forward should point at a starboard target")

    local accuracyWeapon = {
        accuracy = {
            spread = math.rad(2.0),
            trackingJitter = math.rad(0.5),
        },
    }
    local accurateAimA = WeaponSystem:applyAccuracy(
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        accuracyWeapon,
        17)
    local accurateAimB = WeaponSystem:applyAccuracy(
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 0),
        accuracyWeapon,
        17)
    assertVec3Near(accurateAimA.position, accurateAimB.position, 0.0001,
        "weapon accuracy should be repeatable")
    local accuracyDx = accurateAimA.position.x - 10
    local accuracyDy = accurateAimA.position.y
    local accuracyDz = accurateAimA.position.z
    local accuracyOffset = math.sqrt(
        accuracyDx * accuracyDx + accuracyDy * accuracyDy + accuracyDz * accuracyDz)
    assert(accuracyOffset > 0.0001, "nonzero weapon accuracy should perturb the aim point")
    local accuracyBound = 10 * math.tan(math.sqrt(2)
        * (accuracyWeapon.accuracy.spread + accuracyWeapon.accuracy.trackingJitter))
    assert(accuracyOffset <= accuracyBound + 0.0001,
        "weapon accuracy should remain inside its configured angular bound")

    local targetSurface = {
        {
            p0 = Vec3f(-1, 0, -1),
            p1 = Vec3f(1, 0, -1),
            p2 = Vec3f(0, 0, 1),
            normal = Vec3f(0, 1, 0),
            area = 2,
        },
        {
            p0 = Vec3f(-1, 0, -1),
            p1 = Vec3f(0, 0, 1),
            p2 = Vec3f(-1, 0, 1),
            normal = Vec3f(0, -1, 0),
            area = 1,
        },
    }
    local targetPointA = WeaponSystem:sampleTargetPoint(targetSurface, 73, 0.0, {
        motionAmplitude = 0.08,
        motionFrequency = 0.6,
    })
    local targetPointB = WeaponSystem:sampleTargetPoint(targetSurface, 73, 0.0, {
        motionAmplitude = 0.08,
        motionFrequency = 0.6,
    })
    assertVec3Near(targetPointA.position, targetPointB.position, 0.000001,
        "target point selection should be deterministic")
    assert(targetPointA.triangleIndex >= 1 and targetPointA.triangleIndex <= #targetSurface,
        "target point should select a valid surface triangle")
    local barycentric = targetPointA.barycentric
    assert(barycentric.w0 >= 0 and barycentric.w1 >= 0 and barycentric.w2 >= 0,
        "target point barycentric weights should stay non-negative")
    assertNear(barycentric.w0 + barycentric.w1 + barycentric.w2, 1, 0.000001,
        "target point barycentric weights should sum to one")
    local targetPointMoved = WeaponSystem:sampleTargetPoint(targetSurface, 73, 0.1, {
        motionAmplitude = 0.08,
        motionFrequency = 0.6,
    })
    local pointDelta = targetPointMoved.position - targetPointA.position
    assert(pointDelta:length() < 0.2,
        "target point motion should vary smoothly rather than jump across the hull")
    local differentTargetPoint = WeaponSystem:sampleTargetPoint(targetSurface, 74, 0.0, {
        motionAmplitude = 0.08,
        motionFrequency = 0.6,
    })
    assert((differentTargetPoint.position - targetPointA.position):length() > 0.0001,
        "different deterministic target seeds should select natural target variation")
    local facingTargetPoint = WeaponSystem:sampleTargetPoint(targetSurface, 1, 0.0, {
        viewDirection = Vec3f(0, 1, 0),
        minFacingDot = 0.5,
    })
    assert(facingTargetPoint.normal.y > 0.5,
        "target point selection should prefer surface triangles facing the firing mount")

    local slowTurret = ConstructComponents.Turret("slow_aim_test", Position(), {
        weaponId = Enums.Weapon.Type.Plasma,
        traverseRate = 0.2,
        yawMin = -math.pi / 2,
        yawMax = math.pi / 2,
        pitchMin = -math.pi / 4,
        pitchMax = math.pi / 4,
    })
    local slowAim = WeaponSystem:aimTurret(
        slowTurret,
        Vec3f(0, 0, 0),
        Vec3f(10, 0, 10),
        0.5,
        { tracking = { traverseRate = 0.2, aimTolerance = 0.001 } })
    assertNear(slowTurret:getYaw(), -0.1, 0.0001,
        "tracking traverse should be rate limited")
    assert(not slowAim.ready, "rate-limited tracking should not report ready early")

    do
        local baselinePointLights = PointLightSystem:update()
    local baselinePointLightCount = #baselinePointLights
    assertPointLightShaderContract()
    local lightProjectileEntity = ConstructEntities.Projectile(8008, {}, {
        source = targetEntity,
        effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        position = Position(1, 2, 3),
        velocity = Vec3f(0, 0, 10),
        damage = 0,
        lifetime = 3,
    })
    local lightProjectileComponent = lightProjectileEntity:get(RenderingComponents.PointLight)
    assert(lightProjectileComponent and lightProjectileComponent:isEnabled(),
        "projectile entity should own an enabled point-light component")

    local lightBeamEffect = {
        kind = Enums.Weapon.Effect.Beam,
        visual = BeamRegistry:get(Enums.Weapon.Beam.Laser).visual,
        duration = 2,
    }
    local lightBeamEntity = ConstructEntities.Beam(8009, {}, {
        source = targetEntity,
        target = targetEntity,
        effect = lightBeamEffect,
        visual = lightBeamEffect.visual,
        damagePerSecond = 0,
        duration = 2,
        targetPoint = Vec3f(2, 0, 0),
    })
    local lightBeamComponent = lightBeamEntity:get(RenderingComponents.PointLight)
    assert(lightBeamComponent and lightBeamComponent:isEnabled(),
        "beam entity should own an enabled point-light component")
    lightBeamEntity:get(PhysicsComponents.Transform):setPos(Position(2, 0, 0))

    local collectedPointLights = PointLightSystem:update()
    assert(LightManager:getPointLights() == collectedPointLights,
        "point-light system should publish its frame snapshot through LightManager")
    assert(#collectedPointLights == baselinePointLightCount + 2,
        "ECS point-light collection should include projectile and beam entities")
    local projectileLight
    local beamLight
    for _, light in ipairs(collectedPointLights) do
        if light.entityId == lightProjectileEntity.id then
            projectileLight = light
        elseif light.entityId == lightBeamEntity.id then
            beamLight = light
        end
    end
    assert(projectileLight and beamLight,
        "point-light snapshot should retain source entity ownership")
    assertNear(projectileLight.pos.x, 1, 0.0001, "projectile light x")
    assertNear(projectileLight.pos.y, 2, 0.0001, "projectile light y")
    assertNear(projectileLight.pos.z, 3, 0.0001, "projectile light z")
    assertNear(projectileLight.color.z, 3.0, 0.0001, "projectile light color")
    assertNear(projectileLight.radius, 0.0, 0.0001,
        "projectile light uses unlimited inverse-square hull lighting")
    assertNear(projectileLight.intensity, 0.20, 0.0001, "projectile light intensity")
    assertNear(beamLight.pos.x, 2, 0.0001, "beam transform light endpoint x")
    assertNear(beamLight.radius, lightBeamEffect.visual.lightRadius, 0.0001,
        "beam light radius")
    assertNear(beamLight.intensity, lightBeamEffect.visual.lightIntensity, 0.0001,
        "beam light intensity")

    local beamLightState = {
        beams = {
            {
                entity = lightBeamEntity,
                component = lightBeamEntity:get(ConstructComponents.Beam),
                sourcePosition = Vec3f(0, 0, 0),
                targetBody = targetEntity:get(PhysicsComponents.RigidBody):getRigidBody(),
            },
        },
    }
    BeamSystem:update(beamLightState, 0.1)
    local endpointPointLights = PointLightSystem:update()
    local endpointBeamLight
    local beamSampleLights = {}
    for _, light in ipairs(endpointPointLights) do
        if light.entityId == lightBeamEntity.id then
            table.insert(beamSampleLights, light)
            if light.sourceIndex == light.sourceCount then
                endpointBeamLight = light
            end
        end
    end
    assert(#beamSampleLights == 1,
        "beam should publish only its intentional endpoint light; no midpoint point light")
    assert(endpointBeamLight, "beam endpoint update should retain its point light")
    assert(endpointBeamLight.pos.x < 2 and endpointBeamLight.pos.x > 1.5,
        "beam light should sit just source-side of the impact endpoint, not at midpoint")
    assert(endpointBeamLight.pos.x ~= 1,
        "beam endpoint light must not regress to the segment midpoint")

    local projectileBody = lightProjectileEntity:get(PhysicsComponents.RigidBody):getRigidBody()
    projectileBody:setPos(Position(4, 5, 6))
    local movedPointLights = PointLightSystem:update()
    local movedProjectileLight
    for _, light in ipairs(movedPointLights) do
        if light.entityId == lightProjectileEntity.id then
            movedProjectileLight = light
            break
        end
    end
    assert(movedProjectileLight, "moving projectile should retain its ECS light")
    assertNear(movedProjectileLight.pos.x, 4, 0.0001,
        "projectile light should follow rigid-body x")
    assertNear(movedProjectileLight.pos.y, 5, 0.0001,
        "projectile light should follow rigid-body y")
    assertNear(movedProjectileLight.pos.z, 6, 0.0001,
        "projectile light should follow rigid-body z")

    runImpactDissipationRegression(lightProjectileEntity, targetEntity, projectileBody)

    local firingState = {}
    local firingWeapon = WeaponRegistry:get(Enums.Weapon.Type.Plasma)
    local firingLightEntity = WeaponSystem:spawnFiringLight(
        firingState,
        {
            body = projectileBody,
            component = { aimPosition = Position(4, 5, 7) },
        },
        firingWeapon,
        8010)
    assert(firingLightEntity and firingLightEntity:isValid(),
        "weapon firing should create a transient ECS light entity")
    local firingLights = PointLightSystem:update()
    local firingLight
    for _, light in ipairs(firingLights) do
        if light.entityId == firingLightEntity.id then
            firingLight = light
            break
        end
    end
    assert(firingLight, "transient firing light should enter the frame snapshot")
    assert(firingWeapon.effect.visual.firingLight.color,
        "firing light should use a dedicated low-power source color")
    assertNear(firingLight.color.z, firingWeapon.effect.visual.firingLight.color.b, 0.0001,
        "firing light source color")
    assertNear(firingLight.intensity, firingWeapon.effect.visual.firingLight.intensity, 0.0001,
        "firing light intensity")
    assertNear(firingLight.pos.z, 6.12, 0.0001,
        "firing light should be placed just beyond the weapon muzzle")
    assertNear(firingLight.radius, firingWeapon.effect.visual.firingLight.radius, 0.0001,
        "firing light radius")
    PointLightSystem:update(0.06)
    local fadingFiringLight
    for _, light in ipairs(LightManager:getPointLights()) do
        if light.entityId == firingLightEntity.id then
            fadingFiringLight = light
            break
        end
    end
    assert(fadingFiringLight and fadingFiringLight.intensity < firingLight.intensity
        and fadingFiringLight.intensity > 0,
        "firing light should fade before its lifetime expires")
    PointLightSystem:update(0.08)
    assert(not firingLightEntity:isValid(),
        "expired firing light should be destroyed by PointLightSystem")

    local dissipatingProjectile = {
        entity = lightProjectileEntity,
        component = lightProjectileEntity:get(ConstructComponents.Projectile),
        effect = { lifeMax = 3, life = 3, vel = Vec3f(0, 0, 1) },
        lightIntensity = 0.20,
    }
    local dissipationState = {
        projectiles = { dissipatingProjectile },
        removeProjectile = function(state, index)
            table.remove(state.projectiles, index)
        end,
    }
    assert(ProjectileSystem:beginDissipation(dissipatingProjectile),
        "projectile visual should expose a production dissipation lifetime")
    ProjectileSystem:update(dissipationState, 0.175)
    assert(#dissipationState.projectiles == 1,
        "projectile should remain alive during its dissipation window")
    assert(dissipatingProjectile.effect.life < dissipatingProjectile.effect.lifeMax
        and dissipatingProjectile.effect.life > 0,
        "projectile pulse should fade during dissipation")
    local dissipatingLights = PointLightSystem:update()
    local dissipatingLight
    for _, light in ipairs(dissipatingLights) do
        if light.entityId == lightProjectileEntity.id then
            dissipatingLight = light
            break
        end
    end
    assert(dissipatingLight and dissipatingLight.intensity < 0.20
        and dissipatingLight.intensity > 0,
        "projectile point light should fade with the dissipation effect")
    ProjectileSystem:update(dissipationState, 0.175)
    assert(#dissipationState.projectiles == 0,
        "projectile should be removed after dissipation completes")

    Registry:destroyEntity(lightBeamEntity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    Registry:destroyEntity(lightProjectileEntity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    local clearedPointLights = PointLightSystem:update()
    assert(#clearedPointLights == baselinePointLightCount,
        "destroyed ECS light entities should disappear from the next frame snapshot")
    end

    local projectileEntity = ConstructEntities.Projectile(8, {}, {
        source = targetEntity,
        effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        position = Position(0, 0, 0),
        velocity = Vec3f(0, 0, 10),
        damage = 12,
        lifetime = 3,
    })
    assert(projectileEntity:get(PhysicsComponents.RigidBody):getRigidBody(),
        "projectile entity should have a rigid body")
    assert(projectileEntity:get(ConstructComponents.Projectile):getSource() == targetEntity,
        "projectile entity should carry projectile data")

    local targetHealth = targetEntity:get(CoreComponents.Health)
    local damageApplied = ProjectileSystem:applyDamage(
        projectileEntity:get(ConstructComponents.Projectile),
        targetHealth)
    assert(damageApplied, "projectile impact should apply damage")
    assertNear(targetHealth:getCurrentHealth(), 238, 0.0001, "projectile impact health")

    local laserEffect = BeamRegistry:get(Enums.Weapon.Beam.Laser)
    local plasmaEffect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma)
    for _, colorSpec in ipairs({
        { color = laserEffect.visual.bodyColor, label = "laser body color" },
        { color = laserEffect.visual.lightColor, label = "laser light color" },
        { color = plasmaEffect.visual.bodyColor, label = "plasma body color" },
        { color = plasmaEffect.visual.lightColor, label = "plasma light color" },
    }) do
        local color = colorSpec.color
        assert(color and color.r ~= nil and color.g ~= nil and color.b ~= nil,
            colorSpec.label .. " should expose shared Color RGB channels")
        assert(color.a ~= nil, colorSpec.label .. " should expose shared Color alpha")
    end
    assertNear(laserEffect.visual.bodyColor.r, 2.2, 0.0001, "laser body Color red")
    assertNear(plasmaEffect.visual.lightColor.b, 3.0, 0.0001, "plasma light Color blue")

    local lethalState = {
        targetHealth = CoreComponents.Health(10),
        onTargetDestroyed = function(state)
            state.destroyed = true
        end,
    }
    local lethalImpact = ProjectileSystem:applyImpact(
        lethalState,
        { component = projectileEntity:get(ConstructComponents.Projectile) },
        { position = Vec3f(1, 2, 3) })
    assert(lethalImpact and lethalState.destroyed,
        "lethal projectile impact should notify the target lifecycle")

    local mountSpecs = {
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
    local mountMesh = ShipCapital.Sausage(RNG.Create(20260820), Enums.ShipHulls.VeryLarge)
    mountMesh:computeNormals()
    local mountsA = HullMountDiscovery:discover(
        mountMesh,
        20260820,
        mountSpecs,
        {})
    local mountsB = HullMountDiscovery:discover(
        mountMesh,
        20260820,
        mountSpecs,
        {})
    assert(#mountsA == #mountSpecs, "hull discovery should produce every requested mount")
    assert(#mountsB == #mountsA, "hull discovery should be repeatable")
    local seenMountPositions = {}
    local mountsById = {}
    for index, spec in ipairs(mountSpecs) do
        local mountA = mountsA[index]
        local mountB = mountsB[index]
        assert(mountA.mountId == spec.mountId, "hull discovery should preserve mount order")
        assert(mountA.localPosition and mountA.surfaceNormal,
            "discovered mounts should carry local geometry data")
        assert(mountA.surfaceNormal.y > 0.35,
            "discovered hull mounts should remain on upward-facing or sloped hull surfaces")
        assert(mountA.side == spec.side,
            "paired hull discovery should preserve the requested side classification")
        assert(mountA.zoneMatch,
            "discovered hull mount should satisfy the requested zone: " .. spec.mountId)
        assert(mountA.sideMatch,
            "discovered hull mount should satisfy the requested side: " .. spec.mountId)
        if spec.side == "port" then
            assert(mountA.localPosition.x < mountMesh:getCenter().x,
                "port hull mounts should remain on negative-X side of the hull center")
        elseif spec.side == "starboard" then
            assert(mountA.localPosition.x > mountMesh:getCenter().x,
                "starboard hull mounts should remain on positive-X side of the hull center")
        end
        assertVec3Near(mountA.localPosition, mountB.localPosition, 0.0001,
            "deterministic hull mount position " .. spec.mountId)
        for _, previous in ipairs(seenMountPositions) do
            local dx = mountA.localPosition.x - previous.x
            local dy = mountA.localPosition.y - previous.y
            local dz = mountA.localPosition.z - previous.z
            assert(dx * dx + dy * dy + dz * dz > 0.0001,
                "hull discovery should not duplicate mount positions")
        end
        table.insert(seenMountPositions, mountA.localPosition)
        mountsById[mountA.mountId] = mountA
    end
    local zoneCounts = { fore = 0, mid = 0, aft = 0 }
    for _, mount in ipairs(mountsA) do
        zoneCounts[mount.zone] = zoneCounts[mount.zone] + 1
    end
    assert(#mountsA == 10, "expanded hull layout should produce ten mounts")
    assert(zoneCounts.fore == 4 and zoneCounts.mid == 2 and zoneCounts.aft == 4,
        "expanded hull layout should distribute mounts across fore, mid, and aft zones")
    for _, pair in ipairs({ "fore_outer", "fore_inner", "mid", "aft_inner", "aft_outer" }) do
        local port = mountsById[pair .. "_port"]
        local starboard = mountsById[pair .. "_starboard"]
        assertNear(port.localPosition.x + starboard.localPosition.x,
            2 * mountMesh:getCenter().x, 0.0001,
            "mirrored mount " .. pair .. " x")
        assertNear(port.localPosition.y, starboard.localPosition.y, 0.0001,
            "mirrored mount " .. pair .. " y")
        assertNear(port.localPosition.z, starboard.localPosition.z, 0.0001,
            "mirrored mount " .. pair .. " z")
        assertNear(port.surfaceNormal.x, -starboard.surfaceNormal.x, 0.0001,
            "mirrored mount " .. pair .. " normal x")
        assertNear(port.surfaceNormal.y, starboard.surfaceNormal.y, 0.0001,
            "mirrored mount " .. pair .. " normal y")
        assertNear(port.surfaceNormal.z, starboard.surfaceNormal.z, 0.0001,
            "mirrored mount " .. pair .. " normal z")
    end

    local mismatchMesh = {
        getCenter = function()
            return Vec3f(0, 0, 0)
        end,
        getBoundingRadius = function()
            return 1
        end,
        forEachTriangle = function(_, callback)
            callback(
                Vec3f(-0.1, 0, -0.7),
                Vec3f(0.1, 0, -0.7),
                Vec3f(0, 0, -0.9))
        end,
    }
    local mismatchSucceeded, mismatchError = pcall(function()
        HullMountDiscovery:discover(
            mismatchMesh,
            20260820,
            { { mountId = "fore_port_only", zone = "fore", side = "port" } },
            { minNormalDot = 0.5 })
    end)
    assert(not mismatchSucceeded,
        "unpaired hull discovery should reject candidates outside the requested zone and side: "
            .. tostring(mismatchError))

    print("[WeaponSystemTest] intercept, selection, fire planning, ECS data, entities, projectile data, collision, config, projectile entity, damage, and hull mounts: PASS")
    self.finished = true
    self:quit()
end

return WeaponSystemTest
