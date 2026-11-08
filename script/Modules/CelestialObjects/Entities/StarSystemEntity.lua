local PhysicalEntity = require("Modules.PhysicalEntity")
local CelestialObjects = require("Modules.CelestialObjects.Components")

---@alias StarSystemEntity PhysicalEntity
---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("StarSystemEntity", seed,
        CelestialObjects.StarSystem()
    )
end
