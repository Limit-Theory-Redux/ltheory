local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")

---@class UniverseEntity: Entity
---@param seed integer
return function(seed)
    return Entity(
        "UniverseEntity",
        Core.Seed(seed),
        Core.Hierarchy()
    )
end
