local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ShipGenerator = require("Modules.Constructs.Managers.Generators.ShipGenerator")

---@class TargetGenerator
---@overload fun(): TargetGenerator
local TargetGenerator = Class("TargetGenerator", function() end)

---@class TargetGenConfig
---@field shipType ShipType
---@field hull integer|nil Legacy generator hull index (1..6); required for Fighter/Basic types.
---@field position Position|nil
---@field scale number|nil
---@field rotation Quat|nil
---@field isKinematic boolean|nil
---@field collidable boolean|nil
---@field team string|nil
---@field sizeClass string|nil
---@field stats ShipStats|nil
---@field maxHealth number|nil
---@field maxShield number|nil
---@field armor number|nil
---@field shieldRegen number|nil

---Create a targetable generated ship and return its owned runtime handles.
---@param seed integer
---@param config TargetGenConfig|nil
---@return table {entity, body, health, shipData, radius, targetPointSeed}
function TargetGenerator:create(seed, config)
    assert(seed ~= nil, "target generation requires a seed")
    config = config or {}

    local shipType = config.shipType or Enums.ShipType.Capital
    local stats = config.stats or {
        maxHealth = config.maxHealth,
        engine = {},
        defense = {
            maxHealth = config.maxHealth,
            maxShield = config.maxShield or 0,
            armor = config.armor or 0,
            shieldRegen = config.shieldRegen or 0,
        },
    }
    local entity = ShipGenerator:create(seed, shipType, {
        hull = config.hull,
        position = config.position,
        scale = config.scale,
        rotation = config.rotation,
        isKinematic = config.isKinematic ~= false,
    }, stats)
    entity:add(ConstructComponents.Targetable(
        config.team or "debug",
        config.sizeClass or Enums.Target.SizeClass.Small))

    local maxShield = config.maxShield
        or (config.stats and config.stats.defense and config.stats.defense.maxShield)
        or 0
    local shieldRegen = config.shieldRegen
        or (config.stats and config.stats.defense and config.stats.defense.shieldRegen)
        or 0
    if maxShield > 0 then
        entity:add(ConstructComponents.Defense(
            config.maxHealth or 500,
            maxShield,
            0,
            shieldRegen))
    end

    local bodyComponent = entity:get(PhysicsComponents.RigidBody)
    local body = bodyComponent and bodyComponent:getRigidBody()
    local health = entity:get(require("Modules.Core.Components").Health)
    local shipData = entity:get(ConstructComponents.ShipData)
    assert(body and health and shipData and shipData:getGeneratedMesh(),
        "target generation produced incomplete target data")
    body:setCollidable(config.collidable ~= false)

    local pointRng = RNG.Create(seed)
    assert(pointRng, "target generation could not create point RNG")
    return {
        entity = entity,
        body = body,
        health = health,
        shipData = shipData,
        radius = bodyComponent:getRadius(),
        targetPointSeed = pointRng:getInt(0, 2147483646),
    }
end

return TargetGenerator()
