local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")
local Physics = require("Modules.Physics.Components")

---@class PhysicalEntity: Entity
---@param name string
---@param seed integer
---@return Entity
return function(name, seed, ...)
    return Entity(
        name,
        Core.Seed(seed),
        Physics.Transform(),
        Physics.Mass(),
        Core.Hierarchy(),
        ...
    )
end
