local Entity = require("Core.ECS.Entity")

---@class BoxEntity: Entity
---@param material Material
return function(material)
    local Core = require("Modules.Core")
    local Physics = require("Modules.Physics")
    local Rendering = require("Modules.Rendering")

    return Entity(
        "BoxEntity",
        Physics.Components.Transform(),
        Rendering.Components.Render({ material }, Enums.MeshType.Box),
        Physics.Components.RigidBody(),
        Core.Components.Hierarchy()
    )
end
