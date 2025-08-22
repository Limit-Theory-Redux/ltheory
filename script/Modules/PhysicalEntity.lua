local Entity = require("Core.ECS.Entity")

---@class PhysicalEntity: Entity
---@param name string
---@param seed integer
return function(name, seed, ...)
    local Core = require("Modules.Core")
    local Physics = require("Modules.Physics")

    return Entity(
        name,
        Core.Components.Seed(seed),
        Physics.Components.Transform(),
        Physics.Components.Mass(),
        Core.Components.Hierarchy(),
        ...
    )
end
