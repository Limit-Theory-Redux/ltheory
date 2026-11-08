-- Checks the chase camera's distance from a moving ship: the rest distance when still, a
-- log-curve trail that grows from normal flight to travel-drive speed and stops at the cap, no
-- extra lag from moving at a steady speed, the same shares at another zoom, and a return to the
-- rest distance once the ship stops. Logs each failed check and ends with a failure exit code.
local Application            = require("States.Application")
local Registry               = require("Core.ECS.Registry")
local PhysicsComponents      = require("Modules.Physics.Components")
local ConstructComponents    = require("Modules.Constructs.Components")
local CameraEntity           = require("Modules.Cameras.Entities").Camera
local CameraDataComponent    = require("Modules.Cameras.Components").CameraData
local ChaseCameraController  = require("Modules.Cameras.Managers.CameraControllers.ChaseCameraController")
local FlightProfileRegistry  = require("Shared.Registries.FlightProfileRegistry")

---@class CamerasChaseCameraTest: Application
local ChaseCameraTest = Subclass("CamerasChaseCameraTest", Application)

local STEP            = 1 / 60
local HULL            = Enums.ShipHulls.Solo
local SOLO            = Enums.FlightProfile.Solo
local MAX_LAG         = 1.5
local REFERENCE_RATIO = 0.1
local TOP_RATIO       = 40000
-- Speeds as multiples of the profile's max speed: normal flight, a drive in a moon zone, and
-- beyond the top of the curve.
local NORMAL_RATIO    = 1
local MOON_RATIO      = 200
local BEYOND_RATIO    = 2 * TOP_RATIO
-- The camera's smoothing rate (1/s); this many of its time constants let it settle.
local SMOOTHING_RATE        = 10
local SETTLE_TIME_CONSTANTS = 10
local SETTLE_FRAMES   = math.ceil(SETTLE_TIME_CONSTANTS / (SMOOTHING_RATE * STEP))
local ZOOM_FAR        = 4
local TOLERANCE       = 1e-3
-- Speeds are single precision.
local SPEED_TOLERANCE = 1e-4

local function distance(a, b)
    local dx, dy, dz = a.x - b.x, a.y - b.y, a.z - b.z
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

local function close(actual, expected)
    return math.abs(actual - expected) <= TOLERANCE * math.abs(expected)
end

function ChaseCameraTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("CamerasChaseCameraTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---The distance the camera should settle at for a ship at `speed`, given its rest distance.
function ChaseCameraTest:expectedDistance(rest, speed)
    local reference = REFERENCE_RATIO * self.maxSpeed
    local top = TOP_RATIO * self.maxSpeed
    local share = math.min(math.log(1 + speed / reference) / math.log(1 + top / reference), 1)
    return rest * (1 + MAX_LAG * share)
end

---A still ship at the origin, without drag, and a chase camera on it at the given zoom.
function ChaseCameraTest:spawn(radius)
    local entity = Registry:createEntity()
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setScale(Config.game.shipHulls.scale[HULL])
    rbCmp:setRigidBody(rb)
    entity:add(ConstructComponents.FlightProfile(FlightProfileRegistry:get(SOLO)))
    self.world:addRigidBody(rb)
    rb:setPos(Position(0, 0, 0))

    local cameraEntity = CameraEntity()
    local controller = ChaseCameraController(cameraEntity, {
        radius = radius,
        maxLag = MAX_LAG,
        trailReferenceRatio = REFERENCE_RATIO,
        trailTopSpeedRatio = TOP_RATIO,
    })
    cameraEntity:get(CameraDataComponent):setController(controller)
    controller:enable()
    controller:setTarget(entity)
    controller:onPreRender(STEP)
    return { entity = entity, rb = rb, controller = controller, speed = 0 }
end

function ChaseCameraTest:despawn(rig)
    self.world:removeRigidBody(rig.rb)
    Registry:destroyEntity(rig.entity)
end

---Change the ship's speed along +x to `speed` with one physics step's force.
function ChaseCameraTest:setSpeed(rig, speed)
    local force = rig.rb:getMass() * (speed - rig.speed) / STEP
    rig.rb:applyForce(Vec3f(force, 0, 0))
    self.world:update(STEP)
    rig.speed = speed
    rig.controller:onPreRender(STEP)
end

---Let the ship fly on and the camera settle; returns the camera-ship distance and the speed.
function ChaseCameraTest:settle(rig)
    for _ = 1, SETTLE_FRAMES do
        self.world:update(STEP)
        rig.controller:onPreRender(STEP)
    end
    return distance(rig.controller:getPosition(), rig.rb:getPos()), rig.rb:getSpeed()
end

---Settle at `ratio` times the profile's max speed and check the distance against the curve.
function ChaseCameraTest:checkSpeed(rig, rest, ratio, what)
    self:setSpeed(rig, ratio * self.maxSpeed)
    local gap, speed = self:settle(rig)
    self:check(math.abs(speed - ratio * self.maxSpeed) <= SPEED_TOLERANCE * ratio * self.maxSpeed,
        what .. ": the ship holds its speed", string.format("%g", speed))
    local expected = self:expectedDistance(rest, speed)
    self:check(close(gap, expected), what .. ": the distance follows the trail curve",
        string.format("%g, expected %g", gap, expected))
    return gap
end

function ChaseCameraTest:onInit()
    self.passed, self.failed = 0, 0
    self.maxSpeed = FlightProfileRegistry:get(SOLO).maxSpeed

    self.world = Physics.Create()
    local tuning = Config.game.physics
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    self:checkTrail(1)
    self:checkTrail(ZOOM_FAR)

    Log.Info("CamerasChaseCameraTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

function ChaseCameraTest:checkTrail(radius)
    local zoom = string.format("zoom %g", radius)
    local rig = self:spawn(radius)
    local rest = self:settle(rig)
    self:check(rest > 0, zoom .. ": the camera sits away from a still ship", tostring(rest))

    local normal = self:checkSpeed(rig, rest, NORMAL_RATIO, zoom .. ", normal flight")
    local moon = self:checkSpeed(rig, rest, MOON_RATIO, zoom .. ", drive in a moon zone")
    local top = self:checkSpeed(rig, rest, TOP_RATIO, zoom .. ", full drive")
    local beyond = self:checkSpeed(rig, rest, BEYOND_RATIO, zoom .. ", beyond the top speed")
    self:check(rest < normal and normal < moon and moon < top,
        zoom .. ": the camera pulls back as the speed grows",
        string.format("%g, %g, %g, %g", rest, normal, moon, top))
    self:check(close(top, (1 + MAX_LAG) * rest) and close(beyond, top),
        zoom .. ": the trail stops at the cap", string.format("%g, %g, cap %g", top, beyond, (1 + MAX_LAG) * rest))

    self:setSpeed(rig, 0)
    local stopped = self:settle(rig)
    self:check(close(stopped, rest), zoom .. ": the camera returns to the rest distance",
        string.format("%g, rest %g", stopped, rest))
    self:despawn(rig)
end

return ChaseCameraTest
