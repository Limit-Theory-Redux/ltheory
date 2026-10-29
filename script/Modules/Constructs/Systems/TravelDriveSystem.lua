local Registry            = require("Core.ECS.Registry")
local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")

-- A decelerating drive goes idle once its multiplier falls to this.
local IDLE_MULT = 1.1

--- Travel Drive System — provides high-speed interplanetary travel.
--- Operates on every ship with a TravelDriveComponent; the drive is requested through
--- the ship's ControlInputComponent and the system reads no input device.
---
--- States: Idle -> Charging -> Active -> Decelerating -> Idle
---@class TravelDriveSystem
local TravelDriveSystem = {}

--- Take the ship's pending drive request and apply it to the drive state.
---@param drive TravelDriveComponent
---@param input ControlInputComponent
local function consumeRequest(drive, input)
    local request = input:getDriveRequest()
    if request == Enums.TravelDriveRequest.None then return end
    input:setDriveRequest(Enums.TravelDriveRequest.None)

    local state = drive:getState()
    if request == Enums.TravelDriveRequest.Engage then
        if state == "idle" then
            drive:setState("charging")
            drive:setChargeTime(0)
        end
    elseif state == "active" or state == "charging" then
        drive:setState("decelerating")
    end
end

--- The drive multiplier to approach while active: the zone cap, lowered by the ship's cap.
---@param input ControlInputComponent|nil
---@return number
local function targetMult(input)
    local target = GravityWellSystem:getMaxDriveSpeed()
    local cap = input and input:getDriveSpeedCap()
    if cap then
        target = math.min(target, math.max(1.0, cap))
    end
    return target
end

--- Advance one ship's drive.
---@param dt number
---@param drive TravelDriveComponent
---@param input ControlInputComponent|nil
---@param rb RigidBody
---@param profile FlightProfile
local function updateDrive(dt, drive, input, rb, profile)
    local cfg = Config.game.travelDrive

    if input then
        consumeRequest(drive, input)
    end

    local state = drive:getState()
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

        local target = targetMult(input)
        if currentMult > target then
            -- Exponential approach, exact for any step length.
            currentMult = math.max(target, Math.EMA(currentMult, target, dt, 1.0 / cfg.rampDownSpeed))
        else
            currentMult = math.min(target, currentMult + (target - 1) * cfg.rampUpSpeed * dt)
        end

        -- Apply forward thrust
        local fwd = rb:getRot():getForward()
        rb:applyForce(fwd:scale(profile.thrustForward * currentMult))

    elseif state == "decelerating" then
        currentMult = math.max(1.0, currentMult - (cfg.speedMultiplier - 1) * cfg.rampDownSpeed * dt)
        if currentMult <= IDLE_MULT then
            drive:setState("idle")
            currentMult = 1.0
        end
    end

    drive:setCurrentMult(currentMult)
end

--- Advance the drive of every ship that has one.
---@param dt number Delta time
function TravelDriveSystem:update(dt)
    for shipEntity, drive in Registry:iterEntities(ConstructComponents.TravelDrive) do
        local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
        local rb = rbCmp and rbCmp:getRigidBody()
        if rb then
            local profile = ConstructComponents.FlightProfile.ProfileOf(shipEntity)
            updateDrive(dt, drive, shipEntity:get(ConstructComponents.ControlInput), rb, profile)
        end
    end
end

return TravelDriveSystem
