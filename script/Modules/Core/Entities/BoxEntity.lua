local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@class BoxEntity: Entity
---@param material Material
---@return Entity
return function(material)
    return Entity(
        "BoxEntity",
        Physics.Transform(),
        Rendering.Render({ material }, Enums.MeshType.Box),
        Physics.RigidBody(),
        Core.Hierarchy()
    )
end
