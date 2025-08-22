local Entity = require("Core.ECS.Entity")

---@class UniverseEntity: Entity
---@param seed integer
return function(seed)
    local Core = require("Modules.Core")

    return Entity(
        "UniverseEntity",
        Core.Components.Seed(seed),
        Core.Components.Hierarchy()
    )
end
