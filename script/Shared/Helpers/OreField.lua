-- Ore fields: rocks that yield an item for miners (`YieldComponent`), as `Asteroid` entities with a
-- mesh from the asteroid pool and a kinematic sphere body. Sizes are in game units.

local PhysicsComponents = require("Modules.Physics.Components")
local CelestialEntities = require("Modules.CelestialObjects.Entities")
local RenderComp        = require("Modules.Rendering.Components").Render
local Materials         = require("Shared.Registries.Materials")
local AsteroidMeshPool  = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local EconomySetup      = require("Modules.Economy.Managers.EconomySetup")

local OreField = {}

-- The pool the rocks' meshes come from: the size and seed the game's other rocks use.
local MESH_POOL_SIZE, MESH_POOL_SEED = 8, 42

---@class OreFieldSpec
---@field centre Position
---@field halfExtent gameunit Half the field's width; its height is `flatten` of that
---@field flatten number|nil The field's height over its width, 0.25 when nil
---@field count integer
---@field minRadius gameunit
---@field maxRadius gameunit
---@field gap gameunit The least space between two rocks' surfaces
---@field item ItemDefinition
---@field quantity integer What each rock holds, and grows back to
---@field regrowPerSecond number|nil

---A rock that yields `item`, with its body in `world`.
---@param world Physics
---@param seed integer
---@param position Position
---@param radius gameunit
---@param item ItemDefinition
---@param quantity integer
---@param regrowPerSecond number|nil
---@return Entity
function OreField.createRock(world, seed, position, radius, item, quantity, regrowPerSecond)
    AsteroidMeshPool:init(MESH_POOL_SIZE, MESH_POOL_SEED)
    local entity = CelestialEntities.Asteroid(seed)
    local transform = entity:get(PhysicsComponents.Transform)
    transform:setPos(position)
    transform:setScale(radius)
    -- The constructor's render component holds no mesh; it is removed before the one with a mesh is added.
    entity:remove(RenderComp)
    local mesh = AsteroidMeshPool:getFromSeed(seed):get(0)
    entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))

    -- The hull of the drawn mesh, so that rays (a mining laser) and contacts meet the surface seen.
    local rb = RigidBody.CreateConvexHullFromMesh(mesh)
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    world:addRigidBody(rb)
    EconomySetup.equipAsteroid(entity, item, quantity, regrowPerSecond, seed)
    return entity
end

---Rocks spread through a flat box around the centre, not touching, with their bodies in `world`.
---@param world Physics
---@param rng RNG
---@param spec OreFieldSpec
---@return Entity[] rocks Up to `count`, fewer when the box has no room for more
function OreField.build(world, rng, spec)
    local flatten = spec.flatten or 0.25
    local rocks, placed = {}, {}
    for _ = 1, spec.count * 50 do
        if #placed >= spec.count then break end
        local r = rng:getUniformRange(spec.minRadius, spec.maxRadius)
        local p = {
            x = rng:getUniformRange(-1, 1) * spec.halfExtent,
            y = rng:getUniformRange(-flatten, flatten) * spec.halfExtent,
            z = rng:getUniformRange(-1, 1) * spec.halfExtent,
            r = r,
        }
        local free = true
        for _, q in ipairs(placed) do
            local dx, dy, dz = p.x - q.x, p.y - q.y, p.z - q.z
            if math.sqrt(dx * dx + dy * dy + dz * dz) < p.r + q.r + spec.gap then
                free = false
                break
            end
        end
        if free then
            placed[#placed + 1] = p
            local c = spec.centre
            rocks[#rocks + 1] = OreField.createRock(world, rng:get31(), Position(c.x + p.x, c.y + p.y, c.z + p.z), r,
                spec.item, spec.quantity, spec.regrowPerSecond)
        end
    end
    return rocks
end

return OreField
