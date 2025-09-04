local PhysicalEntity = require("Modules.PhysicalEntity")

---@class MoonEntity: PhysicalEntity
---@param seed integer
---@return MoonEntity
return function(seed)
    return PhysicalEntity("MoonEntity", seed)
end
