local PhysicalEntity = require("Modules.PhysicalEntity")
local Rendering = require("Modules.Rendering.Components")
local Economy = require("Modules.Economy.Components")
local CelestialObjects = require("Modules.CelestialObjects.Components")
local Materials = require("Shared.Registries.Materials")

---@param seed integer
---@param item? ItemDefinition The item the rock yields; without it the rock has no Item or Quantity.
---@param itemQuantity? integer
---@return Entity
return function(seed, item, itemQuantity)
    local entity = PhysicalEntity("AsteroidEntity", seed,
        CelestialObjects.Asteroid(),
        Rendering.Render({ Materials.Asteroid }, Enums.MeshType.Asteroid))
    if item then
        entity:add(Economy.Item(item.id))
        entity:add(Economy.Quantity(itemQuantity))
    end
    return entity
end
