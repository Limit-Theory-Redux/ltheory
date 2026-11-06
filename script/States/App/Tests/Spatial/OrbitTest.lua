-- Checks orbits: the orbit data is set up from where the bodies are, predicted positions and
-- velocities match what the system and the physics step produce, and rings follow their planet.
-- Logs each failed check and ends with a failure exit code.

local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local Entity              = require("Core.ECS.Entity")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local OrbitalSystem       = require("Modules.CelestialObjects.Systems.OrbitalSystem")

---@class SpatialOrbitTest: Application
local OrbitTest = Subclass("SpatialOrbitTest", Application)

local STEP = 1 / 60
-- Steps run to compare a prediction with the simulation.
local SIM_STEPS = 600
-- Half-width of the central difference that approximates a velocity, in seconds.
local DIFF_H = 1e-3
local POSITION_TOLERANCE = 1e-6
-- Relative error allowed between a velocity and its approximation.
local VELOCITY_TOLERANCE = 1e-4

local PLANET_RADIUS, PLANET_PHASE, PLANET_INCLINATION_DEG = 1000, 0.7, 25
local MOON_RADIUS, MOON_PHASE = 40, 2.0
-- The speed rule of `OrbitalSystem`: `0.5 / sqrt(radius)`.
local SPEED_SCALE = 0.5

local function near(actual, expected, tolerance)
    return math.abs(actual - expected) <= tolerance
end

function OrbitTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("SpatialOrbitTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---Both positions are near each other, within `POSITION_TOLERANCE`.
function OrbitTest:checkSamePos(actual, expected, what)
    local error = math.max(math.abs(actual.x - expected.x), math.abs(actual.y - expected.y),
        math.abs(actual.z - expected.z))
    self:check(error <= POSITION_TOLERANCE, what, string.format("(%.9g, %.9g, %.9g) vs (%.9g, %.9g, %.9g)",
        actual.x, actual.y, actual.z, expected.x, expected.y, expected.z))
end

---A kinematic sphere owned by `entity`, at `pos`.
function OrbitTest:giveBody(entity, pos)
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rbCmp:setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(pos)
    return rb
end

---Where a body is that orbits `center` at `radius` with `phase` and `inclinationRad`.
local function placed(center, radius, phase, inclinationRad)
    local z = math.sin(phase) * radius
    return Position(center.x + math.cos(phase) * radius, center.y + z * math.sin(inclinationRad),
        center.z + z * math.cos(inclinationRad))
end

function OrbitTest:buildScene()
    local origin = Position(0, 0, 0)
    local inclination = math.rad(PLANET_INCLINATION_DEG)

    self.star = Entity.Create("StarEntity", CelestialComponents.Star())
    self.starRb = self:giveBody(self.star, origin)

    local planetPos = placed(origin, PLANET_RADIUS, PLANET_PHASE, inclination)
    self.planet = Entity.Create("PlanetEntity", CelestialComponents.Planet())
    self.planetRb = self:giveBody(self.planet, planetPos)
    self.planet:add(PhysicsComponents.Transform())
    self.planet:add(SpatialComponents.Orbit(PLANET_RADIUS))
    self.planet:add(SpatialComponents.Inclination(PLANET_INCLINATION_DEG))
    Registry:attachEntity(self.star, self.planet)

    self.moon = Entity.Create("MoonEntity", CelestialComponents.Moon())
    self.moonRb = self:giveBody(self.moon, placed(planetPos, MOON_RADIUS, MOON_PHASE, 0))
    self.moon:add(SpatialComponents.Orbit(MOON_RADIUS))
    Registry:attachEntity(self.planet, self.moon)

    self.ring = Entity.Create("RingEntity", CelestialComponents.AsteroidRing())
    self.ringRb = self:giveBody(self.ring, planetPos)
    Registry:attachEntity(self.planet, self.ring)

    self.orbiters, self.followers = OrbitalSystem:collectOrbiters(self.star)
    self.world:update(STEP)
end

function OrbitTest:checkSetUp()
    self:check(#self.orbiters == 2, "a planet and a moon orbit", tostring(#self.orbiters))
    -- Entities are compared by id: a query yields new handles.
    self:check(self.orbiters[1] and self.orbiters[1].entity.id == self.planet.id
        and self.orbiters[2] and self.orbiters[2].entity.id == self.moon.id, "a parent is listed before its child")
    self:check(#self.followers == 1 and self.followers[1].entity.id == self.ring.id, "a ring follows its planet")

    local orbit = self.planet:get(SpatialComponents.Orbit)
    self:check(near(orbit:getPhase(), PLANET_PHASE, 1e-9), "the phase comes from the generated position",
        tostring(orbit:getPhase()))
    self:check(near(orbit:getInclination(), math.rad(PLANET_INCLINATION_DEG), 1e-12), "the inclination is in radians")
    self:check(near(orbit:getAngularSpeed(), SPEED_SCALE / math.sqrt(PLANET_RADIUS), 1e-12), "the speed follows the radius")

    self:checkSamePos(OrbitalSystem:predictPos(self.planet, 0), self.planetRb:getPos(),
        "the prediction for now is where the planet is")
    self:checkSamePos(OrbitalSystem:predictPos(self.moon, 0), self.moonRb:getPos(),
        "the prediction for now is where the moon is")
    self:checkSamePos(OrbitalSystem:predictPos(self.star, 10), self.starRb:getPos(), "a body without an orbit stays put")
    local vx, vy, vz = OrbitalSystem:velocity(self.star, 0)
    self:check(vx == 0 and vy == 0 and vz == 0, "a body without an orbit has no velocity")
end

---The orbit component alone: a body on a tilted orbit reaches the height the tilt gives.
function OrbitTest:checkInclination()
    local radius, tilt = 10, math.rad(30)
    local orbit = SpatialComponents.Orbit(radius)
    orbit:setInclination(tilt)
    orbit:setAngularSpeed(1)
    local x, y, z = orbit:offsetAt(math.pi / 2)
    self:check(near(x, 0, 1e-9) and near(y, radius * math.sin(tilt), 1e-9) and near(z, radius * math.cos(tilt), 1e-9),
        "a quarter orbit later the body is at the tilted height", string.format("%.6f %.6f %.6f", x, y, z))
    local _, _, vz = orbit:relativeVelocityAt(0)
    self:check(near(vz, radius * math.cos(tilt), 1e-9), "the velocity is tilted like the orbit")
end

---The velocity of `entity` at `t` against a central difference of its predicted position.
function OrbitTest:checkVelocityIsPositionDerivative(entity, what)
    local before = OrbitalSystem:predictPos(entity, -DIFF_H)
    local after = OrbitalSystem:predictPos(entity, DIFF_H)
    local vx, vy, vz = OrbitalSystem:velocity(entity, 0)
    local dx, dy, dz = (after.x - before.x) / (2 * DIFF_H), (after.y - before.y) / (2 * DIFF_H),
        (after.z - before.z) / (2 * DIFF_H)
    local speed = math.sqrt(vx * vx + vy * vy + vz * vz)
    local error = math.sqrt((vx - dx) ^ 2 + (vy - dy) ^ 2 + (vz - dz) ^ 2)
    self:check(speed > 0 and error <= VELOCITY_TOLERANCE * speed, what .. " velocity is the position's derivative",
        string.format("error %.3g of speed %.3g", error, speed))
end

function OrbitTest:checkMatchesSimulation()
    local seconds = SIM_STEPS * STEP
    local planetPredicted = OrbitalSystem:predictPos(self.planet, seconds)
    local moonPredicted = OrbitalSystem:predictPos(self.moon, seconds)

    for _ = 1, SIM_STEPS do
        OrbitalSystem:update(self.orbiters, STEP, self.followers)
        self.world:update(STEP)
    end

    self:checkSamePos(self.planetRb:getPos(), planetPredicted, "the planet is where it was predicted to be")
    self:checkSamePos(self.moonRb:getPos(), moonPredicted, "the moon, which moves with its planet, is where it was predicted to be")
    self:checkSamePos(self.planet:get(PhysicsComponents.Transform):getPos(), planetPredicted,
        "the transform follows the planet")
    self:checkSamePos(self.ringRb:getPos(), self.planetRb:getPos(), "a ring is on its planet")
end

---After a step, physics reports the velocity of the move: the body moved from the position it had
---`STEP` ago, so the velocity predicted for half a step ago is the one it has.
function OrbitTest:checkPhysicsSeesVelocity()
    OrbitalSystem:update(self.orbiters, STEP, self.followers)
    self.world:update(STEP)
    for _, body in ipairs({ { self.planet, self.planetRb, "planet" }, { self.moon, self.moonRb, "moon" } }) do
        local entity, rb, name = body[1], body[2], body[3]
        local vx, vy, vz = OrbitalSystem:velocity(entity, -STEP / 2)
        local velocity = rb:getVelocity()
        local speed = math.sqrt(vx * vx + vy * vy + vz * vz)
        local error = math.sqrt((velocity.x - vx) ^ 2 + (velocity.y - vy) ^ 2 + (velocity.z - vz) ^ 2)
        self:check(error <= VELOCITY_TOLERANCE * speed, "physics sees the velocity of the " .. name,
            string.format("error %.3g of speed %.3g", error, speed))
    end
    local planetVelocity = self.planetRb:getVelocity()
    local ringVelocity = self.ringRb:getVelocity()
    self:check(near(ringVelocity.x, planetVelocity.x, 1e-3) and near(ringVelocity.z, planetVelocity.z, 1e-3),
        "a ring has its planet's velocity")
end

function OrbitTest:onInit()
    self.passed, self.failed = 0, 0

    self.world = Physics.Create()
    self:buildScene()
    self:checkSetUp()
    self:checkInclination()
    self:checkVelocityIsPositionDerivative(self.planet, "the planet's")
    self:checkVelocityIsPositionDerivative(self.moon, "the moon's")
    self:checkMatchesSimulation()
    self:checkPhysicsSeesVelocity()

    Log.Info("SpatialOrbitTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return OrbitTest
