local PhysicalEntity = require("Modules.PhysicalEntity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local CelestialObjects = require("Modules.CelestialObjects.Components")

---@param seed integer
---@param meshes MeshWithMaterial[]
---@return Entity
return function(seed, meshes)
    return PhysicalEntity("AsteroidRingEntity", seed,
        Physics.RigidBody(),
        Rendering.Render(meshes),
        CelestialObjects.Simulation.PlanetaryRingMotion(1, 1)
    )
end
