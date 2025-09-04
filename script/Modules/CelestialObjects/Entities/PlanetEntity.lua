local PhysicalEntity = require("Modules.PhysicalEntity")

---@class PlanetEntity: PhysicalEntity
---@param seed integer
---@return PlanetEntity
return function(seed)
    return PhysicalEntity("PlanetEntity", seed)
end
