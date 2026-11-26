-- Measures how a ship turns while its travel drive runs: the drive thrusts along the nose, the ship
-- has no lateral thrust and steers with a tenth of its torque, so the velocity follows the nose
-- through the linear drag alone. For each drive multiplier it reads the nose's spin at full yaw,
-- flies steady circles with the nose held at a lead angle from the velocity, against the model
-- `rate = drag · tan(lead)`, `speed = straight speed · cos(lead)`, and then corners of several angles
-- flown with the nose turned straight to the new heading: the time to realign, the lowest speed and
-- how far the ship overshoots the corner, as the distance and the flight time by which a turn would
-- have to start earlier. The numbers are logged, and checked against the model and against each
-- other (the times do not depend on the multiplier). Logs each failed check and ends with a failure
-- exit code.

local Application         = require("States.Application")
local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem   = require("Modules.Constructs.Systems.TravelDriveSystem")
local FlightScene         = require("States.App.Tests.Support.FlightScene")
local ScaleConfig         = require("Config.Gen.UniverseScaleConfig")

---@class ConstructsDriveTurnTest: Application
local DriveTurnTest = Subclass("ConstructsDriveTurnTest", Application)

local STEP = 1 / 60
-- The drive multipliers flown: well under the moon's cap, the moon's cap (200) and the planet's (2000).
local MULTIPLIERS = { 50, 200, 2000 }
local CORNER_DEGREES = { 20, 45, 90 }
local LEAD_DEGREES = { 15, 30, 45 }
-- A ship has settled on the drive's speed after this many of the speed's time constants (1 / drag).
local SETTLE_TIME_CONSTANTS = 6
-- The speed at the start of a turn is within this share of the multiplier times the top speed.
local SPEED_TOLERANCE = 0.02
-- The nose's spin and the side it turns to are read after this long at full yaw, in seconds: short,
-- so that the nose has not turned a half circle (it spins at several radians per second).
local SPIN_S = 0.1
-- A circle is flown this long, then read over two windows so that its steadiness shows.
local CIRCLE_S = 8
local WINDOW_S = 2
-- The nose is closed on its heading error with this time constant, in seconds.
local NOSE_TIME_CONSTANT_S = 0.1
-- A corner is over when the velocity is within this angle of the new heading (a velocity error
-- decays with the drag, and the position it adds up to is read once it is this small).
local ALIGNED = math.rad(0.05)
-- A bound that only turns a ship that never realigns into a failure.
local CORNER_MAX_S = 60
-- What the checks allow: the circle's rate against the model, its speed share, the two windows
-- of a circle against each other, and a corner's times at one multiplier against another.
local RATE_TOLERANCE        = 0.08
local SPEED_RATIO_TOLERANCE = 0.02
local STEADY_TOLERANCE      = 0.02
local SCALE_TOLERANCE       = 0.05
local KM = 1000

local Engage = Enums.TravelDriveRequest.Engage

local function vec(v) return { x = v.x, y = v.y, z = v.z } end
local function dot(a, b) return a.x * b.x + a.y * b.y + a.z * b.z end
local function length(v) return math.sqrt(dot(v, v)) end
local function sub(a, b) return { x = a.x - b.x, y = a.y - b.y, z = a.z - b.z } end
local function add(a, b, s) return { x = a.x + b.x * s, y = a.y + b.y * s, z = a.z + b.z * s } end
local function scale(a, s) return { x = a.x * s, y = a.y * s, z = a.z * s } end
local function unit(a) return scale(a, 1 / length(a)) end

local function cross(a, b)
    return { x = a.y * b.z - a.z * b.y, y = a.z * b.x - a.x * b.z, z = a.x * b.y - a.y * b.x }
end

---The angle between two vectors, 0 to pi, accurate for small angles.
local function angleBetween(a, b)
    return math.atan2(length(cross(a, b)), dot(a, b))
end

---Game units as kilometres of the real world.
local function kilometres(gameUnits)
    return ScaleConfig:toRealMeters(gameUnits, "starSystem") / KM
end

---One simulation step in the order the apps use: drive, flight, physics.
function DriveTurnTest:step()
    TravelDriveSystem:update(STEP)
    ShipFlightSystem:update()
    self.world:update(STEP)
end

function DriveTurnTest:steps(seconds, each)
    for _ = 1, math.floor(seconds / STEP + 0.5) do
        self:step()
        if each then each() end
    end
end

function DriveTurnTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("ConstructsDriveTurnTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

function DriveTurnTest:despawn(ship)
    self.world:removeRigidBody(ship.rb)
    Registry:destroyEntity(ship.entity)
end

---A Solo ship flown to the drive's speed at `mult`, straight ahead.
---@return table|nil ship Nil when the drive has not reached its speed
function DriveTurnTest:spawnAtSpeed(mult)
    local ship = FlightScene.spawnShip(self.world, Position(0, 0, 0))
    ship.input = ship.entity:get(ConstructComponents.ControlInput)
    ship.profile = ConstructComponents.FlightProfile.ProfileOf(ship.entity)
    local cfg = Config.game.travelDrive
    ship.input:setDriveSpeedCap(mult)
    ship.input:setDriveRequest(Engage)
    self:steps(cfg.chargeRequired + 1 / cfg.rampUpSpeed + SETTLE_TIME_CONSTANTS / ship.profile.linearDrag)
    ship.speed0 = ship.rb:getSpeed()
    ship.expected = mult * ship.profile.maxSpeed
    local ok = math.abs(ship.speed0 - ship.expected) <= SPEED_TOLERANCE * ship.expected
    self:check(ok, string.format("the drive reaches %dx the top speed", mult),
        string.format("speed %.1f, expected %.1f", ship.speed0, ship.expected))
    if not ok then
        self:despawn(ship)
        return nil
    end
    return ship
end

---The ship's nose, steered to `target` (a unit vector) at the nose's own spin, with `yawToRight` the
---yaw input that turns the nose to its right.
function DriveTurnTest:steerNose(ship, target, nose)
    local rot = ship.rb:getRot()
    local error = math.atan2(dot(target, vec(rot:getRight())), dot(target, vec(rot:getForward())))
    local command = math.max(-1, math.min(1, error / (nose.spin * NOSE_TIME_CONSTANT_S)))
    ship.input:setYaw(nose.yawToRight * command)
end

---Full yaw for a moment: which yaw input turns the nose to its right, and how fast the nose spins
---under the drive's steering torque (rad/s).
function DriveTurnTest:measureNose(mult)
    local ship = self:spawnAtSpeed(mult)
    if not ship then return nil end
    local right0 = vec(ship.rb:getRot():getRight())
    local up = vec(ship.rb:getRot():getUp())
    ship.input:setYaw(1)
    self:steps(SPIN_S)
    local spin = dot(vec(ship.rb:getVelocityA()), up)
    -- The nose has turned to the right if its new heading has a component along the old right.
    local turnedRight = dot(vec(ship.rb:getRot():getForward()), right0) > 0
    Log.Info("ConstructsDriveTurnTest: nose at %dx under full yaw: spin %.3f rad/s, drag-limited rate would be %.3f rad/s",
        mult, math.abs(spin), ship.profile.linearDrag)
    self:despawn(ship)
    return { yawToRight = turnedRight and 1 or -1, spin = math.abs(spin) }
end

---A steady circle with the nose held `degrees` to the right of the velocity: the turn rate, the
---speed and the radius it gives, against the model.
function DriveTurnTest:measureLeadCircle(mult, degrees, nose)
    local ship = self:spawnAtSpeed(mult)
    if not ship then return end
    local lead = math.rad(degrees)
    local drag = ship.profile.linearDrag

    ---The nose's target: the velocity's heading turned to the right by the lead.
    local function target()
        local v = unit(vec(ship.rb:getVelocity()))
        local right = vec(ship.rb:getRot():getRight())
        local side = unit(sub(right, scale(v, dot(right, v))))
        return add(scale(v, math.cos(lead)), side, math.sin(lead))
    end
    local function steer() self:steerNose(ship, target(), nose) end

    self:steps(CIRCLE_S - 2 * WINDOW_S, steer)
    local rates, v0 = {}, vec(ship.rb:getVelocity())
    local speed, count, leadSum = 0, 0, 0
    for window = 1, 2 do
        self:steps(WINDOW_S, function()
            steer()
            if window == 2 then
                local v = vec(ship.rb:getVelocity())
                speed = speed + length(v)
                leadSum = leadSum + angleBetween(vec(ship.rb:getRot():getForward()), v)
                count = count + 1
            end
        end)
        local v1 = vec(ship.rb:getVelocity())
        rates[window] = angleBetween(v0, v1) / WINDOW_S
        v0 = v1
    end
    speed, leadSum = speed / count, leadSum / count

    Log.Info("ConstructsDriveTurnTest: circle at %dx, lead %d deg (held %.1f): rate %.3f rad/s (model %.3f; window before %.3f), "
        .. "speed %.3f of straight (model %.3f), radius %.0f km (model %.0f)",
        mult, degrees, math.deg(leadSum), rates[2], drag * math.tan(lead), rates[1], speed / ship.speed0,
        math.cos(lead), kilometres(speed / rates[2]), kilometres(ship.speed0 * math.cos(lead) / (drag * math.tan(lead))))

    -- The model is held to the lead the nose really had (it lags the velocity's turn a little).
    local what = string.format("%dx, lead %d deg", mult, degrees)
    local modelRate = drag * math.tan(leadSum)
    self:check(math.abs(rates[2] - modelRate) <= RATE_TOLERANCE * modelRate,
        "the turn rate is the drag times the tangent of the lead at " .. what, string.format("%.3f against %.3f", rates[2], modelRate))
    self:check(math.abs(speed / ship.speed0 - math.cos(leadSum)) <= SPEED_RATIO_TOLERANCE,
        "the speed is the cosine of the lead of the straight speed at " .. what,
        string.format("%.3f against %.3f", speed / ship.speed0, math.cos(leadSum)))
    self:check(math.abs(rates[2] - rates[1]) <= STEADY_TOLERANCE * rates[2], "the circle is steady at " .. what)
    self:despawn(ship)
end

---A corner of `degrees` to the right: the nose is turned straight to the new heading. Logs the time
---to realign, the lowest speed, and how far the ship overshoots the corner.
function DriveTurnTest:measureCorner(mult, degrees, nose)
    local ship = self:spawnAtSpeed(mult)
    if not ship then return end
    local theta = math.rad(degrees)
    local right = vec(ship.rb:getRot():getRight())
    local d0 = scale(vec(ship.rb:getVelocity()), 1 / ship.speed0)
    local d1 = add(scale(d0, math.cos(theta)), right, math.sin(theta))
    local corner = vec(ship.rb:getPos())
    local minSpeed, time, aligned = ship.speed0, 0, false

    while time < CORNER_MAX_S do
        self:steerNose(ship, d1, nose)
        self:step()
        time = time + STEP
        local v = vec(ship.rb:getVelocity())
        minSpeed = math.min(minSpeed, length(v))
        if angleBetween(v, d1) < ALIGNED then
            aligned = true
            break
        end
    end

    self:check(aligned, string.format("a corner of %d deg at %dx realigns", degrees, mult))
    if not aligned then
        self:despawn(ship)
        return
    end

    -- The overshoot: how far the ship's line lies beyond the new leg through the corner, on the side
    -- the old heading points to, and the flight time and distance by which the turn had to start earlier.
    local outward = unit(sub(d0, scale(d1, dot(d0, d1))))
    local offset = dot(sub(vec(ship.rb:getPos()), corner), outward)
    local early = offset / math.sin(theta)
    Log.Info("ConstructsDriveTurnTest: corner %d deg at %dx: realigned in %.2f s, lowest speed %.3f of straight, "
        .. "overshoot %.0f km, start the turn %.0f km (%.2f s of flight) early",
        degrees, mult, time, minSpeed / ship.speed0, kilometres(offset), kilometres(early), early / ship.speed0)

    -- A corner is the same in time at every multiplier: the speed scales the distances only.
    local reference = self.corners[degrees]
    if not reference then
        self.corners[degrees] = { time = time, early = early / ship.speed0 }
    else
        self:check(math.abs(time - reference.time) <= SCALE_TOLERANCE * reference.time
            and math.abs(early / ship.speed0 - reference.early) <= SCALE_TOLERANCE * reference.early,
            string.format("a corner of %d deg takes the same time and flight time early at %dx", degrees, mult))
    end
    self:despawn(ship)
end

function DriveTurnTest:onInit()
    self.passed, self.failed = 0, 0
    self.corners = {}
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    for _, mult in ipairs(MULTIPLIERS) do
        local nose = self:measureNose(mult)
        if nose then
            for _, degrees in ipairs(LEAD_DEGREES) do
                self:measureLeadCircle(mult, degrees, nose)
            end
            for _, degrees in ipairs(CORNER_DEGREES) do
                self:measureCorner(mult, degrees, nose)
            end
        end
    end

    Log.Info("ConstructsDriveTurnTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return DriveTurnTest
