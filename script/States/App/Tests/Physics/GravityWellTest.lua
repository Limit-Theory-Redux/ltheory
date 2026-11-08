-- Checks the gravity wells with several ships: each ship with a gravity-well component gets its own
-- zone, carried with the body it is in, and its own travel-drive cap; ships without the component are
-- left alone. Logs each failed check and ends with a failure exit code.

local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local Entity                = require("Core.ECS.Entity")
local PhysicsComponents     = require("Modules.Physics.Components")
local SpatialComponents     = require("Modules.Spatial.Components")
local SpatialEntities       = require("Modules.Spatial.Entities")
local CelestialComponents   = require("Modules.CelestialObjects.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local GravityWellSystem     = require("Modules.Physics.Systems.GravityWellSystem")
local TravelDriveSystem     = require("Modules.Constructs.Systems.TravelDriveSystem")
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")
local FlightProfileHelper   = require("Shared.Helpers.FlightProfileHelper")

---@class PhysicsGravityWellTest: Application
local GravityWellTest = Subclass("PhysicsGravityWellTest", Application)

local STEP            = 1 / 60
-- Safety limit for the drive to reach its cap; the ramp takes a few seconds.
local MAX_DRIVE_STEPS = 60 * 60
local TOLERANCE       = 1e-6
-- Velocities are read back in single precision, which resolves a speed of a hundred only to about 1e-5.
local VELOCITY_TOLERANCE = 1e-3
-- Frames for a change of frame to settle: the carry's leftover velocity falls by half or more each frame.
local CARRY_STEPS     = 40
-- A body move in one frame that is a jump and not a flight.
local JUMP            = 5000

-- Positions on the x axis, in game units.
local STAR_X, STAR_ZONE     = 0, 100
local PLANET_X, PLANET_ZONE = 1000, 40
local MOON_X, MOON_ZONE     = 1010, 5
local IN_MOON_X             = 1008 -- inside the moon's well and the planet's
local IN_PLANET_X           = 1030 -- inside the planet's well only
local OPEN_X                = 5000
local PLANET_MOVE           = 3

local function near(actual, expected)
    return math.abs(actual - expected) <= TOLERANCE
end

function GravityWellTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("PhysicsGravityWellTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

---A sphere at `x` on the x axis, owned by `entity`: kinematic, or a dynamic one with the drag of
---`profile` when it is given.
function GravityWellTest:giveBody(entity, x, profile)
    local rbCmp = entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateSphere()
    if profile then
        FlightProfileHelper.applyToRigidBody(profile, rb)
        rb:setCollidable(false) -- the ships share positions with other bodies, and only the carry is measured
    else
        rb:setKinematic(true)
    end
    rbCmp:setRigidBody(rb)
    self.world:addRigidBody(rb)
    rb:setPos(Position(x, 0, 0))
    return rb
end

---A body named `name` and marked as `kind` (a kind marker component type), with a spherical zone
---of `zoneRadius`, a child of `parent` when given.
function GravityWellTest:spawnBody(name, kind, x, zoneRadius, parent)
    local body = Entity.Create(name, kind())
    local rb = self:giveBody(body, x)
    local zone = SpatialEntities.Zone()
    zone:get(SpatialComponents.Shape):setRadius(zoneRadius)
    Registry:attachEntity(body, zone)
    if parent then Registry:attachEntity(parent, body) end
    return body, rb
end

---A ship at `x`; with a gravity-well component unless `withWell` is false. It has a travel drive
---and a flight profile so that the drive system can run it. It is a kinematic body, or a dynamic
---one when `dynamic` is set.
function GravityWellTest:spawnShip(x, withWell, dynamic)
    local ship = Registry:createEntity()
    local profile = FlightProfileRegistry:get(Enums.FlightProfile.Solo)
    local rb = self:giveBody(ship, x, dynamic and profile or nil)
    local well = withWell ~= false and ship:add(PhysicsComponents.GravityWell(profile.linearDrag)) or nil
    ship:add(ConstructComponents.FlightProfile(profile))
    local drive = ship:add(ConstructComponents.TravelDrive())
    ship:add(ConstructComponents.ControlInput())
    return { entity = ship, rb = rb, well = well, drive = drive }
end

function GravityWellTest:buildScene()
    local star = self:spawnBody("StarEntity", CelestialComponents.Star, STAR_X, STAR_ZONE)
    local planet, planetRb = self:spawnBody("PlanetEntity", CelestialComponents.Planet, PLANET_X, PLANET_ZONE, star)
    self:spawnBody("MoonEntity", CelestialComponents.Moon, MOON_X, MOON_ZONE, planet)
    self.planetRb = planetRb
    GravityWellSystem:collectZones()
    self.world:update(STEP)
end

---Runs the wells for one frame of the physics step.
function GravityWellTest:updateWells()
    GravityWellSystem:update(STEP, self.world:getSolverSubsteps())
end

function GravityWellTest:checkZones()
    self:check(#GravityWellSystem.zones == 3, "a star, a planet and a moon each have a well",
        "found " .. #GravityWellSystem.zones)

    local inMoon = self:spawnShip(IN_MOON_X)
    local inPlanet = self:spawnShip(IN_PLANET_X)
    local inOpen = self:spawnShip(OPEN_X)
    local noWell = self:spawnShip(IN_MOON_X, false)
    self:updateWells()

    self:check(inMoon.well:getZoneType() == "moon", "the innermost well wins", inMoon.well:getZoneType())
    self:check(inPlanet.well:getZoneType() == "planet", "a ship in a planet's well only is in the planet's",
        inPlanet.well:getZoneType())
    self:check(inOpen.well:getZoneType() == "openSpace", "a ship in no well is in open space")
    self:check(inOpen.well:getZoneName() == "Open Space", "open space has its own name")
    self:check(inMoon.well:getZoneName():find("(moon)", 1, true) ~= nil, "the zone name has the zone type",
        inMoon.well:getZoneName())
    self:check(noWell.entity:get(PhysicsComponents.GravityWell) == nil, "a ship without the component is left alone")

    inMoon.rb:setPos(Position(OPEN_X, 0, 0))
    self:updateWells()
    self:check(inMoon.well:getZoneType() == "openSpace", "a ship that left a well is in open space")
    inMoon.rb:setPos(Position(IN_MOON_X, 0, 0))
    self:updateWells()
    self:check(inMoon.well:getZoneType() == "moon", "a ship that came back is in the well again")

    self.ships = { inMoon = inMoon, inPlanet = inPlanet, inOpen = inOpen, noWell = noWell }
end

function GravityWellTest:checkDriveCaps()
    local cfg = Config.game.travelDrive.maxSpeedByZone
    local function capAt(x) return GravityWellSystem:getMaxDriveSpeedAt(Position(x, 0, 0)) end

    self:check(near(capAt(OPEN_X), cfg.openSpace), "far from every body the cap is the open-space cap")
    self:check(near(capAt(MOON_X), cfg.moon), "at a moon the cap is the moon's minimum")
    self:check(near(capAt(STAR_X), cfg.star), "at a star the cap is the star's minimum")

    -- Between the planet and the moon both bodies reduce the cap; the lower one applies.
    local x = 1015
    local planetT = (x - PLANET_X) / (PLANET_ZONE * 2)
    local expected = cfg.planet + (cfg.openSpace - cfg.planet) * planetT * planetT
    self:check(near(capAt(x), expected), "where influences overlap the lowest cap applies",
        capAt(x) .. " vs " .. expected)
end

---Step until `done()` holds; false if it still doesn't after `MAX_DRIVE_STEPS`.
function GravityWellTest:stepUntil(done)
    for _ = 1, MAX_DRIVE_STEPS do
        if done() then return true end
        TravelDriveSystem:update(STEP)
        self.world:update(STEP)
    end
    return done()
end

function GravityWellTest:checkDrivesUseOwnCaps()
    local cfg = Config.game.travelDrive.maxSpeedByZone
    local inMoon, inOpen = self.ships.inMoon, self.ships.inOpen
    inMoon.drive:setState("active")
    inOpen.drive:setState("active")

    -- The moon's influence reaches twice its zone radius; the cap rises with the square of the distance.
    local moonT = math.abs(IN_MOON_X - MOON_X) / (MOON_ZONE * 2)
    local moonCap = cfg.moon + (cfg.openSpace - cfg.moon) * moonT * moonT
    self:check(moonCap < cfg.planet and moonCap < cfg.openSpace, "the test ship's cap is set by the moon")

    local reached = self:stepUntil(function()
        return near(inMoon.drive:getCurrentMult(), moonCap) and near(inOpen.drive:getCurrentMult(), cfg.openSpace)
    end)
    self:check(reached, "two ships in different zones each reach their own cap",
        inMoon.drive:getCurrentMult() .. " and " .. inOpen.drive:getCurrentMult())
end

---One frame: the wells carry their ships, then the physics step runs.
function GravityWellTest:step()
    self:updateWells()
    self.world:update(STEP)
end

---Moves the planet to `PLANET_X + moved` (a move of `PLANET_MOVE` per step is a velocity to the
---system) and runs a frame.
function GravityWellTest:stepPlanetTo(moved)
    self.planetRb:setPos(Position(PLANET_X + moved, 0, 0))
    self:step()
end

---The ships' speed relative to the frame they are carried at, in game units per second.
local function relativeSpeed(ship)
    local v = ship.rb:getVelocity()
    local cx, cy, cz = ship.well:getCarriedVelocity()
    return math.sqrt((v.x - cx) ^ 2 + (v.y - cy) ^ 2 + (v.z - cz) ^ 2)
end

function GravityWellTest:checkCarry()
    local atRest = self:spawnShip(IN_PLANET_X, true, true)
    local inMoon = self:spawnShip(IN_MOON_X, true, true)
    self:step() -- the ships enter their wells
    local x0, moonX0 = atRest.rb:getPos().x, inMoon.rb:getPos().x

    local moved = PLANET_MOVE
    self:stepPlanetTo(moved)
    self:check(near(atRest.rb:getPos().x, x0 + PLANET_MOVE), "a ship in a well moves with the body",
        tostring(atRest.rb:getPos().x))
    self:check(relativeSpeed(atRest) <= VELOCITY_TOLERANCE, "a ship carried by a well has no speed relative to it",
        tostring(relativeSpeed(atRest)))
    self:check(near(inMoon.rb:getPos().x, moonX0), "a ship is carried only by the body of its own well")

    -- A ship that enters a well is not moved by what the body moved before.
    moved = moved + PLANET_MOVE
    self.planetRb:setPos(Position(PLANET_X + moved, 0, 0))
    local entering = self:spawnShip(IN_PLANET_X + moved, true, true)
    local enteringX = entering.rb:getPos().x
    self:step()
    self:check(near(entering.rb:getPos().x, enteringX), "entering a well does not move the ship",
        tostring(entering.rb:getPos().x - enteringX))
    moved = moved + PLANET_MOVE
    self:stepPlanetTo(moved)
    self:check(near(entering.rb:getPos().x, enteringX + PLANET_MOVE), "the ship moves with the body from its second frame",
        tostring(entering.rb:getPos().x - enteringX))

    -- A ship that goes from the planet's well into the moon's, which stands still, stops moving with the planet.
    for _ = 1, CARRY_STEPS do
        moved = moved + PLANET_MOVE
        self:stepPlanetTo(moved)
    end
    entering.rb:setPos(Position(MOON_X, 0, 0))
    local bridgedX = entering.rb:getPos().x
    for _ = 1, CARRY_STEPS do self:step() end
    self:check(entering.well:getZoneType() == "moon", "the ship is in the moon's well", entering.well:getZoneType())
    self:check(near(entering.rb:getPos().x, bridgedX), "a ship that switches to a still body's well stays with it",
        tostring(entering.rb:getPos().x - bridgedX))

    -- A ship that leaves a well is not carried any more.
    atRest.rb:setPos(Position(OPEN_X, 0, 0))
    for _ = 1, CARRY_STEPS do self:step() end
    self:check(atRest.well:getZoneType() == "openSpace", "the ship is in open space")
    self:check(atRest.rb:getSpeed() <= VELOCITY_TOLERANCE, "a ship in open space is not carried",
        tostring(atRest.rb:getSpeed()))

    -- A body that jumps (a rebuilt scene) is not followed as if it had flown there.
    local jumper = self:spawnShip(IN_PLANET_X + moved, true, true)
    self:step()
    self.planetRb:setPos(Position(PLANET_X + moved + JUMP, 0, 0))
    jumper.rb:setPos(Position(IN_PLANET_X + moved + JUMP, 0, 0))
    local jumpedX = jumper.rb:getPos().x
    self:step()
    self:check(near(jumper.rb:getPos().x, jumpedX), "a body that jumps does not throw its ship",
        tostring(jumper.rb:getPos().x - jumpedX))
end

function GravityWellTest:onInit()
    self.passed, self.failed = 0, 0

    self.world = Physics.Create()
    self:buildScene()
    self:checkZones()
    self:checkDriveCaps()
    self:checkDrivesUseOwnCaps()
    self:checkCarry()

    Log.Info("PhysicsGravityWellTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return GravityWellTest
