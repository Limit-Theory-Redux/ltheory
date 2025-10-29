local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@param meshes MeshWithMaterial[]
---@return Entity
return function(meshes)
    return Entity.Create(
        "BoxEntity",
        Physics.Transform(),
        Rendering.Render(meshes),
        Physics.RigidBody()
    )
end
