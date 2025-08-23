local PhysicalEntity = require("Modules.PhysicalEntity")

---@class StarEntity: PhysicalEntity
---@param seed integer
return function(seed)
    return PhysicalEntity("StarEntity", seed)
end
