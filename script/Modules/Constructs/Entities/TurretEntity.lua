local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")

---@param mountId string
---@param localPosition Position
---@param meshes MeshWithMaterial[]|nil
---@param config table|nil
---@return Entity
return function(mountId, localPosition, meshes, config)
    config = config or {}

    local entity = Entity.Create(
        "TurretEntity",
        Physics.Transform(),
        Physics.RigidBody(),
        Rendering.Render(meshes or {}),
        Constructs.Turret(mountId, localPosition, config)
    )

    local rb = RigidBody.CreateBox()
    rb:setKinematic(true)
    rb:setCollidable(false)
    rb:setPos(config.position or localPosition or Position())
    rb:setScale(config.scale or 1)
    entity:get(Physics.RigidBody):setRigidBody(rb)

    return entity
end
