local Registry               = require("Core.ECS.Registry")
local PhysicsComponents      = require("Modules.Physics.Components")
local ConstructComponents    = require("Modules.Constructs.Components")
local FlightProfileComponent = ConstructComponents.FlightProfile

-- Flight-model gains, from the legacy thrust controller.
local BOOST_GAIN        = 2.0 -- boost 1 triples forward thrust
local DRIVE_STEER_GAIN  = 0.1 -- torque while the travel drive is active (no thrust)

--- Ship flight system — direct port of Legacy ThrustController into ECS.
--- Turns each ship's ControlInput into forces and torques; physics does the rest.
---@class ShipFlightSystem
local ShipFlightSystem = {}

---@param v number
---@param limit number
---@return number
local function clampAxis(v, limit)
    return Math.Clamp(v, -limit, limit)
end

--- Apply one ship's control input as forces and torques.
---@param shipEntity Entity
---@param input ControlInputComponent
---@param rb RigidBody
local function actuate(shipEntity, input, rb)
    local profile = FlightProfileComponent.ProfileOf(shipEntity)

    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    local driveActive = drive ~= nil and drive:isActive()

    local pitch = clampAxis(input:getPitch(), 1)
    local yaw   = clampAxis(input:getYaw(), 1)
    local roll  = clampAxis(input:getRoll(), 1)
    local rot   = rb:getRot()

    -- The drive provides the forward force: no thrust or boost on top of it.
    if not driveActive then
        local forward = clampAxis(input:getThrustForward(), 1)
        local right   = clampAxis(input:getThrustRight(), 1)
        local up      = clampAxis(input:getThrustUp(), 1)
        local boost   = Math.Clamp(input:getBoost(), 0, 1)

        if math.abs(forward) > 1e-6 then
            rb:applyForce(rot:getForward():scale(forward * profile.thrustForward * (1.0 + BOOST_GAIN * boost)))
        end
        if math.abs(right) > 1e-6 then
            rb:applyForce(rot:getRight():scale(right * profile.thrustRight))
        end
        if math.abs(up) > 1e-6 then
            rb:applyForce(rot:getUp():scale(up * profile.thrustUp))
        end
    end

    if math.max(math.abs(pitch), math.abs(yaw), math.abs(roll)) > 1e-6 then
        local gain = driveActive and DRIVE_STEER_GAIN or 1.0
        local localTorque = Vec3f(
            pitch * profile.torquePitch * gain,
            -yaw  * profile.torqueYaw * gain,
            -roll * profile.torqueRoll * gain
        )
        rb:applyTorque(rot:mulV(localTorque))
    end
end

--- Actuate every ship that has a control input and a rigid body.
function ShipFlightSystem:update()
    for shipEntity, input in Registry:iterEntities(ConstructComponents.ControlInput) do
        local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
        local rb = rbCmp and rbCmp:getRigidBody()
        if rb then
            actuate(shipEntity, input, rb)
        end
    end
end

---@param shipEntity Entity
---@return number
function ShipFlightSystem:getSpeed(shipEntity)
    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not rbCmp then return 0 end
    local rb = rbCmp:getRigidBody()
    if not rb then return 0 end
    return rb:getSpeed()
end

return ShipFlightSystem
