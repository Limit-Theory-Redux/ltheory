local PhysicalEntity = require("Modules.PhysicalEntity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")
local Economy = require("Modules.Economy.Components")

---Create a Fighter ship entity
---@param seed integer
---@param meshes MeshWithMaterial[]
---@param type ShipType
---@return Entity
return function(seed, meshes, type)
    return PhysicalEntity("SpaceshipEntity", seed,
        Physics.RigidBody(),
        Rendering.Render(meshes),
        Constructs.ShipData(type),
        Economy.Inventory()
    )
end
