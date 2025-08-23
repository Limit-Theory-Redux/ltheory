local PhysicalEntity = require("Modules.PhysicalEntity")

---@class SpaceshipEntity: PhysicalEntity
---@param seed integer
return function(seed)
    return PhysicalEntity("SpaceshipEntity", seed)
end
