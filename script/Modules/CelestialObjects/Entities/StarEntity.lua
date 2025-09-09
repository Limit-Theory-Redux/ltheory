local PhysicalEntity = require("Modules.PhysicalEntity")

---@class StarEntity: PhysicalEntity
---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("StarEntity", seed)
end
