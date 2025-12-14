local PhysicalEntity = require("Modules.PhysicalEntity")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local Constructs = require("Modules.Constructs.Components")
local Economy = require("Modules.Economy.Components")
local Core = require("Modules.Core.Components")

---@class ShipStats
---@field maxHealth number
---@field engine ShipStatsEngine
---@field defense ShipStatsDefense

---@class ShipStatsEngine
---@field exhaustScale number
---@field thrustPower number
---@field afterburnerPower number
---@field afterburnerFuel number
---@field fuelRegenRate number

---@class ShipStatsDefense
---@field maxHealth number
---@field maxShield number
---@field armor number
---@field shieldRegen number

---Create a Fighter ship entity
---@param seed integer
---@param meshes MeshWithMaterial[]
---@param type ShipType
---@param stats ShipStats|nil
---@return Entity
return function(seed, meshes, type, stats)
    return PhysicalEntity("SpaceshipEntity", seed,
        Physics.RigidBody(),
        Rendering.Render(meshes),
        Core.Health(stats and stats.maxHealth),
        Constructs.ShipData(type, nil, nil),
        Constructs.Engines(
            stats and stats.engine.exhaustScale,
            stats and stats.engine.thrustPower,
            stats and stats.engine.afterburnerPower,
            stats and stats.engine.afterburnerFuel,
            stats and stats.engine.fuelRegenRate
        ),
        Constructs.Defense(
            stats and stats.defense.maxHealth,
            stats and stats.defense.maxShield,
            stats and stats.defense.armor,
            stats and stats.defense.shieldRegen
        ),
        Economy.Inventory()
    )
end
