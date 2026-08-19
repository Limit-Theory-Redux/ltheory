local PhysicalEntity = require("Modules.PhysicalEntity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")

---@param seed integer
---@param meshes MeshWithMaterial[]|nil
---@param config table
---@return Entity
return function(seed, meshes, config)
    config = config or {}

    local entity = PhysicalEntity(
        "WeaponProjectileEntity",
        seed or 0,
        Physics.RigidBody(),
        Rendering.Render(meshes or {}),
        Constructs.Projectile(
            config.source,
            config.velocity or Vec3f(),
            config.damage or 0,
            config.lifetime or 0
        )
    )

    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setCollidable(false)
    rb:setPos(config.position or Position())
    rb:setScale(config.scale or 1)
    entity:get(Physics.RigidBody):setRigidBody(rb)

    return entity
end
