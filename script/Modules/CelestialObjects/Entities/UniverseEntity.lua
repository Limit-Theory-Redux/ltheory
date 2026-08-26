local Entity = require("Core.ECS.Entity")
local Core = require("Modules.Core.Components")

---@param seed integer
---@return Entity
return function(seed)
    return Entity.Create(
        "UniverseEntity",
        Core.Seed(seed)
    )
end
