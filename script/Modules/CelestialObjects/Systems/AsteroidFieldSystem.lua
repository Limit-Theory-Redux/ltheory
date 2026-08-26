local Registry          = require("Core.ECS.Registry")
local Entity            = require("Core.ECS.Entity")
local CoreComponents    = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local RenderComp        = require("Modules.Rendering.Components").Render
local Materials         = require("Shared.Registries.Materials")
local CameraManager     = require("Modules.Cameras.Managers.CameraManager")
local AsteroidMeshPool  = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")

--- AsteroidFieldSystem — spawns/despawns asteroid entities based on player distance.
--- Nearby asteroids become real entities (physics, selectable, mineable).
--- Distant asteroids are batch-rendered by AsteroidBeltRenderer.
---@class AsteroidFieldSystem
local AsteroidFieldSystem = {}

local SPAWN_RADIUS    = 50000    -- Spawn entities within this distance
local DESPAWN_RADIUS  = 80000    -- Remove beyond this (wide hysteresis)
local TRIMESH_RADIUS  = 1000     -- Upgrade to trimesh collider within this distance
local MAX_SPAWNED_TOTAL = 100    -- Max concurrent spawned entities ACROSS ALL belts/rings
local MAX_SPAWN_PER_UPDATE = 5   -- Max new entities to spawn per update
local UPDATE_INTERVAL = 1.0      -- Seconds between spawn checks

--- Allow benchmarks/tests to raise the spawn caps (module-local).
---@param total number Max concurrent spawned entities
---@param perUpdate number Max new entities per update
function AsteroidFieldSystem.setSpawnCaps(total, perUpdate)
    MAX_SPAWNED_TOTAL = total or 100
    MAX_SPAWN_PER_UPDATE = perUpdate or 5
end

--- Resolve the RENDER ORIGIN of a belt/ring entity: rings attach to a
--- parent body (planet) that ORBITS, so their own transform goes stale as
--- the parent moves. The world position of the belt's asteroids is
--- (origin + baked offset), with origin = the PARENT's current transform
--- when one exists, else the entity's own transform. This must match the
--- renderer's origin resolution or spawned entities would sit at a
--- different place than the drawn rocks.
---@param beltEntity Entity
---@return number, number, number origin x, y, z
local function getRenderOrigin(beltEntity)
    local originEntity = beltEntity
    local parentCmp = beltEntity:get(CoreComponents.Parent)
    if parentCmp then
        local p = parentCmp:getParent()
        if p and p:get(PhysicsComponents.Transform) then
            originEntity = p
        end
    end
    local t = originEntity:get(PhysicsComponents.Transform)
    if t then
        local p = t:getPos()
        return p.x, p.y, p.z
    end
    return 0, 0, 0
end

local spawnedAsteroids = {}  -- [beltEntity] = { [asteroidIndex] = entity }
local timeSinceUpdate = 0
local totalSpawned = 0       -- Global count across all belts/rings

--- Collect in-range, not-yet-spawned candidates sorted by distance
--- (nearest first) so the spawn cap is spent on the rocks that are
--- actually near the player instead of whichever appear first in data.
---@param asteroids table
---@param beltPosX number
---@param beltPosY number
---@param beltPosZ number
---@param refX number
---@param refY number
---@param refZ number
---@param spawned table
---@return table candidates { idx, distSq } sorted ascending
local function collectNearCandidates(asteroids, beltPosX, beltPosY, beltPosZ, refX, refY, refZ, spawned)
    local candidates = {}
    local spawnDistSq = SPAWN_RADIUS * SPAWN_RADIUS
    for idx, a in ipairs(asteroids) do
        if spawned[idx] == nil then
            local dx = beltPosX + a.px - refX
            local dy = beltPosY + a.py - refY
            local dz = beltPosZ + a.pz - refZ
            local distSq = dx*dx + dy*dy + dz*dz
            if distSq < spawnDistSq then
                candidates[#candidates + 1] = { idx = idx, distSq = distSq }
            end
        end
    end
    table.sort(candidates, function(x, y) return x.distSq < y.distSq end)
    return candidates
end

--- Find the currently-spawned rock farthest from the reference point
--- (across all belts) so the cap can make room for closer candidates.
---@param asteroids table
---@param beltPosX number
---@param beltPosY number
---@param beltPosZ number
---@param refX number
---@param refY number
---@param refZ number
---@param spawned table
---@return integer|nil idx, number farthestDistSq
local function findFarthestSpawned(asteroids, beltPosX, beltPosY, beltPosZ, refX, refY, refZ, spawned)
    local farthestIdx, farthestDistSq = nil, -1
    for idx, entity in pairs(spawned) do
        local a = asteroids[idx]
        if a then
            local dx = beltPosX + a.px - refX
            local dy = beltPosY + a.py - refY
            local dz = beltPosZ + a.pz - refZ
            local distSq = dx*dx + dy*dy + dz*dz
            if distSq > farthestDistSq then
                farthestDistSq = distSq
                farthestIdx = idx
            end
        end
    end
    return farthestIdx, farthestDistSq
end

-- Eviction gap: only evict when the candidate is at least this much
-- closer than the farthest currently-spawned rock (avoids churn).
local EVICT_GAP_SQ = 0.25

--- Despawn a spawned asteroid entity (removes body from the world).
---@param entity Entity
---@param physicsWorld Physics|nil
local function despawnAsteroid(entity, physicsWorld)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    if rbCmp and rbCmp:getRigidBody() and physicsWorld then
        physicsWorld:removeRigidBody(rbCmp:getRigidBody())
    end
    Registry:destroyEntity(entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
end

--- Update: check distances, spawn/despawn asteroid entities
---@param dt number
---@param beltEntities table Array of belt entities with AsteroidBeltComponent
---@param physicsWorld Physics
---@param refEntity Entity|nil Entity to measure distance from (player ship).
---        Falls back to the active camera eye. A stable reference (ship,
---        not camera) prevents the map camera from despawning everything
---        when it activates and re-spawning it on return.
function AsteroidFieldSystem:update(dt, beltEntities, physicsWorld, refEntity)
    if not beltEntities or #beltEntities == 0 then return end

    timeSinceUpdate = timeSinceUpdate + dt
    if timeSinceUpdate < UPDATE_INTERVAL then return end
    timeSinceUpdate = 0

    local refX, refY, refZ
    if refEntity then
        local rbCmp = refEntity:get(PhysicsComponents.RigidBody)
        local rb = rbCmp and rbCmp:getRigidBody()
        if rb then
            local p = rb:getPos()
            refX, refY, refZ = p.x, p.y, p.z
        end
    end
    if not refX then
        local eye = CameraManager:getEye()
        if not eye then return end
        refX, refY, refZ = eye.x, eye.y, eye.z
    end

    for _, beltEntity in ipairs(beltEntities) do
        local beltCmp = beltEntity:get(CelestialComponents.AsteroidBelt)
        if not beltCmp then goto next_belt end

        local asteroids = beltCmp:getAsteroidData()
        if not spawnedAsteroids[beltEntity] then
            spawnedAsteroids[beltEntity] = {}
        end

        -- Belt/ring render origin: rings are offset from their parent
        -- planet, whose position changes as it orbits (see getRenderOrigin)
        local beltPosX, beltPosY, beltPosZ = getRenderOrigin(beltEntity)
        local spawned = spawnedAsteroids[beltEntity]

        -- Despawn distant, update positions for rings
        for idx, entity in pairs(spawned) do
            local a = asteroids[idx]
            if a then
                local dx = beltPosX + a.px - refX
                local dy = beltPosY + a.py - refY
                local dz = beltPosZ + a.pz - refZ
                local distSq = dx*dx + dy*dy + dz*dz

                if distSq > DESPAWN_RADIUS * DESPAWN_RADIUS then
                    despawnAsteroid(entity, physicsWorld)
                    spawned[idx] = nil
                    totalSpawned = totalSpawned - 1
                    a.spawned = false
                end
                -- Position updates handled by updatePositions() every frame
            end
        end

        -- Spawn nearby asteroids (rate-limited, global cap), nearest first:
        -- the cap is spent on the closest candidates, and when the cap is
        -- full the farthest spawned rock is evicted (with a hysteresis gap)
        -- so the real-entity set tracks the player's near field instead of
        -- leaving close rocks rendered as fake instanced ones.
        local candidates = collectNearCandidates(asteroids, beltPosX, beltPosY, beltPosZ, refX, refY, refZ, spawned)
        local spawnedThisUpdate = 0
        for _, cand in ipairs(candidates) do
            if spawnedThisUpdate >= MAX_SPAWN_PER_UPDATE then break end

            if totalSpawned >= MAX_SPAWNED_TOTAL then
                -- Make room: evict the farthest currently-spawned rock if
                -- the candidate is meaningfully closer (4x in distSq).
                local farthestIdx, farthestDistSq = findFarthestSpawned(
                    asteroids, beltPosX, beltPosY, beltPosZ, refX, refY, refZ, spawned)
                if farthestIdx and farthestDistSq > cand.distSq * (1.0 / EVICT_GAP_SQ) then
                    local farEntity = spawned[farthestIdx]
                    if farEntity then
                        despawnAsteroid(farEntity, physicsWorld)
                        spawned[farthestIdx] = nil
                        totalSpawned = totalSpawned - 1
                        local farA = asteroids[farthestIdx]
                        if farA then farA.spawned = false end
                    end
                else
                    break -- cap reached, no meaningful room: stop
                end
            end

            local idx = cand.idx
            local a = asteroids[idx]
            if not a then goto next_candidate end

            -- Simple static asteroid entity
            local worldX = beltPosX + a.px
            local worldY = beltPosY + a.py
            local worldZ = beltPosZ + a.pz

            local entity = Entity.Create("AsteroidEntity",
                CoreComponents.Seed(a.rotSeed),
                CoreComponents.Type("Asteroid"),
                PhysicsComponents.Transform()
            )

            local transform = entity:get(PhysicsComponents.Transform)
            transform:setPos(Position(worldX, worldY, worldZ))
            transform:setScale(a.scale)

            -- Mesh from pool + asteroid material
            local lodMesh = beltCmp:getLodMesh() or AsteroidMeshPool:getFromSeed(a.rotSeed)
            local mesh = lodMesh and lodMesh:get(0)
            if mesh then
                entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid() } }))
            end

            -- Sphere collider
            local rb = RigidBody.CreateSphere()
            rb:setKinematic(true)
            rb:setPos(Position(worldX, worldY, worldZ))
            rb:setScale(a.scale)
            local rbCmp = entity:add(PhysicsComponents.RigidBody())
            rbCmp:setRigidBody(rb)
            if physicsWorld then
                physicsWorld:addRigidBody(rb)
            end

            -- Attach to belt entity so map/label systems find it
            Registry:attachEntity(beltEntity, entity)

            spawned[idx] = entity
            a.spawned = true
            totalSpawned = totalSpawned + 1
            spawnedThisUpdate = spawnedThisUpdate + 1

            if totalSpawned <= 5 then
                Log.Info("AsteroidField: spawned entity %d at (%.0f, %.0f, %.0f) scale=%.1f",
                    idx, beltPosX + a.px, beltPosY + a.py, beltPosZ + a.pz, a.scale)
            end

            ::next_candidate::
        end

        ::next_belt::
    end
end

--- Get all currently spawned asteroid entities (for map, labels, etc.)
---@return table Array of { entity, label, pos }
--- Update all spawned asteroid positions every frame (follows parent movement)
--- Also handles trimesh collider upgrade/downgrade based on distance
function AsteroidFieldSystem:updatePositions()
    for beltEntity, spawned in pairs(spawnedAsteroids) do
        local beltCmp = beltEntity:get(CelestialComponents.AsteroidBelt)
        if not beltCmp then goto next_pos end
        local asteroids = beltCmp:getAsteroidData()
        local bx, by, bz = getRenderOrigin(beltEntity)
        for idx, entity in pairs(spawned) do
            local a = asteroids[idx]
            if a then
                local worldPos = Position(bx + a.px, by + a.py, bz + a.pz)
                local t = entity:get(PhysicsComponents.Transform)
                if t then t:setPos(worldPos) end
                local rbCmp = entity:get(PhysicsComponents.RigidBody)
                if rbCmp and rbCmp:getRigidBody() then
                    rbCmp:getRigidBody():setPos(worldPos)

                end
            end
        end
        ::next_pos::
    end
end

function AsteroidFieldSystem:getSpawnedEntities()
    local result = {}
    for beltEntity, spawned in pairs(spawnedAsteroids) do
        local beltCmp = beltEntity:get(CelestialComponents.AsteroidBelt)
        if beltCmp then
            local asteroids = beltCmp:getAsteroidData()
            local bx, by, bz = getRenderOrigin(beltEntity)
            for idx, entity in pairs(spawned) do
                local a = asteroids[idx]
                if a then
                    table.insert(result, {
                        entity = entity,
                        label = "Asteroid " .. idx,
                        pos = Position(bx + a.px, by + a.py, bz + a.pz),
                        scale = a.scale,
                    })
                end
            end
        end
    end
    return result
end

--- Spawned asteroid indices for a belt (small set: capped by MAX_SPAWNED_TOTAL).
--- The belt renderer uses this to build a flat spawned-flag array once per
--- frame instead of reading a per-asteroid table field in its hot loop.
---@param beltEntity Entity
---@return table indices Array of asteroid indices (1-based into asteroid data)
function AsteroidFieldSystem:getSpawnedIndices(beltEntity)
    local spawned = spawnedAsteroids[beltEntity]
    if not spawned then return {} end
    local result = {}
    for idx in pairs(spawned) do
        result[#result + 1] = idx
    end
    return result
end

--- Clean up all spawned entities (on regenerate, etc.)
---@param physicsWorld Physics
function AsteroidFieldSystem:cleanup(physicsWorld)
    for beltEntity, spawned in pairs(spawnedAsteroids) do
        for idx, entity in pairs(spawned) do
            local rbCmp = entity:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() and physicsWorld then
                physicsWorld:removeRigidBody(rbCmp:getRigidBody())
            end
            Registry:destroyEntity(entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
        end
    end
    spawnedAsteroids = {}
    totalSpawned = 0
end

return AsteroidFieldSystem
