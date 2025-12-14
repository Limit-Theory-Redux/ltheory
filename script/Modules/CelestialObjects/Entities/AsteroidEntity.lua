local PhysicalEntity = require("Modules.PhysicalEntity")
local Rendering = require("Modules.Rendering.Components")
local Economy = require("Modules.Economy.Components")
local Materials = require("Shared.Registries.Materials")

---@param seed integer
---@param item ItemDefinition
---@return Entity
return function(seed, item, itemQuantity)
    return PhysicalEntity("AsteroidEntity", seed,
        Rendering.Render({ Materials.Asteroid }, Enums.MeshType.Asteroid),
        Economy.Item(item.id),
        Economy.Quantity(itemQuantity))
end
