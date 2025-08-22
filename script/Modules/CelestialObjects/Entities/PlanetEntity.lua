---@class PlanetEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")

    return PhysicalEntity("PlanetEntity", seed)
end
