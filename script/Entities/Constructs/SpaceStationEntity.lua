local Entity = require("Core.ECS.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function SpaceStationEntity(seed)
    return Entity(
        "SpaceStationEntity",
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent(),
        Components.InventoryComponent(),
        Components.MarketplaceComponent()
    )
end

return SpaceStationEntity
