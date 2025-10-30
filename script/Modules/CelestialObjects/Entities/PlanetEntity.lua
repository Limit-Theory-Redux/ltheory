local PhysicalEntity = require("Modules.PhysicalEntity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@param seed integer
---@param meshes MeshWithMaterial[]
---@return Entity
return function(seed, meshes)
    return PhysicalEntity("PlanetEntity", seed,
        Physics.RigidBody(),
        Rendering.Render(meshes))
end
