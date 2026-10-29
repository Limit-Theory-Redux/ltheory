-- Checks the physics queries through the Lua FFI: entity ids in the results and their
-- lookup, the exclude and mask filters, positions far from the origin, sphere sweeps and
-- radius queries. Logs each failed check and ends with a failure exit code.
local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local PhysicsComponents   = require("Modules.Physics.Components")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class PhysicsQueryTest: Application
local QueryTest           = Subclass("PhysicsQueryTest", Application)

local RADIUS              = 0.003 -- gu, a solo ship
local STEP                = 1 / 60
local TOLERANCE           = 0.01 * RADIUS
-- Single precision can't place a ship-sized body this far from the origin.
local FAR                 = UniverseScaleConfig:auToGameUnits(1)
local GROUP_A             = 0x1
local GROUP_B             = 0x2

local function near(actual, expected)
    return math.abs(actual - expected) <= TOLERANCE
end

function QueryTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("PhysicsQueryTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---A kinematic sphere of `RADIUS` at `x` on the x axis, owned by a new entity.
function QueryTest:spawnSphere(x)
    local entity = Registry:createEntity()
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setScale(RADIUS)
    rb:setKinematic(true)
    rbCmp:setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(Position(x, 0, 0))
    return entity, rb
end

function QueryTest:onInit()
    self.passed, self.failed = 0, 0

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    local a, rbA = self:spawnSphere(FAR + 1)
    local b, rbB = self:spawnSphere(FAR + 2)
    local c, rbC = self:spawnSphere(FAR + 3)
    rbB:setCollisionGroup(GROUP_B)
    rbA:setSoftCcdPrediction(tuning.softCcdDistance(Config.game.shipFlight.maxSpeedGame))
    self.world:update(STEP)

    self:checkIds(a, rbA)
    self:checkRayCasts(a, b, c, rbA)
    self:checkShapeCasts(a, rbA)
    self:checkSweeps(a, b, rbA)
    self:checkRadiusQueries(b, c, rbB)

    -- A non-collidable body is still hit by a masked query.
    rbC:setCollidable(false)
    local hit = self.world:rayCastFiltered(self:rayAlongX(FAR, 4), rbA, GROUP_A)
    self:check(tonumber(hit.id) == c.id, "masked ray cast hits a non-collidable body")

    Log.Info("PhysicsQueryTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

function QueryTest:rayAlongX(from, length)
    local ray = Ray()
    ray.px, ray.py, ray.pz = from, 0, 0
    ray.dirx, ray.diry, ray.dirz = 1, 0, 0
    ray.tMin, ray.tMax = 0, length
    return ray
end

function QueryTest:checkIds(a, rbA)
    self:check(tonumber(rbA:getUserId()) == a.id, "the body carries its entity id")
    local found = Registry:getEntityById(rbA:getUserId())
    self:check(found ~= nil and found.id == a.id, "lookup by the uint64 id")
    self:check(Registry:getEntityById(0) == nil, "lookup of id 0 finds nothing")
end

function QueryTest:checkRayCasts(a, b, c, rbA)
    local ray = self:rayAlongX(FAR, 4)
    local hit = self.world:rayCast(ray)
    self:check(tonumber(hit.id) == a.id, "ray cast reports the entity id")
    self:check(hit.body == rbA, "ray cast reports the body")
    self:check(near(hit.posx, FAR + 1 - RADIUS), "ray cast position", tostring(hit.posx))
    self:check(hit.normx < -0.99, "ray cast normal", tostring(hit.normx))

    hit = self.world:rayCastFiltered(ray, rbA)
    self:check(tonumber(hit.id) == b.id, "filtered ray cast skips the excluded body")
    hit = self.world:rayCastFiltered(ray, rbA, GROUP_A)
    self:check(tonumber(hit.id) == c.id, "filtered ray cast skips bodies outside the mask")

    hit = self.world:rayCast(self:rayAlongX(FAR, 0.5))
    self:check(tonumber(hit.id) == 0, "a ray that misses reports id 0")
end

function QueryTest:checkShapeCasts(a, rbA)
    local probe = 2 / 3 * RADIUS
    -- Centred just outside the body's surface, the probes overlap it by a third of its radius.
    local touching = Position(FAR + 1 + RADIUS + probe / 2, 0, 0)
    local spheres = self.world:sphereCast(touching, probe)
    self:check(spheres.hits_len == 1 and tonumber(spheres.ids[0]) == a.id
        and spheres.hits[0] == rbA, "sphere cast reports the entity id")
    local boxes = self.world:boxCast(touching, Quat.Identity(), Vec3f(probe, probe, probe))
    self:check(boxes.hits_len == 1 and tonumber(boxes.ids[0]) == a.id,
        "box cast reports the entity id")
end

function QueryTest:checkSweeps(a, b, rbA)
    local from, to = Position(FAR, 0, 0), Position(FAR + 4, 0, 0)
    local sweepRadius = 2 * RADIUS
    local hit = self.world:sphereSweep(from, to, sweepRadius)
    self:check(tonumber(hit.id) == a.id, "sphere sweep reports the entity id")
    self:check(near(hit.t, 1 - RADIUS - sweepRadius), "sphere sweep travel distance", tostring(hit.t))
    self:check(near(hit.posx, FAR + 1 - RADIUS), "sphere sweep contact", tostring(hit.posx))
    hit = self.world:sphereSweep(from, to, sweepRadius, rbA)
    self:check(tonumber(hit.id) == b.id, "sphere sweep skips the excluded body")
end

function QueryTest:checkRadiusQueries(b, c, rbB)
    local center = Position(FAR + 2.4, 0, 0)
    local found = self.world:queryRadius(center, 1)
    self:check(found.len == 2 and tonumber(found.ids[0]) == b.id and tonumber(found.ids[1]) == c.id,
        "radius query lists the nearest first")
    self:check(found.len == 2 and near(found.distances[0], 0.4 - RADIUS)
        and near(found.distances[1], 0.6 - RADIUS), "radius query surface distances")
    found = self.world:queryRadius(center, 1, rbB)
    self:check(found.len == 1 and tonumber(found.ids[0]) == c.id, "radius query skips the excluded body")
end

return QueryTest
