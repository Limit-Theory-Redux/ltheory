-- Checks that the autopilot flies a ship through its control input: it arrives at a point and at
-- a body, with and without the travel drive, stops outside the body's hull, and leaves no axes
-- behind. Logs each failed check and ends with a failure exit code.
local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local Entity                = require("Core.ECS.Entity")
local PhysicsComponents     = require("Modules.Physics.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local AutoPilotSystem       = require("Modules.Constructs.Systems.AutoPilotSystem")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local FlightProfileHelper   = require("Shared.Helpers.FlightProfileHelper")

---@class ConstructsAutoPilotTest: Application
local AutoPilotTest = Subclass("ConstructsAutoPilotTest", Application)

local STEP         = 1 / 60
local HULL         = Enums.ShipHulls.Solo
local SOLO         = Enums.FlightProfile.Solo
-- Bound on the time a ship takes to stop once the autopilot has let go: it turns a ship that never
-- stops into a failure.
local MAX_SETTLE_S = 120
-- The ETA at the moment the drive runs may differ from the flight that remains by this share. It
-- leaves out the drive's ramp-up, so it runs short.
local ETA_TOLERANCE = 0.2
-- The ship has stopped below this speed, in game units per second.
local STOPPED      = 1e-3
-- Where the targets are: mostly behind the ship and to one side, so that it has to turn.
local DIRECTION    = { x = 0.6, y = 0.3, z = 0.74 }
local BODY_RADIUS  = 100

---@class AutoPilotScenario
---@field name string
---@field distance gameunit Distance of the target from the ship's start
---@field radius gameunit|nil Radius of a target body; nil for a bare point
---@field drive boolean Whether the travel drive is expected to engage
---@field maxFlight number Seconds of simulated time the autopilot may take to arrive: about 1.2 times what it takes

---@type AutoPilotScenario[]
local SCENARIOS = {
    { name = "a point, no drive",  distance = 300,   drive = false, maxFlight = 300 },
    { name = "a body, no drive",   distance = 600,   radius = BODY_RADIUS, drive = false, maxFlight = 570 },
    { name = "a body, with drive", distance = 20000, radius = BODY_RADIUS, drive = true,  maxFlight = 40 },
}

function AutoPilotTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("ConstructsAutoPilotTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---A ship set up like the player ship, at the origin, with the autopilot component.
function AutoPilotTest:spawnShip()
    local profile = FlightProfileRegistry:get(SOLO)
    local entity = Registry:createEntity()
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    rb:setScale(Config.game.shipHulls.scale[HULL])
    FlightProfileHelper.applyToRigidBody(profile, rb)
    entity:add(ConstructComponents.FlightProfile(profile))
    rbCmp:setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(Position(0, 0, 0))
    entity:add(ConstructComponents.TravelDrive())
    entity:add(ConstructComponents.ControlInput())
    entity:add(ConstructComponents.AutoPilot())
    return { entity = entity, rb = rb }
end

---A fixed sphere of `radius`, the target of a body scenario.
function AutoPilotTest:spawnBody(position, radius)
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(position)
    rb:setScale(radius)
    local entity = Entity.Create("AutoPilotTestBody", PhysicsComponents.RigidBody(rb))
    self.world:addRigidBody(rb)
    return { entity = entity, rb = rb }
end

---Remove a scenario's bodies from physics and destroy their entities, so the actuation systems
---no longer see them.
function AutoPilotTest:despawn(ship, body)
    for _, thing in ipairs({ ship, body or false }) do
        if thing then
            self.world:removeRigidBody(thing.rb)
            Registry:destroyEntity(thing.entity)
        end
    end
end

---One simulation step in the order the apps use: the autopilot at PreSim, then the drive, flight and physics.
function AutoPilotTest:step(ship)
    AutoPilotSystem:update(STEP, ship.entity)
    TravelDriveSystem:update(STEP)
    ShipFlightSystem:update()
    self.world:update(STEP)
end

local function axesAreZero(input)
    return input:getThrustForward() == 0 and input:getThrustRight() == 0 and input:getThrustUp() == 0
        and input:getPitch() == 0 and input:getYaw() == 0 and input:getRoll() == 0
end

---Fly one scenario and check it.
---@param scenario AutoPilotScenario
function AutoPilotTest:run(scenario)
    local name = scenario.name .. ": "
    local length = math.sqrt(DIRECTION.x ^ 2 + DIRECTION.y ^ 2 + DIRECTION.z ^ 2)
    local targetPos = Position(
        DIRECTION.x / length * scenario.distance,
        DIRECTION.y / length * scenario.distance,
        DIRECTION.z / length * scenario.distance)

    local ship = self:spawnShip()
    local input = ship.entity:get(ConstructComponents.ControlInput)
    local drive = ship.entity:get(ConstructComponents.TravelDrive)
    local ap = ship.entity:get(ConstructComponents.AutoPilot)
    local body = scenario.radius and self:spawnBody(targetPos, scenario.radius)
    if body then
        AutoPilotSystem:engageEntity(ship.entity, self.world, body.entity)
    else
        AutoPilotSystem:engagePosition(ship.entity, self.world, targetPos)
    end
    local targetRadius = scenario.radius or 0
    local hull = ship.rb:getBoundingRadius()
    self:check(ap:isActive(), name .. "engaging activates the autopilot")

    -- The first update writes the axes of a ship that is far from its goal and not facing it.
    AutoPilotSystem:update(STEP, ship.entity)
    self:check(not axesAreZero(input), name .. "the autopilot writes the control input")

    local closest, driveSeen, arrivedAt = math.huge, false, nil
    local etaAtDrive, driveAt
    for stepIndex = 1, math.floor(scenario.maxFlight / STEP) do
        self:step(ship)
        closest = math.min(closest, ship.rb:getPos():distance(targetPos))
        if drive:isActive() and not driveSeen then
            driveSeen, etaAtDrive, driveAt = true, ap.eta, stepIndex * STEP
        end
        if not ap:isActive() then
            arrivedAt = stepIndex * STEP
            break
        end
    end
    self:check(arrivedAt ~= nil, name .. "the autopilot arrives", string.format("%.0f s", scenario.maxFlight))
    if not arrivedAt then
        self:despawn(ship, body)
        return
    end
    if etaAtDrive then
        local left = arrivedAt - driveAt
        self:check(math.abs(etaAtDrive - left) <= ETA_TOLERANCE * left, name .. "the ETA matches the flight",
            string.format("eta %.1f s, left %.1f s", etaAtDrive, left))
    end

    local standOff = AutoPilotSystem.standOff(targetRadius)
    local arrivalRadius = AutoPilotSystem.arrivalRadius(targetRadius)
    local distance = ship.rb:getPos():distance(targetPos)
    self:check(math.abs(distance - standOff) <= arrivalRadius + hull, name .. "it arrives at the stand-off distance",
        string.format("%.1f against %.1f ± %.1f", distance, standOff, arrivalRadius))
    self:check(axesAreZero(input), name .. "disengaging leaves no axes")
    self:check(driveSeen == scenario.drive, name .. "the travel drive engages only when it should",
        string.format("drive seen %s", tostring(driveSeen)))

    -- It stops outside the hull, once the drive and the drag have run out.
    for _ = 1, math.floor(MAX_SETTLE_S / STEP) do
        if ship.rb:getSpeed() < STOPPED and drive:getState() == "idle" then break end
        self:step(ship)
        closest = math.min(closest, ship.rb:getPos():distance(targetPos))
    end
    self:check(ship.rb:getSpeed() < STOPPED, name .. "the ship comes to rest", tostring(ship.rb:getSpeed()))
    self:check(drive:getState() == "idle", name .. "the drive is idle", drive:getState())
    self:check(closest > targetRadius + hull, name .. "the ship stays outside the target's hull",
        string.format("closest %.1f, hull %.1f", closest, targetRadius + hull))
    Log.Info("ConstructsAutoPilotTest: %sarrived after %.0f s at %.0f (stand-off %.0f), closest %.0f, drive %s, eta at drive %s",
        name, arrivedAt, distance, standOff, closest, tostring(driveSeen),
        etaAtDrive and string.format("%.1f s, left %.1f s", etaAtDrive, arrivedAt - driveAt) or "-")

    self:despawn(ship, body)
end

function AutoPilotTest:onInit()
    self.passed, self.failed = 0, 0

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    for _, scenario in ipairs(SCENARIOS) do
        self:run(scenario)
    end

    Log.Info("ConstructsAutoPilotTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return AutoPilotTest
