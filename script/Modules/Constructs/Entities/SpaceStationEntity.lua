local PhysicalEntity = require("Modules.PhysicalEntity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")
local Economy = require("Modules.Economy.Components")
local Core = require("Modules.Core.Components")

---@class StationStats
---@field maxHealth number
---@field defense ShipStatsDefense

---Create a Fighter ship entity
---@param seed integer
---@param meshes MeshWithMaterial[]
---@param stats StationStats|nil
---@return Entity
return function(seed, meshes, stats)
    return PhysicalEntity("SpaceStationEntity", seed,
        Physics.RigidBody(),
        Rendering.Render(meshes),
        Core.Health(stats and stats.maxHealth),
        Constructs.Defense(
            stats and stats.defense.maxHealth,
            stats and stats.defense.maxShield,
            stats and stats.defense.armor,
            stats and stats.defense.shieldRegen
        ),
        Economy.Inventory(),
        Economy.Marketplace())
end
