---@class AsteroidRingEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")

    return PhysicalEntity("AsteroidRingEntity", seed)
end
