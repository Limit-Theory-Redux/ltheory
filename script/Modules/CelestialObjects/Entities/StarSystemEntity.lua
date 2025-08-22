---@class StarSystemEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")

    return PhysicalEntity("StarSystemEntity", seed)
end
