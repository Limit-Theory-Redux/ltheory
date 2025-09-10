local PhysicalEntity = require("Modules.PhysicalEntity")

---@alias StarSystemEntity PhysicalEntity
---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("StarSystemEntity", seed)
end
