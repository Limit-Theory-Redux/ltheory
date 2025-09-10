local PhysicalEntity = require("Modules.PhysicalEntity")

---@class SpaceshipEntity: PhysicalEntity
---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("SpaceshipEntity", seed)
end
