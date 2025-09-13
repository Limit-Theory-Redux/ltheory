local PhysicalEntity = require("Modules.PhysicalEntity")
local Rendering = require("Modules.Rendering.Components")
local Materials = require("Shared.Registries.Materials")

---@param seed integer
---@return Entity
return function(seed)
    return PhysicalEntity("AsteroidEntity", seed,
        Rendering.Transform(),
        Rendering.Render({ Materials.Asteroid }, Enums.MeshType.Asteroid))
end
