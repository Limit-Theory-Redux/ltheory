---Creates one TurretEntity per installed mount.
---Resolves the mount's weapon identity through WeaponRegistry, builds the
---turret mesh/presentation for it, attaches the entity to the parent ship
---and returns the mount/turret records (position, arc, tracking refs) that
---the weapon systems tick each frame. Mounts are lifted off the hull surface
---by ShipArmamentManager before this runs.
local Registry = require("Core.ECS.Registry")
local Materials = require("Shared.Registries.Materials")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local ConstructEntities = require("Modules.Constructs.Entities")

---@class TurretLoadoutGenerator
---@overload fun(): TurretLoadoutGenerator
local TurretLoadoutGenerator = Class("TurretLoadoutGenerator", function() end)

---A box of half-extents (x, y, z) centred at (cx, cy, cz).
---@return Mesh
local function block(x, y, z, cx, cy, cz)
    local mesh = Mesh.Box(1)
    mesh:scale(x, y, z)
    mesh:translate(cx, cy, cz)
    return mesh
end

local turretMesh = nil

---The turret's body, in the space the mount scales (about -1 to 1, forward -Z, up +Y): a ring that
---sits on the hull, a housing and twin barrels reaching forward, so where a turret aims shows. One
---mesh, shared by every turret.
---@return Mesh
local function getTurretMesh()
    if turretMesh then return turretMesh end
    local mesh = block(0.95, 0.18, 0.95, 0, -0.62, 0)
    mesh:addMesh(block(0.6, 0.38, 0.7, 0, -0.06, 0.08))
    for _, side in ipairs({ -1, 1 }) do
        mesh:addMesh(block(0.09, 0.09, 0.85, side * 0.27, 0.02, -1.35))
        mesh:addMesh(block(0.14, 0.14, 0.12, side * 0.27, 0.02, -2.18))
    end
    turretMesh = mesh
    return mesh
end

---@param parent Entity
---@param mounts table[]
---@return table[]
function TurretLoadoutGenerator:create(parent, mounts)
    assert(parent and mounts)

    local turrets = {}
    for index, mount in ipairs(mounts) do
        assert(type(mount.mountId) == "string")
        assert(type(mount.weaponId) == "number"
            or (type(mount.weaponRef) == "table" and mount.weaponRef.canonicalKey),
            "mount has no explicit weapon ID or procedural weapon ref: " .. mount.mountId)
        local mountWeapon = WeaponRegistry:resolveIdentity(
            mount.weaponId,
            mount.weaponRef)
        assert(mountWeapon, "missing weapon definition for mount " .. mount.mountId)
        -- Lit like the hull it sits on.
        local mesh = getTurretMesh()
        local material = Materials.Metal:instance()
        local bodyLocalPosition = mount.bodyLocalPosition or mount.localPosition
        local turret = ConstructEntities.Turret(
            mount.mountId,
            bodyLocalPosition,
            { { mesh = mesh, material = material } },
            {
                bodyMesh = mesh,
                position = mount.position,
                localRotation = mount.localRotation,
                scale = mountWeapon.turretScale,
                weaponId = mount.weaponId,
                weaponRef = mount.weaponRef or mountWeapon.weaponRef,
                pairId = mount.pairId,
                mountSizeClass = mount.mountSizeClass,
                mountRole = mount.mountRole,
                surfaceBand = mount.surfaceBand,
                arc = mount.arc,
                yawMin = mount.yawMin,
                yawMax = mount.yawMax,
                pitchMin = mount.pitchMin,
                pitchMax = mount.pitchMax,
                traverseRate = mountWeapon.tracking.traverseRate,
                trackingModuleRef = mount.trackingModuleRef,
                trackingModuleStats = mount.trackingModuleStats,
            })

        Registry:attachEntity(parent, turret)
        turrets[index] = {
            mountId = mount.mountId,
            entity = turret,
            localPosition = mount.localPosition,
            bodyLocalPosition = bodyLocalPosition,
            localRotation = mount.localRotation,
            surfaceNormal = mount.surfaceNormal,
            pairId = mount.pairId,
            mountSizeClass = mount.mountSizeClass,
            mountRole = mount.mountRole,
            surfaceBand = mount.surfaceBand,
            arc = mount.arc,
            zoneMatch = mount.zoneMatch,
            sideMatch = mount.sideMatch,
            weaponRef = mount.weaponRef,
            weaponId = mount.weaponId,
            trackingModuleRef = mount.trackingModuleRef,
        }
    end

    return turrets
end

return TurretLoadoutGenerator()
