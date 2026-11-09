local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local NavAgentHelper      = require("Shared.Helpers.NavAgentHelper")
local TravelDriveMath     = require("Shared.Helpers.TravelDriveMath")

-- Below this drive multiplier the drive is released and the NavWorld flies the rest.
local HANDOVER_MULT = TravelDriveMath.HANDOVER_MULT

-- The ship has arrived when it is within this share of the gap between the target's hull and the
-- stand-off distance from the stand-off point, so a ship that coasts in afterwards stops outside the hull.
local ARRIVAL_SHARE = 0.5
-- The stand-off distance is this many times the target's radius, and at least `MIN_STAND_OFF`.
local STAND_OFF_RADII = 1.5
local MIN_STAND_OFF = 50
-- The drive engages only beyond this many stand-off distances from the target.
local DRIVE_ENGAGE_STAND_OFFS = 100
-- The collision mask the NavWorld senses obstacles with: every collision group.
local SENSING_MASK = 0xFFFFFFFF

--- AutoPilot System — steers a ship toward a target entity or position.
--- Operates on the AutoPilotComponent attached to ship entities. It writes the ship's
--- ControlInput from a NavWorld that has the ship as its only agent; it applies no force itself.
--- The NavWorld senses the collidable bodies around the ship and steers around them. While the
--- travel drive runs the ship has no lateral thrust and turns weakly, so that avoidance only
--- acts in normal flight.
---@class AutoPilotSystem
local AutoPilotSystem = {}

--- The distance from a target's centre at which the autopilot stops the ship.
---@param targetRadius gameunit 0 for a point
---@return gameunit
function AutoPilotSystem.standOff(targetRadius)
    return math.max(MIN_STAND_OFF, targetRadius * STAND_OFF_RADII)
end

--- How far from its stand-off point the ship counts as arrived.
---@param targetRadius gameunit 0 for a point
---@return gameunit
function AutoPilotSystem.arrivalRadius(targetRadius)
    return ARRIVAL_SHARE * (AutoPilotSystem.standOff(targetRadius) - targetRadius)
end

--- Give the engagement its own NavWorld, with the ship as the only agent. The ship's body must stay
--- in `physics` while the autopilot is engaged: `NavWorld:step` panics if it has left.
---@param ap AutoPilotComponent
---@param shipEntity Entity
---@param physics Physics The world the ship's body is in
---@return boolean started False when the ship has no rigid body
local function startNavigation(ap, shipEntity, physics)
    local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    if not rb then
        Log.Warn("AutoPilot: the ship has no rigid body, not engaging")
        return false
    end
    local nav = NavWorld.Create(physics)
    nav:setSensing(SENSING_MASK)
    local agent = NavAgentHelper.addShip(nav, shipEntity, rb)
    ap:setNavigation(nav, agent, ffi.new("double[?]", NavWorld.AxisCount()), ffi.new("bool[1]"))
    return true
end

--- Engage autopilot to fly toward an entity
---@param shipEntity Entity
---@param physics Physics The world the ship's body is in
---@param targetEntity Entity
---@param arrivalRange number|nil
function AutoPilotSystem:engageEntity(shipEntity, physics, targetEntity, arrivalRange)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap then return end
    if not startNavigation(ap, shipEntity, physics) then return end

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
---@param physics Physics The world the ship's body is in
---@param pos Position
---@param arrivalRange number|nil
---@param parentEntity Entity|nil Parent entity for tracking (e.g., planet for ring asteroids)
---@param localOffsetX number|nil Local X offset from parent
---@param localOffsetZ number|nil Local Z offset from parent
function AutoPilotSystem:engagePosition(shipEntity, physics, pos, arrivalRange, parentEntity, localOffsetX, localOffsetZ)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap then return end
    if not startNavigation(ap, shipEntity, physics) then return end

    local cfg = Config.game.autoPilot or {}
    ap:setActive(true)
    ap:setTargetEntity(nil)
    ap:setTargetPos(pos)
    ap:setArrivalRange(arrivalRange or cfg.arrivalRange or 500)
    ap:setElapsed(0)
    ap._parentEntity = parentEntity
    ap._localOffsetX = localOffsetX
    ap._localOffsetZ = localOffsetZ
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
    ap:clearNavigation()

    -- Hand the controls back: no axes left over from the autopilot, the travel drive stopped and its cap lifted.
    local input = shipEntity:get(ConstructComponents.ControlInput)
    if input then
        input:clearAxes()
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
        input:setDriveSpeedCap(nil)
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

--- The body the autopilot's goal moves with: its target entity, or the body that a
--- parent-relative target point rides on. Nil for a fixed point.
---@param ap AutoPilotComponent
---@return RigidBody|nil
local function trackedBody(ap)
    local source = ap:getTargetEntity() or ap._parentEntity
    local rbCmp = source and source:get(PhysicsComponents.RigidBody)
    return rbCmp and rbCmp:getRigidBody() or nil
end

--- The velocity of what the autopilot is tracking. Zero for a fixed point.
---@param body RigidBody|nil
---@return gameunit vx, gameunit vy, gameunit vz Game units per second
local function bodyVelocity(body)
    if not body then return 0, 0, 0 end
    local velocity = body:getVelocity()
    return velocity.x, velocity.y, velocity.z
end

--- Fly the agent to `goal`: a point that moves with `frameBody` keeps its place relative to the
--- body, so the NavWorld matches the body's velocity; a bare point stays where it is.
---@param nav NavWorld
---@param agent integer
---@param frameBody RigidBody|nil
---@param goal Position
---@param arrivalRadius gameunit
local function setGoal(nav, agent, frameBody, goal, arrivalRadius)
    if not frameBody then
        nav:setArrive(agent, goal, arrivalRadius)
        return
    end
    local bodyPos = frameBody:getPos()
    nav:setArriveRelative(agent, frameBody,
        Vec3d(goal.x - bodyPos.x, goal.y - bodyPos.y, goal.z - bodyPos.z), arrivalRadius)
end

--- The goal the ship flies to and the radius around it that counts as arrived: the point on the
--- line from the steering point to the ship that is `safeRange` from the steering point. A ship
--- that is already within `safeRange` of its target has arrived where it is.
---@param shipPos Position
---@param steerPos Position The point the ship steers at (the target, or its lead intercept)
---@param steerDist number Distance from the ship to `steerPos`
---@param targetDist number Distance from the ship to the target itself
---@param safeRange number Stand-off distance from the target's centre
---@return Position goal
local function standOffGoal(shipPos, steerPos, steerDist, targetDist, safeRange)
    if targetDist <= safeRange or steerDist <= safeRange then
        return shipPos
    end
    local back = safeRange / steerDist
    return Position(
        steerPos.x + (shipPos.x - steerPos.x) * back,
        steerPos.y + (shipPos.y - steerPos.y) * back,
        steerPos.z + (shipPos.z - steerPos.z) * back)
end

--- The drive multiplier the braking law wants at a distance from the goal (`TravelDriveMath`).
---@param goalDist gameunit
---@param profile FlightProfile
---@return number
local function brakingMultiplier(goalDist, profile)
    local lag = TravelDriveMath.brakingTime(Config.game.travelDrive.rampDownSpeed, profile.linearDrag)
    return TravelDriveMath.brakingMultiplier(goalDist, profile.maxSpeed, lag)
end

--- Seconds until the ship is within `arrivalRadius` of the goal, from the same law that
--- `updateDrive` flies (`TravelDriveMath.travelTime`).
---@param goalDist gameunit Distance to the goal, 0 or more
---@param arrivalRadius gameunit
---@param profile FlightProfile
---@param zoneMax number The drive multiplier the zone allows
---@param driveUsed boolean Whether the drive is, or will be, engaged
---@return number seconds
local function travelTime(goalDist, arrivalRadius, profile, zoneMax, driveUsed)
    return TravelDriveMath.travelTime(goalDist, arrivalRadius, profile.maxSpeed,
        Config.game.travelDrive.rampDownSpeed, profile.linearDrag, zoneMax, driveUsed)
end

--- Is the drive engaged, or far enough away that it will be?
---@param drive TravelDriveComponent|nil
---@param dist gameunit Distance to the target's centre
---@param safeRange gameunit
---@return boolean
local function driveWillRun(drive, dist, safeRange)
    local cfg = Config.game.autoPilot or {}
    if not drive or cfg.enableTravelDrive == false then return false end
    return drive:isActive() or dist > safeRange * DRIVE_ENGAGE_STAND_OFFS
end

--- Radius of the target for the stand-off distance.
---@param targetEntity Entity|nil
---@return gameunit
local function targetRadiusOf(targetEntity)
    if not targetEntity then return 0 end
    local tRbCmp = targetEntity:get(PhysicsComponents.RigidBody)
    if tRbCmp and tRbCmp:getRigidBody() then
        return tRbCmp:getRigidBody():getBoundingRadius()
    end
    local tTransform = targetEntity:get(PhysicsComponents.Transform)
    return tTransform and tTransform:getScale() or 0
end

--- Drive the travel drive from the braking law: the cap follows the distance to the goal, and the
--- drive is released for the final approach, which the NavWorld flies at normal speed.
---@param dt number
---@param ap AutoPilotComponent
---@param drive TravelDriveComponent
---@param input ControlInputComponent
---@param profile FlightProfile
---@param zoneMax number
---@param dist gameunit Distance to the target's centre
---@param safeRange gameunit
local function updateDrive(dt, ap, drive, input, profile, zoneMax, dist, safeRange)
    local wanted = brakingMultiplier(math.max(0, dist - safeRange), profile)
    -- The drive system ramps its multiplier toward this cap over time.
    input:setDriveSpeedCap(math.min(zoneMax, math.max(1, wanted)))
    if drive:getState() ~= "idle" and wanted < HANDOVER_MULT then
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
    end

    local cfg = Config.game.autoPilot or {}
    local elapsed = ap:getElapsed() + dt
    ap:setElapsed(elapsed)
    if cfg.enableTravelDrive ~= false and elapsed > (cfg.travelDriveDelay or 5.0)
        and dist > safeRange * DRIVE_ENGAGE_STAND_OFFS and drive:getState() == "idle" then
        input:setDriveRequest(Enums.TravelDriveRequest.Engage)
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

    -- Recompute position from parent + local offset (for orbiting targets)
    if ap._parentEntity and ap._localOffsetX then
        local pRb = ap._parentEntity:get(PhysicsComponents.RigidBody)
        if pRb and pRb:getRigidBody() then
            local pp = pRb:getRigidBody():getPos()
            targetPos = Position(pp.x + ap._localOffsetX, pp.y, pp.z + ap._localOffsetZ)
            ap:setTargetPos(targetPos)
        end
    end

    if not targetPos then self:disengage(shipEntity); return end

    local shipPos = rb:getPos()
    -- Relative to the well that carries the ship: the part of its velocity that drag acts on.
    local speed = ShipFlightSystem:getSpeed(shipEntity)
    local profile = ConstructComponents.FlightProfile.ProfileOf(shipEntity)
    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    local zoneMax = GravityWellSystem:getMaxDriveSpeedAt(shipPos)

    local frameBody = trackedBody(ap)
    ap.targetVelX, ap.targetVelY, ap.targetVelZ = bodyVelocity(frameBody)

    local targetRadius = targetRadiusOf(targetEntity)
    local safeRange = self.standOff(targetRadius)
    local arrivalRadius = self.arrivalRadius(targetRadius)

    -- Quick distance check for approach mode
    local qdx = targetPos.x - shipPos.x
    local qdy = targetPos.y - shipPos.y
    local qdz = targetPos.z - shipPos.z
    local quickDist = math.sqrt(qdx*qdx + qdy*qdy + qdz*qdz)

    -- Lead prediction: aim where target WILL BE when we arrive
    -- Skip leading when close (within 5000 units) or inside any gravity well — fly direct
    local closeApproach = quickDist < 5000
    local well = shipEntity:get(PhysicsComponents.GravityWell)
    local inGravityWell = well ~= nil and well:getZoneType() ~= PhysicsComponents.GravityWell.OPEN_SPACE
    local inTargetZone = closeApproach or inGravityWell

    local interceptPos = targetPos
    local tSpeed = math.sqrt(ap.targetVelX^2 + ap.targetVelY^2 + ap.targetVelZ^2)
    if tSpeed > 0.1 and not inTargetZone then
        local driveUsed = driveWillRun(drive, quickDist, safeRange)
        local eta = travelTime(math.max(0, quickDist - safeRange), arrivalRadius, profile, zoneMax, driveUsed)

        -- Two-pass refinement
        for _ = 1, 2 do
            local lx = targetPos.x + ap.targetVelX * eta
            local ly = targetPos.y + ap.targetVelY * eta
            local lz = targetPos.z + ap.targetVelZ * eta
            local ldx = lx - shipPos.x
            local ldy = ly - shipPos.y
            local ldz = lz - shipPos.z
            local leadDist = math.sqrt(ldx*ldx + ldy*ldy + ldz*ldz)
            eta = travelTime(math.max(0, leadDist - safeRange), arrivalRadius, profile, zoneMax, driveUsed)
        end

        -- Blend lead prediction: full lead when far, direct approach when close
        local tdx = targetPos.x - shipPos.x
        local tdy = targetPos.y - shipPos.y
        local tdz = targetPos.z - shipPos.z
        local rawDist = math.sqrt(tdx*tdx + tdy*tdy + tdz*tdz)
        local leadBlend = math.min(1.0, rawDist / math.max(1, speed * 30))  -- fade over 30s travel

        interceptPos = Position(
            targetPos.x + ap.targetVelX * eta * leadBlend,
            targetPos.y + ap.targetVelY * eta * leadBlend,
            targetPos.z + ap.targetVelZ * eta * leadBlend)
    end

    -- Distance to ACTUAL target (for arrival check + drive decel)
    local actualTargetPos = targetPos
    local adx = actualTargetPos.x - shipPos.x
    local ady = actualTargetPos.y - shipPos.y
    local adz = actualTargetPos.z - shipPos.z
    local actualDist = math.sqrt(adx * adx + ady * ady + adz * adz)

    -- Distance to intercept (for steering)
    local idx = interceptPos.x - shipPos.x
    local idy = interceptPos.y - shipPos.y
    local idz = interceptPos.z - shipPos.z
    local interceptDist = math.sqrt(idx * idx + idy * idy + idz * idz)

    -- Store nav data on component for map/HUD
    ap.interceptPos = interceptPos
    ap.distance = actualDist
    ap.eta = travelTime(math.max(0, actualDist - safeRange), arrivalRadius, profile, zoneMax,
        driveWillRun(drive, actualDist, safeRange))

    -- Use actual distance for arrival/drive logic, intercept for steering
    local dist = actualDist

    -- Fly to the stand-off point; the NavWorld steers (it also brakes and stops there) and says
    -- when the ship has arrived. The point moves with the tracked body, and the ship is steered
    -- relative to the well that carries it.
    local nav, agent = ap:getNav(), ap:getAgent()
    setGoal(nav, agent, frameBody,
        standOffGoal(shipPos, interceptPos, interceptDist, actualDist, safeRange), arrivalRadius)
    NavAgentHelper.setFrameFromWell(nav, agent, shipEntity)
    nav:step()
    nav:copyArrived(ap:getArrived(), 1)
    if ap:getArrived()[0] then
        Log.Info("AutoPilot: arrived at destination (dist=%.0f, safe=%.0f)", dist, safeRange)
        self:disengage(shipEntity)
        return
    end
    nav:copyAxes(ap:getAxes(), NavWorld.AxisCount())

    local input = shipEntity:get(ConstructComponents.ControlInput)
    if input then
        NavAgentHelper.writeAxes(ap:getAxes(), agent, input)
    end

    if drive and input then
        updateDrive(dt, ap, drive, input, profile, zoneMax, dist, safeRange)
    else
        ap:setElapsed(ap:getElapsed() + dt)
    end
end

return AutoPilotSystem
