local PhysicalEntity = require("Modules.PhysicalEntity")

---@class SpaceshipEntity: PhysicalEntity
---@param seed integer
---@return SpaceshipEntity
return function(seed)
    return PhysicalEntity("SpaceshipEntity", seed)
end
