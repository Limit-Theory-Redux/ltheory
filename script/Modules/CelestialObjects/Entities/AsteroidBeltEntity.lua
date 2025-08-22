---@class AsteroidBeltEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")

    return PhysicalEntity("AsteroidBeltEntity", seed)
end
