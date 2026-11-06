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
        local mesh = Mesh.Box(8)
        local material = Materials.DebugColor:instance()
        local visual = WeaponRegistry:getPresentation(mountWeapon)
        if visual and visual.bodyColor then
            local color = material:params().color
            color.x, color.y, color.z = visual.bodyColor.r, visual.bodyColor.g, visual.bodyColor.b
            material:commit()
        end
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
