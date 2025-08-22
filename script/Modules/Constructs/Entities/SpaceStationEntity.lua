---@class SpaceStationEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")
    local Economy = require("Modules.Economy")

    return PhysicalEntity("SpaceStationEntity", seed,
        Economy.Components.Inventory(),
        Economy.Components.Marketplace())
end
