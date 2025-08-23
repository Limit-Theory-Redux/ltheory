local PhysicalEntity = require("Modules.PhysicalEntity")

---@class StarSystemEntity: PhysicalEntity
---@param seed integer
return function(seed)
    return PhysicalEntity("StarSystemEntity", seed)
end
