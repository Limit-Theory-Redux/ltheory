local PhysicalEntity = require("Modules.PhysicalEntity")
local CelestialObjects = require("Modules.CelestialObjects.Components")

---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("AsteroidBeltEntity", seed,
        CelestialObjects.AsteroidBelt()
    )
end
