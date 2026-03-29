local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")

--- AutoPilot System — steers a ship toward a target entity or position.
--- Operates on the AutoPilotComponent attached to ship entities.
---@class AutoPilotSystem
local AutoPilotSystem = {}

--- Engage autopilot to fly toward an entity
---@param shipEntity Entity
---@param targetEntity Entity
---@param arrivalRange number|nil
function AutoPilotSystem:engageEntity(shipEntity, targetEntity, arrivalRange)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap then return end

    local cfg = Config.game.autoPilot or {}
    ap:setActive(true)
    ap:setTargetEntity(targetEntity)
    ap:setTargetPos(nil)
    ap:setArrivalRange(arrivalRange or cfg.arrivalRange or 500)
    ap:setElapsed(0)
    Log.Info("AutoPilot: engaged to %s", tostring(targetEntity))
end

--- Engage autopilot to fly toward a position
---@param shipEntity Entity
---@param pos Position
---@param arrivalRange number|nil
function AutoPilotSystem:engagePosition(shipEntity, pos, arrivalRange)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap then return end

    local cfg = Config.game.autoPilot or {}
    ap:setActive(true)
    ap:setTargetEntity(nil)
    ap:setTargetPos(pos)
    ap:setArrivalRange(arrivalRange or cfg.arrivalRange or 500)
    ap:setElapsed(0)
    Log.Info("AutoPilot: engaged to position")
end

--- Disengage autopilot
---@param shipEntity Entity
function AutoPilotSystem:disengage(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap then return end

    if ap:isActive() then
        Log.Info("AutoPilot: disengaged")
    end
    ap:setActive(false)
    ap:setTargetEntity(nil)
    ap:setTargetPos(nil)
    ap:setElapsed(0)

    -- Reset travel drive through component
    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    if drive then
        drive:setState("idle")
        drive:setCurrentMult(1.0)
        drive:setChargeTime(0)
    end
end

--- Is autopilot currently active for this ship?
---@param shipEntity Entity
---@return boolean
function AutoPilotSystem:isActive(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    return ap and ap:isActive() or false
end

--- Get target name for HUD display
---@param shipEntity Entity
---@return string
function AutoPilotSystem:getTargetName(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    return ap and ap:getTargetName() or ""
end

--- Get distance to autopilot target
---@param shipEntity Entity
---@return number distance, number speed
function AutoPilotSystem:getDistanceAndSpeed(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap or not ap:isActive() then return 0, 0 end

    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not rbCmp then return 0, 0 end
    local rb = rbCmp:getRigidBody()
    if not rb then return 0, 0 end

    local shipPos = rb:getPos()
    local speed = rb:getSpeed()

    -- Get target position
    local targetPos = ap:getTargetPos()
    local targetEntity = ap:getTargetEntity()
    if targetEntity then
        local tRbCmp = targetEntity:get(PhysicsComponents.RigidBody)
        if tRbCmp and tRbCmp:getRigidBody() then
            targetPos = tRbCmp:getRigidBody():getPos()
        else
            local tTransform = targetEntity:get(PhysicsComponents.Transform)
            if tTransform then targetPos = tTransform:getPos() end
        end
    end

    if not targetPos then return 0, speed end

    local dx = targetPos.x - shipPos.x
    local dy = targetPos.y - shipPos.y
    local dz = targetPos.z - shipPos.z
    return math.sqrt(dx * dx + dy * dy + dz * dz), speed
end

--- Exponential mapping for smooth control output (from legacy)
local function expMap(x)
    if x >= 0 then
        return 1.0 - math.exp(-x)
    else
        return -(1.0 - math.exp(x))
    end
end

--- Update autopilot — steer ship toward target
---@param dt number
---@param shipEntity Entity
function AutoPilotSystem:update(dt, shipEntity)
    if not shipEntity then return end

    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap or not ap:isActive() then return end

    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    if not rbCmp then self:disengage(shipEntity); return end
    local rb = rbCmp:getRigidBody()
    if not rb then self:disengage(shipEntity); return end

    -- Get target position
    local targetEntity = ap:getTargetEntity()
    local targetPos = ap:getTargetPos()
    if targetEntity then
        local tRbCmp = targetEntity:get(PhysicsComponents.RigidBody)
        if tRbCmp and tRbCmp:getRigidBody() then
            targetPos = tRbCmp:getRigidBody():getPos()
        else
            local tTransform = targetEntity:get(PhysicsComponents.Transform)
            if tTransform then targetPos = tTransform:getPos() end
        end
    end

    if not targetPos then self:disengage(shipEntity); return end

    -- Calculate course
    local shipPos = rb:getPos()
    local dx = targetPos.x - shipPos.x
    local dy = targetPos.y - shipPos.y
    local dz = targetPos.z - shipPos.z
    local dist = math.sqrt(dx * dx + dy * dy + dz * dz)

    -- Get target radius for safe distance
    local targetRadius = 0
    if targetEntity then
        local tRbCmp = targetEntity:get(PhysicsComponents.RigidBody)
        if tRbCmp and tRbCmp:getRigidBody() then
            targetRadius = tRbCmp:getRigidBody():getBoundingRadius()
        else
            local tTransform = targetEntity:get(PhysicsComponents.Transform)
            if tTransform then targetRadius = tTransform:getScale() end
        end
    end

    -- Safe arrival distance: 1.5x object radius, minimum 50 units
    local safeRange = math.max(50, targetRadius * 1.5)

    -- Arrival check
    if dist < safeRange then
        Log.Info("AutoPilot: arrived at destination (dist=%.0f, safe=%.0f)", dist, safeRange)
        self:disengage(shipEntity)
        return
    end

    local cfg = Config.game.shipFlight
    local vel = rb:getVelocity()
    local speed = rb:getSpeed()
    local rot = rb:getRot()
    local fwd = rot:getForward()
    local rt  = rot:getRight()
    local up  = rot:getUp()

    -- Get target body's velocity (if it's a moving body like a planet)
    local targetVelX, targetVelY, targetVelZ = 0, 0, 0
    if targetEntity then
        local tRbCmp = targetEntity:get(PhysicsComponents.RigidBody)
        if tRbCmp and tRbCmp:getRigidBody() then
            local tVel = tRbCmp:getRigidBody():getVelocity()
            targetVelX, targetVelY, targetVelZ = tVel.x, tVel.y, tVel.z
        end
    end

    -- Compute DESIRED VELOCITY
    local dirX, dirY, dirZ = dx / dist, dy / dist, dz / dist

    -- Desired speed: deceleration curve
    local maxDecel = cfg.thrustForward / rb:getMass()
    local brakingDist = math.max(0, dist - safeRange)
    local desiredApproachSpeed = math.min(
        math.sqrt(2.0 * maxDecel * brakingDist),
        dist * 0.5
    )

    -- Desired velocity = approach direction * approach speed + target's velocity (match orbit)
    local desVelX = dirX * desiredApproachSpeed + targetVelX
    local desVelY = dirY * desiredApproachSpeed + targetVelY
    local desVelZ = dirZ * desiredApproachSpeed + targetVelZ

    -- Velocity error
    local errX = desVelX - vel.x
    local errY = desVelY - vel.y
    local errZ = desVelZ - vel.z
    local errLen = math.sqrt(errX * errX + errY * errY + errZ * errZ)

    -- Apply force to correct velocity error
    if errLen > 0.1 then
        local corrX = errX / errLen
        local corrY = errY / errLen
        local corrZ = errZ / errLen
        local desiredDecel = math.min(errLen, speed * 0.5)
        local forceScale = desiredDecel * rb:getMass()
        rb:applyForce(Vec3f(corrX * forceScale, corrY * forceScale, corrZ * forceScale))
    end

    -- Steering: point ship toward desired velocity direction
    local desVelLen = math.sqrt(desVelX * desVelX + desVelY * desVelY + desVelZ * desVelZ)
    if desVelLen > 1 then
        local courseX = desVelX / desVelLen
        local courseY = desVelY / desVelLen
        local courseZ = desVelZ / desVelLen

        local crossX = fwd.y * courseZ - fwd.z * courseY
        local crossY = fwd.z * courseX - fwd.x * courseZ
        local crossZ = fwd.x * courseY - fwd.y * courseX

        local yawError   = Math.Clamp(-(up.x * crossX + up.y * crossY + up.z * crossZ) * 3, -1, 1)
        local pitchError = Math.Clamp((rt.x * crossX + rt.y * crossY + rt.z * crossZ) * 3, -1, 1)

        rb:applyTorque(rot:mulV(Vec3f(
            pitchError * cfg.torquePitch * 0.3,
            -yawError * cfg.torqueYaw * 0.3,
            0
        )))
    end

    -- Travel drive management — distance-based speed limiting
    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    -- Disengage travel drive at 10k km (1000 game units) from target
    local driveDisengageDist = math.max(safeRange * 5, 1000)
    if drive then
        if dist < driveDisengageDist and drive:isActive() then
            drive:setState("decelerating")
        end

        -- Smoothly cap drive multiplier based on distance to target
        if drive:isActive() and dist > driveDisengageDist then
            -- Max multiplier proportional to distance: ensures smooth decel
            local maxMult = math.max(1, dist * 0.005)
            if drive:getCurrentMult() > maxMult then
                drive:setCurrentMult(math.max(maxMult, drive:getCurrentMult() * 0.9))
            end
        end

        local autoPilotCfg = Config.game.autoPilot or {}
        local travelDriveDelay = autoPilotCfg.travelDriveDelay or 5.0
        local enableTravelDrive = autoPilotCfg.enableTravelDrive ~= false

        local elapsed = ap:getElapsed() + dt
        ap:setElapsed(elapsed)

        if enableTravelDrive and elapsed > travelDriveDelay
           and dist > driveDisengageDist * 2 then
            if not drive:isActive() and drive:getState() == "idle" then
                drive:setState("charging")
                drive:setChargeTime(0)
            end
        end
    else
        ap:setElapsed(ap:getElapsed() + dt)
    end
end

return AutoPilotSystem
