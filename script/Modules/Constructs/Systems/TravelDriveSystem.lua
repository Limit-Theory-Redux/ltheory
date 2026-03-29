local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ShipActions         = require("Input.ActionBindings.ShipActions")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")

--- Travel Drive System — provides high-speed interplanetary travel.
--- Operates on the TravelDriveComponent attached to ship entities.
---
--- States: Idle -> Charging -> Active -> Decelerating -> Idle
---@class TravelDriveSystem
local TravelDriveSystem = {}

--- Update the travel drive for a ship entity
---@param dt number Delta time
---@param shipEntity Entity
function TravelDriveSystem:update(dt, shipEntity)
    if not shipEntity then return end

    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    if not drive then return end

    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not rbCmp then return end
    local rb = rbCmp:getRigidBody()
    if not rb then return end

    local cfg = Config.game.travelDrive
    local state = drive:getState()

    -- Toggle travel drive with dedicated key (T)
    if ShipActions.TravelDrive:isPressed() then
        if state == "idle" then
            drive:setState("charging")
            drive:setChargeTime(0)
            state = "charging"
        elseif state == "active" or state == "charging" then
            drive:setState("decelerating")
            state = "decelerating"
        end
    end

    local currentMult = drive:getCurrentMult()

    if state == "idle" then
        currentMult = math.max(1.0, currentMult - cfg.rampDownSpeed * dt)

    elseif state == "charging" then
        local chargeTime = drive:getChargeTime() + dt
        drive:setChargeTime(chargeTime)
        if chargeTime >= cfg.chargeRequired then
            drive:setState("active")
            drive:setActiveTime(0)
        end
        currentMult = math.max(1.0, currentMult - cfg.rampDownSpeed * dt)

    elseif state == "active" then
        drive:setActiveTime(drive:getActiveTime() + dt)

        -- Zone-based speed cap
        local maxSpeed = GravityWellSystem:getMaxDriveSpeed()

        if currentMult > maxSpeed then
            currentMult = math.max(maxSpeed,
                currentMult - (currentMult - maxSpeed) * cfg.rampDownSpeed * dt)
        else
            currentMult = math.min(maxSpeed,
                currentMult + (maxSpeed - 1) * cfg.rampUpSpeed * dt)
        end

        -- Apply forward thrust
        local rot = rb:getRot()
        local fwd = rot:getForward()
        local thrustForward = Config.game.shipFlight.thrustForward
        rb:applyForce(fwd:scale(thrustForward * currentMult))

    elseif state == "decelerating" then
        currentMult = math.max(1.0, currentMult - (cfg.speedMultiplier - 1) * cfg.rampDownSpeed * dt)
        if currentMult <= 1.1 then
            drive:setState("idle")
            currentMult = 1.0
        end
    end

    drive:setCurrentMult(currentMult)

    -- Weapon fire cancels the drive
    if ShipActions.Fire and ShipActions.Fire:get() > 0.5 then
        state = drive:getState()
        if state == "active" or state == "charging" then
            drive:setState("decelerating")
        end
    end
end

return TravelDriveSystem
