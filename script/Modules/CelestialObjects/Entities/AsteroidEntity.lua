local Materials = require("Shared.Registries.Materials")

---@class AsteroidEntity: PhysicalEntity
---@param seed integer
return function(seed)
    local PhysicalEntity = require("Modules.PhysicalEntity")
    local Rendering = require("Modules.Rendering")

    return PhysicalEntity("AsteroidEntity", seed,
        Rendering.Components.Transform(),
        Rendering.Components.Render({ Materials.Asteroid }, Enums.MeshType.Asteroid))
end
