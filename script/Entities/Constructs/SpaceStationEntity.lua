local Entity = require("Entities.Entity")
local Components = require("Components")

---@param seed integer
---@return Entity
local function SpaceStationEntity(seed)
    return Entity(
        Components.NameComponent(),
        Components.SeedComponent(seed),
        Components.TransformComponent(),
        Components.MassComponent(),
        Components.HierarchyComponent(),
        Components.InventoryComponent(),
        Components.MarketplaceComponent()
    )
end

return SpaceStationEntity
