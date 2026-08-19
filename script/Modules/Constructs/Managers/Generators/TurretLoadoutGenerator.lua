local Registry = require("Core.ECS.Registry")
local Materials = require("Shared.Registries.Materials")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local ProceduralCatalog = require("Shared.Content.ProceduralCatalog")
local ConstructEntities = require("Modules.Constructs.Entities")

---@class TurretLoadoutGenerator
---@overload fun(): TurretLoadoutGenerator
local TurretLoadoutGenerator = Class("TurretLoadoutGenerator", function() end)

---@param parent Entity
---@param mounts table[]
---@param weapon table
---@return table[]
function TurretLoadoutGenerator:create(parent, mounts, weapon)
    assert(parent and mounts and weapon)

    local turrets = {}
    for index, mount in ipairs(mounts) do
        assert(type(mount.mountId) == "string")
        assert(type(mount.weaponId) == "number"
            or (type(mount.weaponRef) == "table" and mount.weaponRef.canonicalKey),
            "mount has no explicit weapon ID or procedural weapon ref: " .. mount.mountId)
        local mountWeapon = type(mount.weaponId) == "number"
            and WeaponRegistry:get(mount.weaponId)
            or ProceduralCatalog:resolve(mount.weaponRef)
        assert(mountWeapon, "missing weapon definition for mount " .. mount.mountId)
        local mesh = Mesh.Box(8)
        local material = Materials.DebugColor()
        local visual = WeaponRegistry:getPresentation(mountWeapon)
        if visual and visual.bodyColor and material.constShaderVars[1] then
            material.constShaderVars[1].value = {
                visual.bodyColor.r,
                visual.bodyColor.g,
                visual.bodyColor.b,
            }
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
            zoneMatch = mount.zoneMatch,
            sideMatch = mount.sideMatch,
            weaponRef = mount.weaponRef,
            trackingModuleRef = mount.trackingModuleRef,
        }
    end

    return turrets
end

return TurretLoadoutGenerator()
