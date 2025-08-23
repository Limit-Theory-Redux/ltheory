local PhysicalEntity = require("Modules.PhysicalEntity")

---@class PlanetEntity: PhysicalEntity
---@param seed integer
return function(seed)
    return PhysicalEntity("PlanetEntity", seed)
end
