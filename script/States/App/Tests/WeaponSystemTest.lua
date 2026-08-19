local Application = require("States.Application")
local ffi = require("ffi")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local ShipWeaponRegistry = require("Shared.Registries.ShipWeaponRegistry")

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

    local mountIds = { "fore_port", "fore_starboard", "aft_port" }
    local volley = WeaponSystem:planFire(
        "volley",
        mountIds,
        { fore_port = true, fore_starboard = true, aft_port = true },
        1)
    assert(#volley.shots == 3, "volley should fire all eligible mounts together")
    assert(volley.shots[1] == "fore_port", "volley order should remain deterministic")
    assert(volley.shots[3] == "aft_port", "volley should preserve configured order")

    local blockedVolley = WeaponSystem:planFire(
        "volley",
        mountIds,
        { fore_port = true, fore_starboard = false, aft_port = true },
        1)
    assert(#blockedVolley.shots == 2, "volley should fire every ready unobstructed mount")
    assert(blockedVolley.shots[1] == "fore_port" and blockedVolley.shots[2] == "aft_port",
        "volley should exclude only mounts that are not ready")

    local sequence = WeaponSystem:planFire(
        "sequence",
        mountIds,
        { fore_port = true, fore_starboard = true, aft_port = true },
        2)
    assert(#sequence.shots == 1, "sequence should fire one mount per step")
    assert(sequence.shots[1] == "fore_starboard", "sequence should use its configured index")
    assert(sequence.nextIndex == 3, "sequence should advance to the next configured mount")

    local blockedSequence = WeaponSystem:planFire(
        "sequence",
        mountIds,
        { fore_port = false, fore_starboard = true, aft_port = true },
        1)
    assert(#blockedSequence.shots == 1, "sequence should skip a blocked mount")
    assert(blockedSequence.shots[1] == "fore_starboard", "sequence should select the next ready mount")
    assert(blockedSequence.nextIndex == 3, "sequence should advance after skipping a blocked mount")

    local sourceBody = {}
    local targetBodyForSight = {}
    local blockingBody = {}
    local clearSightWorld = {
        rayCast = function(_, ray)
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
    local hullBlockedWorld = {
        rayCast = function() return { body = capitalBody } end,
    }
    local hullVisible, hullVisibleReason = WeaponSystem:hasLineOfSight(
        hullBlockedWorld,
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

    local ConstructComponents = require("Modules.Constructs.Components")
    local targetable = ConstructComponents.Targetable("debug")
    assert(targetable:isEnabled(), "targetable components should start enabled")
    assert(targetable:getTeam() == "debug", "targetable should preserve its team")
    local capitalTargetable = ConstructComponents.Targetable("debug", "capital")
    assert(capitalTargetable:getSizeClass() == "capital", "targetable should preserve size class")

    local turret = ConstructComponents.Turret("fore_port", Position(1, 2, 3))
    assert(turret:getMountId() == "fore_port", "turret mount ID should be stable")
    assertVec3Near(turret:getLocalPosition(), Vec3f(1, 2, 3), 0.0001, "turret local position")
    assert(turret:getWeaponKey() == "debugPulseTurret", "turret should use the debug weapon by default")
    assertNear(turret:getYaw(), 0, 0.0001, "turret initial yaw")
    assertNear(turret:getPitch(), 0, 0.0001, "turret initial pitch")

    local targeting = ConstructComponents.Targeting(250)
    assert(targeting:getTarget() == nil, "targeting should start without a target")
    assertNear(targeting:getRange(), 250, 0.0001, "target acquisition range")

    local control = ConstructComponents.WeaponControl(
        "sequence",
        { "fore_port", "fore_starboard", "aft_port" })
    assert(control:getMode() == "sequence", "weapon control should preserve its mode")
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
    local turretEntity = ConstructEntities.Turret("test_mount", Position(1, 2, 3), {})
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

    local debugWeapon = ShipWeaponRegistry:get("debugPulseTurret")
    assert(debugWeapon, "debug pulse turret should be registered")
    assert(debugWeapon.projectileSpeed > 0 and debugWeapon.cooldown > 0,
        "registered debug pulse turret should define projectile timing")
    assert(debugWeapon.ai.modeBySizeClass.capital == "volley",
        "registered AI policy should use volley mode for capital targets")
    assert(debugWeapon.ai.modeBySizeClass.small == "sequence",
        "registered AI policy should use sequence mode for small targets")
    assert(WeaponSystem:selectFireMode("capital", debugWeapon.ai) == "volley",
        "AI mode selection should use the target size class")
    assert(WeaponSystem:selectFireMode("small", debugWeapon.ai) == "sequence",
        "AI mode selection should use sequence for small targets")
    local yaw, pitch = WeaponSystem:directionToAngles(Vec3f(10, 0, 10))
    assertNear(yaw, math.pi / 4, 0.0001, "turret yaw convention")
    assertNear(pitch, 0, 0.0001, "turret level pitch")
    local elevatedYaw, elevatedPitch = WeaponSystem:directionToAngles(Vec3f(0, 5, 10))
    assertNear(elevatedYaw, 0, 0.0001, "turret forward yaw")
    assertNear(elevatedPitch, math.atan2(5, 10), 0.0001, "turret elevation pitch")

    local aimingTurret = ConstructComponents.Turret("aim_test", Position(), {
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
    assertNear(aimingTurret:getYaw(), math.pi / 4, 0.0001, "turret articulated yaw")
    assert(aimResult.ready, "turret should be ready after reaching its aim solution")

    local projectileEntity = ConstructEntities.Projectile(8, {}, {
        source = targetEntity,
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

    print("[WeaponSystemTest] intercept, selection, fire planning, ECS data, entities, projectile data, collision, config, projectile entity, and damage: PASS")
    self.finished = true
    self:quit()
end

return WeaponSystemTest
