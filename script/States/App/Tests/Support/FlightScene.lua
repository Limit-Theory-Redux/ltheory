-- The bodies the flight test apps put in a physics world: Solo ships set up like the player's, fixed
-- spheres, and a planet with a gravity well on an orbit around a star. Distances are game units.

local Registry              = require("Core.ECS.Registry")
local Entity                = require("Core.ECS.Entity")
local PhysicsComponents     = require("Modules.Physics.Components")
local SpatialComponents     = require("Modules.Spatial.Components")
local SpatialEntities       = require("Modules.Spatial.Entities")
local CelestialComponents   = require("Modules.CelestialObjects.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local OrbitalSystem         = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local FlightProfileHelper   = require("Shared.Helpers.FlightProfileHelper")

local HULL = Enums.ShipHulls.Solo
local SOLO = Enums.FlightProfile.Solo

---@class FlightScene
local FlightScene = {
    -- The moving well: a planet on a circular orbit around a star, with a gravity well.
    WELL_ORBIT_RADIUS = 10000,
    WELL_RADIUS       = 1000,
}

---A Solo ship set up like the player's, at `start`, in `world`.
---@param world Physics
---@param start Position
---@param options { inWell: boolean|nil, autoPilot: boolean|nil }|nil `inWell`: a gravity well component lets a zone carry the ship; `autoPilot`: the autopilot component
---@return { entity: Entity, rb: RigidBody }
function FlightScene.spawnShip(world, start, options)
    options = options or {}
    local profile = FlightProfileRegistry:get(SOLO)
    local entity = Registry:createEntity()
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setScale(Config.game.shipHulls.scale[HULL])
    FlightProfileHelper.applyToRigidBody(profile, rb)
    entity:add(ConstructComponents.FlightProfile(profile))
    rbCmp:setRigidBody(rb)
    world:addRigidBody(rb)
    rb:setPos(start)
    if options.inWell then entity:add(PhysicsComponents.GravityWell(profile.linearDrag)) end
    entity:add(ConstructComponents.TravelDrive())
    entity:add(ConstructComponents.ControlInput())
    if options.autoPilot then entity:add(ConstructComponents.AutoPilot()) end
    return { entity = entity, rb = rb }
end

---A kinematic sphere of `radius` at `pos`, owned by `entity` and in the world.
---@param world Physics
---@param entity Entity
---@param pos Position
---@param radius gameunit
---@return RigidBody
function FlightScene.giveKinematicBody(world, entity, pos, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setScale(radius)
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    world:addRigidBody(rb)
    rb:setPos(pos)
    return rb
end

---A fixed sphere of `radius` at `position`.
---@param world Physics
---@param position Position
---@param radius gameunit
---@return { entity: Entity, rb: RigidBody }
function FlightScene.spawnBody(world, position, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(radius)
    local entity = Entity.Create("FlightSceneBody", PhysicsComponents.RigidBody(rb))
    world:addRigidBody(rb)
    return { entity = entity, rb = rb }
end

---A planet of `radius` at `planetPos` on a circular orbit around a star, with a gravity well, whose
---speed on its orbit is `speedup` times the Solo top speed.
---@param world Physics
---@param planetPos Position
---@param radius gameunit
---@param speedup number
---@param options { moon: boolean|nil, wellRadius: gameunit|nil }|nil `moon`: the body is a moon, not a planet (its zone caps the drive lower); `wellRadius`: the well's radius, `WELL_RADIUS` when nil
---@return table well `planetBody` is the planet as a target: its entity and rigid body
function FlightScene.buildWell(world, planetPos, radius, speedup, options)
    local star = Entity.Create("StarEntity", CelestialComponents.Star())
    local starRb = FlightScene.giveKinematicBody(world, star,
        Position(planetPos.x - FlightScene.WELL_ORBIT_RADIUS, planetPos.y, planetPos.z), 1)

    local isMoon = options and options.moon
    local planet = isMoon and Entity.Create("MoonEntity", CelestialComponents.Moon())
        or Entity.Create("PlanetEntity", CelestialComponents.Planet())
    local planetRb = FlightScene.giveKinematicBody(world, planet, planetPos, radius)
    planet:add(PhysicsComponents.Transform()):setPos(planetPos)
    planet:add(SpatialComponents.Orbit(FlightScene.WELL_ORBIT_RADIUS))
    Registry:attachEntity(star, planet)

    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setRadius(options and options.wellRadius or FlightScene.WELL_RADIUS)
    Registry:attachEntity(planet, zone)

    local orbiters, followers = OrbitalSystem:collectOrbiters(star)
    local topSpeed = FlightProfileRegistry:get(SOLO).maxSpeed
    for _, orbiter in ipairs(orbiters) do
        orbiter.orbit:setAngularSpeed(speedup * topSpeed / FlightScene.WELL_ORBIT_RADIUS)
    end
    GravityWellSystem:collectZones()
    return {
        star = star, starRb = starRb, zone = zone, orbiters = orbiters, followers = followers,
        planetPos = planetPos, planetBody = { entity = planet, rb = planetRb },
    }
end

---Remove a moving well's bodies from physics and destroy its entities.
---@param world Physics
---@param well table From `buildWell`
function FlightScene.despawnWell(world, well)
    world:removeRigidBody(well.starRb)
    Registry:destroyEntity(well.zone)
    Registry:destroyEntity(well.star)
    GravityWellSystem:collectZones()
end

---Remove the bodies from physics and destroy their entities, so the actuation systems no longer see them.
---@param world Physics
---@param things { entity: Entity, rb: RigidBody }[]
function FlightScene.despawn(world, things)
    for _, thing in ipairs(things) do
        world:removeRigidBody(thing.rb)
        Registry:destroyEntity(thing.entity)
    end
end

return FlightScene
