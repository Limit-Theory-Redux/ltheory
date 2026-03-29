local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ShipActions         = require("Input.ActionBindings.ShipActions")

--- Ship flight system — direct port of Legacy ThrustController into ECS.
--- Raw input -> forces/torques each frame. Physics handles the rest.
--- Per-ship FPS aiming state stored in ShipFlightControlComponent.
---@class ShipFlightSystem
local ShipFlightSystem = {}

--- Update ship from input.
---@param dt number
---@param shipEntity Entity
---@param mouseAim boolean Mouse controls ship rotation
---@param useDelta boolean Use mouse delta (FPS) vs screen-relative (Chase)
function ShipFlightSystem:update(dt, shipEntity, mouseAim, useDelta)
    if not shipEntity then return end

    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not rbCmp then return end
    local rb = rbCmp:getRigidBody()
    if not rb then return end

    local cfg = Config.game.shipFlight

    -- Read thrust input
    local forward = ShipActions.ThrustZ:get()
    local right   = ShipActions.ThrustX:get()
    local up      = ShipActions.ThrustY:get()
    local roll    = ShipActions.Roll:get()
    local boost   = ShipActions.Boost:get()

    -- Disable boost during travel drive
    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    if drive and drive:isActive() then
        boost = 0
    end

    -- Read rotation input
    local yaw   = 0
    local pitch = 0
    if mouseAim then
        if useDelta then
            -- FPS mode: indirect aiming with accumulation and decay
            local flightCtrl = shipEntity:get(ConstructComponents.ShipFlightControl)
            if flightCtrl then
                local delta = Input:mouse():delta()
                local sx = Window:width()
                local sy = Window:height()
                local sens = 1.0

                local tYaw = Math.Clamp(
                    flightCtrl:getTargetYaw() + delta.x / sx * sens, -0.8, 0.8)
                local tPitch = Math.Clamp(
                    flightCtrl:getTargetPitch() - delta.y / sy * sens, -0.8, 0.8)

                yaw   = tYaw
                pitch = tPitch

                -- Decay toward center when mouse stops
                flightCtrl:setTargetYaw(tYaw * (1.0 - 4.0 * dt))
                flightCtrl:setTargetPitch(tPitch * (1.0 - 4.0 * dt))
            end
        else
            -- Chase mode: screen-relative mouse position (legacy style)
            yaw   = ShipActions.Yaw:get()
            pitch = ShipActions.Pitch:get()
        end
        -- Dead zones
        if math.abs(yaw)   < 0.004 then yaw   = 0 end
        if math.abs(pitch) < 0.008 then pitch = 0 end

        -- Config inversion
        if cfg.invertYaw   then yaw   = -yaw end
        if cfg.invertPitch then pitch = -pitch end
    end

    -- Boost (matches legacy: 1.0 + 2.0 * boost)
    local mult = 1.0 + 2.0 * boost

    -- Ship orientation
    local rot = rb:getRot()

    -- Apply translational forces
    if math.abs(forward) > 1e-6 then
        rb:applyForce(rot:getForward():scale(forward * cfg.thrustForward * mult))
    end
    if math.abs(right) > 1e-6 then
        rb:applyForce(rot:getRight():scale(right * cfg.thrustRight * 0.5))
    end
    if math.abs(up) > 1e-6 then
        rb:applyForce(rot:getUp():scale(up * cfg.thrustUp))
    end

    -- Apply torque in local space
    if math.max(math.max(math.abs(pitch), math.abs(yaw)), math.abs(roll)) > 1e-6 then
        local localTorque = Vec3f(
            pitch * cfg.torquePitch,
            -yaw  * cfg.torqueYaw,
            -roll * cfg.torqueRoll * 0.5
        )
        rb:applyTorque(rot:mulV(localTorque))
    end
end

--- Reduced steering only (for travel drive — no thrust, slower rotation)
---@param dt number
---@param shipEntity Entity
---@param mouseAim boolean
function ShipFlightSystem:updateSteeringOnly(dt, shipEntity, mouseAim)
    if not shipEntity then return end

    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not rbCmp then return end
    local rb = rbCmp:getRigidBody()
    if not rb then return end

    local cfg = Config.game.shipFlight
    local roll = ShipActions.Roll:get()
    local yaw, pitch = 0, 0
    if mouseAim then
        local delta = Input:mouse():delta()
        local sx, sy = Window:width(), Window:height()
        yaw   = Math.Clamp(delta.x / sx * 4.0, -1.0, 1.0)
        pitch = Math.Clamp(-delta.y / sy * 4.0, -1.0, 1.0)
        if math.abs(yaw)   < 0.0001 then yaw   = 0 end
        if math.abs(pitch) < 0.0001 then pitch = 0 end
    end

    -- Reduced torque at high speed (10% of normal)
    local steerFactor = 0.1
    local rot = rb:getRot()
    if math.max(math.max(math.abs(pitch), math.abs(yaw)), math.abs(roll)) > 1e-6 then
        local localTorque = Vec3f(
            pitch * cfg.torquePitch * steerFactor,
            -yaw  * cfg.torqueYaw * steerFactor,
            -roll * cfg.torqueRoll * 0.5 * steerFactor
        )
        rb:applyTorque(rot:mulV(localTorque))
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
