local PhysicalEntity = require("Modules.PhysicalEntity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")

---@param seed integer
---@param meshes MeshWithMaterial[]|nil
---@param config table|nil
---@return Entity
return function(seed, meshes, config)
    config = config or {}

    local entity = PhysicalEntity(
        "DebugTargetEntity",
        seed or 0,
        Physics.RigidBody(),
        Rendering.Render(meshes or {}),
        Core.Health(config.maxHealth),
        Constructs.Targetable(config.team or "debug", config.sizeClass or "small")
    )

    local rb = RigidBody.CreateSphere()
    rb:setKinematic(config.isKinematic ~= false)
    rb:setPos(config.position or Position())
    rb:setScale(config.scale or 1)
    entity:get(Physics.RigidBody):setRigidBody(rb)

    return entity
end
