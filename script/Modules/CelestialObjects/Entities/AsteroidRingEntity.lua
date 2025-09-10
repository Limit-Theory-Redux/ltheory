local PhysicalEntity = require("Modules.PhysicalEntity")

---@class AsteroidRingEntity: PhysicalEntity
---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("AsteroidRingEntity", seed)
end
