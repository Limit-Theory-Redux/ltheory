local PhysicalEntity = require("Modules.PhysicalEntity")

---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("PlanetEntity", seed)
end
