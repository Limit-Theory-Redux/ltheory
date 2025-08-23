local PhysicalEntity = require("Modules.PhysicalEntity")

---@class AsteroidBeltEntity: PhysicalEntity
---@param seed integer
return function(seed)
    return PhysicalEntity("AsteroidBeltEntity", seed)
end
