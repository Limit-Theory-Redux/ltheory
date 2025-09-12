local PhysicalEntity = require("Modules.PhysicalEntity")
local Economy = require("Modules.Economy.Components")

---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("SpaceStationEntity", seed,
        Economy.Inventory(),
        Economy.Marketplace())
end
