local Registry = require("Core.ECS.Registry")
local Materials = require("Shared.Registries.Materials")
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
        local mesh = Mesh.Box(8)
        local material = Materials.DebugColor()
        local turret = ConstructEntities.Turret(
            mount.mountId,
            mount.localPosition,
            { { mesh = mesh, material = material } },
            {
                bodyMesh = mesh,
                position = mount.position,
                scale = weapon.turretScale,
                weaponKey = mount.weaponKey or "debugPulseTurret",
                yawMin = mount.yawMin,
                yawMax = mount.yawMax,
                pitchMin = mount.pitchMin,
                pitchMax = mount.pitchMax,
                traverseRate = weapon.traverseRate,
            })

        Registry:attachEntity(parent, turret)
        turrets[index] = {
            mountId = mount.mountId,
            entity = turret,
            localPosition = mount.localPosition,
        }
    end

    return turrets
end

return TurretLoadoutGenerator()
