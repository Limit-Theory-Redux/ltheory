local PhysicalEntity = require("Modules.PhysicalEntity")
local Economy = require("Modules.Economy.Components")

---@class SpaceStationEntity: PhysicalEntity
---@param seed integer
---@return SpaceStationEntity
return function(seed)
    return PhysicalEntity("SpaceStationEntity", seed,
        Economy.Inventory(),
        Economy.Marketplace())
end
