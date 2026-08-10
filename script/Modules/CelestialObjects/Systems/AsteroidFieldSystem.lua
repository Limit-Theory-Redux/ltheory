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

local spawnedAsteroids = {}  -- [beltEntity] = { [asteroidIndex] = entity }
local timeSinceUpdate = 0
local totalSpawned = 0       -- Global count across all belts/rings

--- Update: check distances, spawn/despawn asteroid entities
---@param dt number
---@param beltEntities table Array of belt entities with AsteroidBeltComponent
---@param physicsWorld Physics
function AsteroidFieldSystem:update(dt, beltEntities, physicsWorld)
    if not beltEntities or #beltEntities == 0 then return end

    timeSinceUpdate = timeSinceUpdate + dt
    if timeSinceUpdate < UPDATE_INTERVAL then return end
    timeSinceUpdate = 0

    local eye = CameraManager:getEye()
    if not eye then return end

    for _, beltEntity in ipairs(beltEntities) do
        local beltCmp = beltEntity:get(CelestialComponents.AsteroidBelt)
        if not beltCmp then goto next_belt end

        local asteroids = beltCmp:getAsteroidData()
        if not spawnedAsteroids[beltEntity] then
            spawnedAsteroids[beltEntity] = {}
        end

        -- Get belt/ring entity world position (rings are offset from their parent planet)
        local beltTransform = beltEntity:get(PhysicsComponents.Transform)
        local beltPosX = beltTransform and beltTransform:getPos().x or 0
        local beltPosY = beltTransform and beltTransform:getPos().y or 0
        local beltPosZ = beltTransform and beltTransform:getPos().z or 0
        local spawned = spawnedAsteroids[beltEntity]

        -- Despawn distant, update positions for rings
        for idx, entity in pairs(spawned) do
            local a = asteroids[idx]
            if a then
                local dx = beltPosX + a.px - eye.x
                local dy = beltPosY + a.py - eye.y
                local dz = beltPosZ + a.pz - eye.z
                local distSq = dx*dx + dy*dy + dz*dz

                if distSq > DESPAWN_RADIUS * DESPAWN_RADIUS then
                    -- Despawn
                    local rbCmp = entity:get(PhysicsComponents.RigidBody)
                    if rbCmp and rbCmp:getRigidBody() and physicsWorld then
                        physicsWorld:removeRigidBody(rbCmp:getRigidBody())
                    end
                    Registry:destroyEntity(entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
                    spawned[idx] = nil
                    totalSpawned = totalSpawned - 1
                    a.spawned = false
                end
                -- Position updates handled by updatePositions() every frame
            end
        end

        -- Spawn nearby asteroids (rate-limited, global cap)
        local spawnedThisUpdate = 0
        for idx, a in ipairs(asteroids) do
            if totalSpawned >= MAX_SPAWNED_TOTAL then break end
            if spawnedThisUpdate >= MAX_SPAWN_PER_UPDATE then break end
            if spawned[idx] then goto next_asteroid end

            local dx = beltPosX + a.px - eye.x
            local dy = beltPosY + a.py - eye.y
            local dz = beltPosZ + a.pz - eye.z
            local distSq = dx*dx + dy*dy + dz*dz

            if distSq < SPAWN_RADIUS * SPAWN_RADIUS then
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
            end

            ::next_asteroid::
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
        local bt = beltEntity:get(PhysicsComponents.Transform)
        if not bt then goto next_pos end
        local bp = bt:getPos()
        local bx, by, bz = bp.x, bp.y, bp.z
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
            local bt = beltEntity:get(PhysicsComponents.Transform)
            local bx = bt and bt:getPos().x or 0
            local by = bt and bt:getPos().y or 0
            local bz = bt and bt:getPos().z or 0
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
