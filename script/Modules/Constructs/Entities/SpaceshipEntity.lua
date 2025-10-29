local PhysicalEntity = require("Modules.PhysicalEntity")
local Economy = require("Modules.Economy.Components")

---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("SpaceshipEntity", seed,
        Economy.Inventory())
end
