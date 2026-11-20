---Flight profile definitions: one per ship class.
---Speeds are authored in real m/s and compressed to game units once, here.
---Thrust is derived so that forward thrust settles at the maximum speed against
---linear drag. Registers into FlightProfileRegistry.
---Loaded from Config/General/GameConfig.lua during config load.

local FlightProfile = require("Shared.Types.FlightProfile")
local ScaleConfig   = require("Config.Gen.UniverseScaleConfig")

---@param mass number kg
---@param linearDrag number 1/s
---@param maxSpeedReal number m/s
---@return number Force in gameunits that holds `maxSpeedReal` against the drag
local function forwardThrust(mass, linearDrag, maxSpeedReal)
    return ScaleConfig:toGameUnits(maxSpeedReal, "starSystem") * mass * linearDrag
end

-- Solo fighter (~50 m hull).
local SOLO_MAX_SPEED   = 8800 -- m/s
local SOLO_MASS        = 5000
local SOLO_LINEAR_DRAG = 2.0
local SOLO_ANGULAR_DRAG = 20.0
-- Torque is tuned by feel. The equilibrium turn rate is torque / (inertia * angularDrag),
-- with inertia tiny at small globalScale: angularDrag dominates the feel, torque sets
-- responsiveness.
local SOLO_TORQUE      = 50
-- Strafing and vertical thrust are fractions of the forward thrust.
local SOLO_RIGHT_RATIO = 0.25
local SOLO_UP_RATIO    = 0.25
-- Roll torque relative to pitch and yaw.
local SOLO_ROLL_RATIO  = 0.3

local soloThrust = forwardThrust(SOLO_MASS, SOLO_LINEAR_DRAG, SOLO_MAX_SPEED)

FlightProfile {
    id            = Enums.FlightProfile.Solo,
    name          = "Solo",
    mass          = SOLO_MASS,
    linearDrag    = SOLO_LINEAR_DRAG,
    angularDrag   = SOLO_ANGULAR_DRAG,
    thrustForward = soloThrust,
    thrustRight   = soloThrust * SOLO_RIGHT_RATIO,
    thrustUp      = soloThrust * SOLO_UP_RATIO,
    torquePitch   = SOLO_TORQUE,
    torqueYaw     = SOLO_TORQUE,
    torqueRoll    = SOLO_TORQUE * SOLO_ROLL_RATIO,
    maxSpeed      = ScaleConfig:toGameUnits(SOLO_MAX_SPEED, "starSystem"),
}
