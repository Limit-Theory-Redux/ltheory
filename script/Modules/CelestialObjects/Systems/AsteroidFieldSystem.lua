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

local SPAWN_RADIUS    = 200000   -- Spawn entities within this distance
local DESPAWN_RADIUS  = 300000   -- Remove entities beyond this distance
local TRIMESH_RADIUS  = 1000     -- Upgrade to trimesh collider within this distance
local MAX_SPAWNED_TOTAL = 200    -- Max concurrent spawned entities ACROSS ALL belts/rings
local MAX_SPAWN_PER_UPDATE = 10  -- Max new entities to spawn per update (spread over frames)
local UPDATE_INTERVAL = 0.5      -- Seconds between spawn checks

local spawnedAsteroids = {}  -- [beltEntity] = { [asteroidIndex] = entity }
local rotationCache = {}     -- [beltEntity] = { [asteroidIndex] = Quat }
local timeSinceUpdate = 0
local totalSpawned = 0       -- Global count across all belts/rings

--- Get or compute rotation for an asteroid
local function getRotation(beltEntity, idx, rotSeed)
    if not rotationCache[beltEntity] then
        rotationCache[beltEntity] = {}
    end
    local cache = rotationCache[beltEntity]
    if not cache[idx] then
        local rng = RNG.Create(rotSeed)
        local ax = rng:getUniform() - 0.5
        local ay = rng:getUniform() - 0.5
        local az = rng:getUniform() - 0.5
        local len = math.sqrt(ax*ax + ay*ay + az*az)
        if len > 0.001 then ax, ay, az = ax/len, ay/len, az/len end
        local angle = rng:getUniform() * math.pi * 2
        cache[idx] = Quat.FromAxisAngle(Vec3f(ax, ay, az), angle)
    end
    return cache[idx]
end

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

        -- Despawn distant asteroids + upgrade/downgrade colliders
        local lodMesh = beltCmp:getLodMesh()
        local highDetailMesh = lodMesh and lodMesh:get(0)
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
                elseif highDetailMesh then
                    -- Upgrade sphere → trimesh when very close
                    local rbCmp = entity:get(PhysicsComponents.RigidBody)
                    if rbCmp and rbCmp:getRigidBody() then
                        local isTrimesh = a._trimesh
                        if distSq < TRIMESH_RADIUS * TRIMESH_RADIUS and not isTrimesh then
                            physicsWorld:removeRigidBody(rbCmp:getRigidBody())
                            local rb = RigidBody.CreateTrimeshFromMesh(highDetailMesh)
                            rb:setKinematic(true)
                            rb:setPos(Position(beltPosX + a.px, beltPosY + a.py, beltPosZ + a.pz))
                            rb:setScale(a.scale)
                            rbCmp:setRigidBody(rb)
                            physicsWorld:addRigidBody(rb)
                            a._trimesh = true
                        elseif distSq > TRIMESH_RADIUS * TRIMESH_RADIUS * 4 and isTrimesh then
                            -- Downgrade back to sphere when moving away
                            physicsWorld:removeRigidBody(rbCmp:getRigidBody())
                            local rb = RigidBody.CreateSphere()
                            rb:setKinematic(true)
                            rb:setPos(Position(beltPosX + a.px, beltPosY + a.py, beltPosZ + a.pz))
                            rb:setScale(a.scale)
                            rbCmp:setRigidBody(rb)
                            physicsWorld:addRigidBody(rb)
                            a._trimesh = false
                        end
                    end
                end
            end
        end

        -- Spawn nearby asteroids (rate-limited, global cap)
        local spawnedThisUpdate = 0
        for idx, a in ipairs(asteroids) do
            if totalSpawned >= MAX_SPAWNED_TOTAL then break end
            if spawnedThisUpdate >= MAX_SPAWN_PER_UPDATE then break end
            if spawned[idx] then goto next_asteroid end

            local dx = a.px - eye.x
            local dy = a.py - eye.y
            local dz = a.pz - eye.z
            local distSq = dx*dx + dy*dy + dz*dz

            if distSq < SPAWN_RADIUS * SPAWN_RADIUS then
                -- Create named entity with proper components
                local entity = Entity.Create("AsteroidEntity",
                    CoreComponents.Seed(a.rotSeed),
                    CoreComponents.Type("Asteroid"),
                    PhysicsComponents.Transform()
                )

                local transform = entity:get(PhysicsComponents.Transform)
                transform:setPos(Position(beltPosX + a.px, beltPosY + a.py, beltPosZ + a.pz))
                transform:setScale(a.scale)

                -- Get mesh from pool (same as batch renderer uses)
                local lodMesh = beltCmp:getLodMesh()
                local highDetailMesh = lodMesh and lodMesh:get(0)
                -- Fallback to pool if belt has no mesh
                if not lodMesh then
                    lodMesh = AsteroidMeshPool:getFromSeed(a.rotSeed)
                    highDetailMesh = lodMesh and lodMesh:get(0)
                end

                -- Render with LodMesh for smooth LOD transitions (no pop)
                local asteroidMat = Materials.Asteroid()
                local asteroidPos = Position(beltPosX + a.px, beltPosY + a.py, beltPosZ + a.pz)
                local asteroidScale = a.scale
                local asteroidRot = getRotation(beltEntity, idx, a.rotSeed)

                if lodMesh then
                    entity:add(RenderComp(function(ent, blendMode)
                        if blendMode ~= BlendMode.Disabled then return end
                        local eye = CameraManager:getEye()
                        if not eye then return end
                        local rx = asteroidPos.x - eye.x
                        local ry = asteroidPos.y - eye.y
                        local rz = asteroidPos.z - eye.z
                        local distSq = rx*rx + ry*ry + rz*rz
                        local relPos = Vec3f(rx, ry, rz)
                        local mat = Matrix.FromPosRotScale(relPos, asteroidRot, asteroidScale)
                        local matIT = mat:inverse()
                        local sh = asteroidMat:getShaderState()
                        sh:start()
                        sh:shader():setMatrix('mWorld', mat)
                        sh:shader():setMatrixT('mWorldIT', matIT)
                        sh:shader():setFloat('scale', asteroidScale)
                        lodMesh:draw(distSq)
                        sh:stop()
                    end))
                end

                -- Physics: sphere collider (fast, good enough for asteroids)
                local rb = RigidBody.CreateSphere()
                rb:setKinematic(true)
                rb:setPos(Position(a.px, a.py, a.pz))
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
            end

            ::next_asteroid::
        end

        ::next_belt::
    end
end

--- Get all currently spawned asteroid entities (for map, labels, etc.)
---@return table Array of { entity, label, pos }
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
    rotationCache = {}
    totalSpawned = 0
end

return AsteroidFieldSystem
