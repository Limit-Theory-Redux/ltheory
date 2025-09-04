local PhysicalEntity = require("Modules.PhysicalEntity")
local Rendering = require("Modules.Rendering.Components")
local Materials = require("Shared.Registries.Materials")

---@class AsteroidEntity: PhysicalEntity
---@param seed integer
---@return AsteroidEntity
return function(seed)
    return PhysicalEntity("AsteroidEntity", seed,
        Rendering.Transform(),
        Rendering.Render({ Materials.Asteroid }, Enums.MeshType.Asteroid))
end
