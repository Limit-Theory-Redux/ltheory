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
    local effect = config.effect
    assert(effect and effect.kind == Enums.Weapon.Effect.Projectile,
        "ProjectileEntity requires a projectile effect definition")
    local visual = config.visual or effect.visual or {}

    local lifetime = config.lifetime or effect.lifetime
    local scale = config.scale or effect.scale
    local entity = PhysicalEntity(
        "ProjectileEntity",
        seed or 0,
        Physics.RigidBody(),
        Rendering.PointLight(
            visual.lightColor,
            visual.lightRadius,
            visual.lightIntensity),
        Rendering.Render(meshes or {}),
        Constructs.Projectile(
            config.source,
            config.velocity or Vec3f(),
            config.damage or 0,
            lifetime,
            config.guidance,
            config.targetBody,
            config.targetEntity,
            visual
        )
    )

    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setCollidable(false)
    rb:setPos(config.position or Position())
    rb:setScale(scale or 1)
    entity:get(Physics.RigidBody):setRigidBody(rb)

    return entity
end
