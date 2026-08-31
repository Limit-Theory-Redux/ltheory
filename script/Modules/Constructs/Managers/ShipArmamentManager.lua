local PhysicsComponents = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local HullMountDiscovery = require("Modules.Constructs.Managers.Generators.HullMountDiscovery")
local LoadoutGenerator = require("Modules.Constructs.Managers.Generators.LoadoutGenerator")
local TurretLoadoutGenerator = require("Modules.Constructs.Managers.Generators.TurretLoadoutGenerator")
local WeaponResolver = require("Shared.Content.WeaponResolver")

---@class ShipArmamentManager
---@overload fun(): ShipArmamentManager
local ShipArmamentManager = Class("ShipArmamentManager", function(self)
    self.mountDiscovery = HullMountDiscovery
    self.loadoutGenerator = LoadoutGenerator
    self.turretGenerator = TurretLoadoutGenerator
end)

---@param shipHandle table
---@param args table {seed, specifications?, options?}
---@return table mounts, table discovery
function ShipArmamentManager:discoverMounts(shipHandle, args)
    assert(type(shipHandle) == "table" and shipHandle.generatedMesh,
        "armament discovery requires a generated ship handle")
    assert(type(args) == "table" and args.seed ~= nil,
        "armament discovery requires a seed")
    local options = args.options or {}
    if options.structuralSockets == nil then
        options.structuralSockets = shipHandle.structuralSockets
    end
    local mounts = self.mountDiscovery:discover(
        shipHandle.generatedMesh,
        args.seed,
        args.specifications,
        options)
    return mounts, { fallbackUsed = false }
end

---@param mounts table[]
---@param args table {explicitLoadout?, policy?, resolver?}
---@return table LoadoutPlan
function ShipArmamentManager:planLoadout(mounts, args)
    assert(type(args) == "table", "armament loadout planning requires options")
    local resolver = args.resolver or function(entry)
        local identity = WeaponResolver:normalize(entry)
        local weapon = WeaponResolver:resolve(identity)
        return weapon, identity
    end
    local explicitLoadout = args.explicitLoadout or args.explicitByMount
    local result = self.loadoutGenerator:build(
        mounts,
        explicitLoadout,
        args.policy,
        resolver,
        { requireAll = args.requireAll ~= false })
    return result
end

local function identityForResolved(resolved)
    return {
        weaponId = resolved.weaponId,
        weaponRef = resolved.weaponRef,
    }
end

---@param shipHandle table
---@param mounts table[]
---@param plan table
---@param args table
---@return table ArmamentHandle
function ShipArmamentManager:installLoadout(shipHandle, mounts, plan, args)
    assert(type(shipHandle) == "table" and shipHandle.root,
        "armament installation requires a ship handle")
    assert(type(mounts) == "table" and type(plan) == "table",
        "armament installation requires mounts and a loadout plan")
    args = args or {}
    local bodyComponent = shipHandle.bodyComponent
        or shipHandle.root:get(PhysicsComponents.RigidBody)
    local parentBody = shipHandle.rigidBody
        or (bodyComponent and bodyComponent:getRigidBody())
    assert(bodyComponent and parentBody,
        "armament installation requires a ship rigid body")

    local placement = args.mountPlacement or {}
    -- Clearance scales with hull size; the floor is a fraction of hull
    -- radius (not an absolute unit value) so it stays proportional across
    -- ship scales.
    local clearance = math.max(
        bodyComponent:getRadius() * (placement.clearanceMultiplier or 0.04),
        bodyComponent:getRadius() * (placement.minimumClearanceFraction or 0.02))
    local localClearance = clearance / math.max(bodyComponent:getScale(), 0.0001)
    local installedMounts = {}
    local beamMountCount = 0
    local projectileMountCount = 0
    for _, mount in ipairs(mounts) do
        local resolved = plan.loadoutByMount[mount.mountId]
        if resolved then
            assert(resolved.weapon and resolved.weapon.effect,
                "loadout mount has no resolved effect: " .. mount.mountId)
            mount.weaponId = resolved.weaponId
            mount.weaponRef = resolved.weaponRef
            if args.trackingModule then
                mount.trackingModuleRef = args.trackingModule.ref
                mount.trackingModuleStats = args.trackingModule.stats
            end
            if resolved.weapon.effect.kind == Enums.Weapon.Effect.Beam then
                beamMountCount = beamMountCount + 1
            elseif resolved.weapon.effect.kind == Enums.Weapon.Effect.Projectile then
                projectileMountCount = projectileMountCount + 1
            else
                error("loadout mount has an unsupported effect kind: " .. mount.mountId)
            end
            mount.bodyLocalPosition = Position(
                mount.localPosition.x + mount.surfaceNormal.x * localClearance,
                mount.localPosition.y + mount.surfaceNormal.y * localClearance,
                mount.localPosition.z + mount.surfaceNormal.z * localClearance)
            mount.position = bodyComponent:toWorldScaled(mount.bodyLocalPosition)
            table.insert(installedMounts, mount)
        end
    end

    local firstLoadout = plan.loadout[1]
    local defaultWeapon = args.defaultWeapon or (firstLoadout and firstLoadout.weapon)
    assert(defaultWeapon, "armament installation requires a default weapon")
    local controlArgs = args.control or {}
    local controlMode = controlArgs.mode
        or (defaultWeapon.firePolicy and defaultWeapon.firePolicy.defaultMode)
    assert(controlMode, "armament default weapon has no fire mode")
    local mountIds = {}
    local installedById = {}
    for _, mount in ipairs(installedMounts) do
        table.insert(mountIds, mount.mountId)
        installedById[mount.mountId] = true
    end
    local sequence = controlArgs.sequence or mountIds
    for index, mountId in ipairs(sequence) do
        assert(installedById[mountId],
            "weapon control sequence references an uninstalled mount at index "
            .. tostring(index) .. ": " .. tostring(mountId))
    end
    assert(#sequence > 0, "weapon control requires at least one installed mount")
    local control = shipHandle.root:add(ConstructComponents.WeaponControl(
        controlMode, sequence))
    if controlArgs.active ~= nil then
        control:setActive(controlArgs.active == true)
    end

    local capacitor
    if args.capacitors ~= false then
        capacitor = shipHandle.root:add(ConstructComponents.WeaponCapacitor(args.capacitors or {}))
    end
    local weaponTrackingComponent
    if args.trackingComponent ~= false then
        weaponTrackingComponent = shipHandle.root:add(ConstructComponents.WeaponTracking(
            args.trackingComponent or {}))
    end
    local targeting
    if args.targeting ~= false then
        local targetingArgs = args.targeting or {}
        targeting = shipHandle.root:add(ConstructComponents.Targeting(
            targetingArgs.range or defaultWeapon.range))
        targeting:setAutoAcquireEnabled(targetingArgs.autoAcquireEnabled == nil
            and control:isActive()
            or targetingArgs.autoAcquireEnabled == true)
    end

    local generatedTurrets = self.turretGenerator:create(shipHandle.root, installedMounts)
    local turrets = {}
    local turretsById = {}
    for index, generatedTurret in ipairs(generatedTurrets) do
        local mount = installedMounts[index]
        local turretComponent = generatedTurret.entity:get(ConstructComponents.Turret)
        local turretBodyComponent = generatedTurret.entity:get(PhysicsComponents.RigidBody)
        local turretBody = turretBodyComponent and turretBodyComponent:getRigidBody()
        assert(turretComponent and turretBody,
            "turret generator produced an incomplete turret")
        shipHandle.scope:trackEntity(generatedTurret.entity)
        -- Turret bodies are kinematic and non-collidable; they must stay OUT
        -- of the physics world or they pollute weapon raycasts. Track them for
        -- ownership/cleanup only (matches pre-refactor world composition).
        shipHandle.scope:trackRigidBody(turretBody)
        local record = {
            mountId = generatedTurret.mountId,
            entity = generatedTurret.entity,
            component = turretComponent,
            body = turretBody,
            localPosition = generatedTurret.localPosition,
            bodyLocalPosition = generatedTurret.bodyLocalPosition,
            localRotation = generatedTurret.localRotation
                or turretComponent:getLocalRotation(),
            surfaceNormal = generatedTurret.surfaceNormal,
            pairId = generatedTurret.pairId,
            mountSizeClass = generatedTurret.mountSizeClass,
            mountRole = generatedTurret.mountRole,
            surfaceBand = generatedTurret.surfaceBand,
            arc = generatedTurret.arc,
            parentBody = parentBody,
            zoneMatch = generatedTurret.zoneMatch,
            sideMatch = generatedTurret.sideMatch,
            weaponId = mount.weaponId,
            weaponRef = mount.weaponRef,
        }
        turrets[index] = record
        turretsById[record.mountId] = record
    end

    return {
        mounts = installedMounts,
        mountIds = mountIds,
        loadout = plan.loadout,
        loadoutByMount = plan.loadoutByMount,
        rawLoadoutByMount = plan.rawLoadoutByMount,
        turrets = turrets,
        turretsById = turretsById,
        control = control,
        capacitor = capacitor,
        weaponTrackingComponent = weaponTrackingComponent,
        targeting = targeting,
        defaultWeapon = defaultWeapon,
        defaultIdentity = firstLoadout and identityForResolved(firstLoadout),
        beamMountCount = beamMountCount,
        projectileMountCount = projectileMountCount,
    }
end

return ShipArmamentManager
