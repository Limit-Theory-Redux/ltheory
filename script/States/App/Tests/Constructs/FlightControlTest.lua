-- Checks that ships fly from their ControlInput alone: thrust, boost, torque, the travel
-- drive requests and ramps, and frame-rate independence. Logs each failed check and ends
-- with a failure exit code.
local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local PhysicsComponents     = require("Modules.Physics.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local FlightProfile         = require("Shared.Types.FlightProfile")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local FlightProfileHelper   = require("Shared.Helpers.FlightProfileHelper")

---@class ConstructsFlightControlTest: Application
local FlightControlTest = Subclass("ConstructsFlightControlTest", Application)

local STEP            = 1 / 60
-- Distance between test ships along x; they fly along their own forward axis, so they never meet.
local SHIP_SPACING    = 1000
local HULL            = Enums.ShipHulls.Solo
local SOLO            = Enums.FlightProfile.Solo
-- A profile only this test registers: Solo with twice the thrust and torque.
local STRONG          = 1000
local STRONG_FACTOR   = 2
-- Strafing and vertical thrust relative to forward thrust, and roll torque relative to pitch,
-- as the player ship has always flown.
local LATERAL_RATIO   = 0.25
local ROLL_RATIO      = 0.3
-- Forward speed under full thrust is within this of the profile's speed once settled.
local SETTLED         = 0.99
local RELATIVE_TOLERANCE = 0.01
-- Velocities are single precision: directions agree to about this much.
local SINGLE_PRECISION = 1e-5
-- A bound that only turns a drive that never changes state into a failure.
local MAX_DRIVE_STEPS = 10000
-- A drive multiplier far above any cap used below.
local HIGH_MULT       = 20000
local LOW_CAP         = 100

local Engage    = Enums.TravelDriveRequest.Engage
local Disengage = Enums.TravelDriveRequest.Disengage

local function close(actual, expected, tolerance)
    return math.abs(actual - expected) <= tolerance * math.max(math.abs(expected), 1e-12)
end

local function dot(a, b)
    return a.x * b.x + a.y * b.y + a.z * b.z
end

local function length(v)
    return math.sqrt(dot(v, v))
end

function FlightControlTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("ConstructsFlightControlTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---A ship with a rigid body, a flight profile, a travel drive and a control input, set up like
---the player ship. `profileId` defaults to the Solo profile.
function FlightControlTest:spawnShip(profileId)
    local profile = FlightProfileRegistry:get(profileId or SOLO)
    local entity = Registry:createEntity()
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setScale(Config.game.shipHulls.scale[HULL])
    FlightProfileHelper.applyToRigidBody(profile, rb)
    entity:add(ConstructComponents.FlightProfile(profile))
    rbCmp:setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(Position(self.nextX, 0, 0))
    self.nextX = self.nextX + SHIP_SPACING

    local drive = entity:add(ConstructComponents.TravelDrive())
    local input = entity:add(ConstructComponents.ControlInput())
    return { entity = entity, rb = rb, drive = drive, input = input }
end

function FlightControlTest:despawn(ship)
    self.world:removeRigidBody(ship.rb)
    Registry:destroyEntity(ship.entity)
end

---One simulation step in the order the apps use: drive, flight, physics.
function FlightControlTest:step(dt)
    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    self.world:update(dt)
end

---Step until `done()` holds; false if it still doesn't after `MAX_DRIVE_STEPS`.
function FlightControlTest:stepUntil(done)
    for _ = 1, MAX_DRIVE_STEPS do
        if done() then return true end
        self:step(STEP)
    end
    return done()
end

function FlightControlTest:onInit()
    self.passed, self.failed = 0, 0
    self.nextX = 0

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    self:checkProfiles()
    self:checkThrust()
    self:checkTorque()
    self:checkDriveRequests()
    self:checkDriveSpeedCap()

    Log.Info("ConstructsFlightControlTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

---Register a profile like Solo, with thrust and torque scaled by `factor`.
local function registerStrongProfile(id, factor)
    local solo = FlightProfileRegistry:get(SOLO)
    local args = { id = id, name = "Strong" }
    for field, value in pairs(solo) do
        if field ~= "id" and field ~= "name" then args[field] = value end
    end
    for _, field in ipairs({ "thrustForward", "thrustRight", "thrustUp",
        "torquePitch", "torqueYaw", "torqueRoll" }) do
        args[field] = args[field] * factor
    end
    FlightProfile(args)
end

function FlightControlTest:checkProfiles()
    self:check(FlightProfileRegistry:has(SOLO), "the Solo profile is registered")
    self:check(not pcall(FlightProfileRegistry.get, FlightProfileRegistry, STRONG),
        "an unknown profile id is an error")
    self:check(FlightProfile({ id = STRONG + 1, name = "" }) == nil, "a profile without a name is refused")
    self:check(not pcall(FlightProfile, { id = STRONG + 1, name = "Broken", mass = -1 }),
        "a profile with a non-positive field is an error")

    registerStrongProfile(STRONG, STRONG_FACTOR)
    local solo, strong = self:spawnShip(SOLO), self:spawnShip(STRONG)
    for _, ship in ipairs({ solo, strong }) do
        ship.input:setThrustForward(1)
        ship.input:setPitch(1)
    end
    self:step(STEP)
    self:check(close(strong.rb:getSpeed() / solo.rb:getSpeed(), STRONG_FACTOR, RELATIVE_TOLERANCE),
        "a ship accelerates by its own profile's thrust")
    self:check(close(length(strong.rb:getVelocityA()) / length(solo.rb:getVelocityA()), STRONG_FACTOR,
        RELATIVE_TOLERANCE), "a ship turns by its own profile's torque")
    self:despawn(solo)
    self:despawn(strong)

    -- Full forward thrust settles at the profile's speed.
    local ship = self:spawnShip(SOLO)
    local maxSpeed = FlightProfileRegistry:get(SOLO).maxSpeed
    ship.input:setThrustForward(1)
    self:check(self:stepUntil(function() return ship.rb:getSpeed() >= SETTLED * maxSpeed end),
        "forward thrust reaches the profile's speed")
    self:check(ship.rb:getSpeed() <= maxSpeed * (1 + RELATIVE_TOLERANCE),
        "forward thrust does not exceed the profile's speed", tostring(ship.rb:getSpeed()))
    self:despawn(ship)

    -- The strafe and roll limits keep the proportions the player ship has always had.
    local strafing, rolling, pitching = self:spawnShip(), self:spawnShip(), self:spawnShip()
    local forwarding = self:spawnShip()
    forwarding.input:setThrustForward(1)
    strafing.input:setThrustRight(1)
    rolling.input:setRoll(1)
    pitching.input:setPitch(1)
    self:step(STEP)
    self:check(close(strafing.rb:getSpeed() / forwarding.rb:getSpeed(), LATERAL_RATIO, RELATIVE_TOLERANCE),
        "strafing thrust is a quarter of forward thrust")
    self:check(close(length(rolling.rb:getVelocityA()) / length(pitching.rb:getVelocityA()), ROLL_RATIO,
        RELATIVE_TOLERANCE), "roll torque is 0.3 of pitch torque")
    for _, s in ipairs({ strafing, rolling, pitching, forwarding }) do self:despawn(s) end
end

function FlightControlTest:checkThrust()
    local idle, flying, boosted, over = self:spawnShip(), self:spawnShip(), self:spawnShip(), self:spawnShip()
    flying.input:setThrustForward(1)
    boosted.input:setThrustForward(1)
    boosted.input:setBoost(1)
    over.input:setThrustForward(5) -- beyond the axis range
    self:step(STEP)

    local forward = flying.rb:getRot():getForward()
    local velocity = flying.rb:getVelocity()
    self:check(flying.rb:getSpeed() > 0, "a ship with forward input moves")
    self:check(dot(velocity, forward) > 0.999 * length(velocity), "forward input thrusts along the nose")
    self:check(idle.rb:getSpeed() == 0, "a ship without input stays still")
    self:check(close(boosted.rb:getSpeed() / flying.rb:getSpeed(), 3, RELATIVE_TOLERANCE),
        "full boost triples forward thrust", tostring(boosted.rb:getSpeed() / flying.rb:getSpeed()))
    self:check(close(over.rb:getSpeed(), flying.rb:getSpeed(), RELATIVE_TOLERANCE),
        "axes are clamped to 1")

    -- The component is the only input: a pending request without axes moves nothing.
    flying.input:clearAxes()
    local before = flying.rb:getVelocity()
    self:step(STEP)
    local after = flying.rb:getVelocity()
    self:check(length(after) < length(before), "a ship without input only slows down")

    for _, ship in ipairs({ idle, flying, boosted, over }) do self:despawn(ship) end
end

function FlightControlTest:checkTorque()
    local turning, steering = self:spawnShip(), self:spawnShip()
    turning.input:setPitch(1)
    steering.input:setPitch(1)
    steering.drive:setState("active")
    self:step(STEP)

    local axis = turning.rb:getRot():mulV(Vec3f(1, 0, 0))
    local spin = turning.rb:getVelocityA()
    self:check(length(spin) > 0 and dot(spin, axis) > 0.999 * length(spin), "pitch input turns about the right axis")
    self:check(close(length(steering.rb:getVelocityA()) / length(spin), 0.1, RELATIVE_TOLERANCE),
        "an active drive steers with a tenth of the torque")

    self:despawn(turning)
    self:despawn(steering)
end

function FlightControlTest:checkDriveRequests()
    local cfg = Config.game.travelDrive

    -- Engage charges, the request is consumed, and charging ends in the active state.
    local ship = self:spawnShip()
    ship.input:setDriveRequest(Engage)
    self:step(STEP)
    self:check(ship.drive:getState() == "charging", "an engage request starts charging")
    self:check(ship.input:getDriveRequest() == Enums.TravelDriveRequest.None, "the request is consumed")
    self:check(self:stepUntil(function() return ship.drive:getState() == "active" end),
        "charging ends in the active state")
    self:check(ship.drive:getChargeTime() >= cfg.chargeRequired, "charging takes the configured time")

    -- Thrust is ignored while active; the drive pushes along the nose.
    ship.input:setThrustRight(1)
    self:step(STEP)
    local right = ship.rb:getRot():getRight()
    self:check(math.abs(dot(ship.rb:getVelocity(), right)) < SINGLE_PRECISION * ship.rb:getSpeed(),
        "lateral thrust is ignored while the drive is active")

    -- A fast drive takes several steps to slow down.
    ship.drive:setCurrentMult(HIGH_MULT)
    ship.input:setDriveRequest(Disengage)
    self:step(STEP)
    self:check(ship.drive:getState() == "decelerating", "a disengage request stops an active drive")
    self:check(self:stepUntil(function() return ship.drive:getState() == "idle" end),
        "a stopping drive ends idle")
    self:check(ship.drive:getCurrentMult() == 1.0, "an idle drive has no multiplier")
    self:despawn(ship)

    -- Disengage while charging cancels the charge.
    ship = self:spawnShip()
    ship.input:setDriveRequest(Engage)
    self:step(STEP)
    ship.input:setDriveRequest(Disengage)
    self:step(STEP)
    self:check(ship.drive:getState() ~= "charging" and ship.drive:getState() ~= "active",
        "a disengage request cancels charging")
    self:despawn(ship)
end

---The multiplier of a drive that starts at `HIGH_MULT` under a cap, after `seconds` of
---simulation in steps of `dt`.
function FlightControlTest:multAfter(seconds, dt)
    local ship = self:spawnShip()
    ship.drive:setState("active")
    ship.drive:setCurrentMult(HIGH_MULT)
    ship.input:setDriveSpeedCap(LOW_CAP)
    for _ = 1, math.floor(seconds / dt + 0.5) do self:step(dt) end
    local mult = ship.drive:getCurrentMult()
    self:despawn(ship)
    return mult
end

function FlightControlTest:checkDriveSpeedCap()
    local cfg = Config.game.travelDrive
    local period = 1 / cfg.rampDownSpeed
    local expected = LOW_CAP + (HIGH_MULT - LOW_CAP) * math.exp(-1)

    local atPeriod = self:multAfter(period, STEP)
    self:check(close(atPeriod, expected, RELATIVE_TOLERANCE),
        "a capped drive approaches the cap exponentially", tostring(atPeriod))
    self:check(atPeriod > LOW_CAP, "a capped drive never undershoots the cap")

    local slow, fast = self:multAfter(period, 1 / 30), self:multAfter(period, 1 / 120)
    self:check(close(slow, fast, 1e-6), "the ramp does not depend on the step length",
        string.format("%.6f vs %.6f", slow, fast))
end

return FlightControlTest
