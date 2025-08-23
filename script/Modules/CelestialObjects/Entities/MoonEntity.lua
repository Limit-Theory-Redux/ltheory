local PhysicalEntity = require("Modules.PhysicalEntity")

---@class MoonEntity: PhysicalEntity
---@param seed integer
return function(seed)
    return PhysicalEntity("MoonEntity", seed)
end
