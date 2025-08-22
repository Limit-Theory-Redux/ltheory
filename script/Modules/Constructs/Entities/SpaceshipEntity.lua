---@class SpaceshipEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")

    return PhysicalEntity("SpaceshipEntity", seed)
end
