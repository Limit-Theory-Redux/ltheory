local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local NavAgentHelper      = require("Shared.Helpers.NavAgentHelper")
local PlanFlight          = require("Shared.Helpers.PlanFlight")

local Phase = PlanFlight.Phase

-- The collision mask the NavWorld senses obstacles with: every collision group.
local SENSING_MASK = 0xFFFFFFFF

--- AutoPilot System — steers a ship toward a target entity or position.
--- Operates on the AutoPilotComponent attached to ship entities. It writes the ship's
--- ControlInput from a NavWorld that has the ship as its only agent; it applies no force itself.
--- The NavWorld senses the collidable bodies around the ship and steers around them. While the
--- travel drive runs the ship has no lateral thrust and turns weakly, so that avoidance only
--- acts in normal flight. The path to the target comes from the system planner (`PlanFlight`):
--- the ship flies the plan's corners, under the drive on the legs the plan marks, and then the
--- final approach on its own steering.
---@class AutoPilotSystem
local AutoPilotSystem = {}

-- Defined below, where the target helpers it uses are.
local beginPlan

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
    beginPlan(ap, shipEntity)
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
    beginPlan(ap, shipEntity)
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
    PlanFlight.drop(ap:getPlan())
    ap:setPlan(nil)
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

--- Is the ship's path still being planned? The ship steers straight to the target until the plan has come.
---@param shipEntity Entity
---@return boolean
function AutoPilotSystem:isPlanning(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    local plan = ap and ap:getPlan()
    return plan ~= nil and plan.phase == Phase.Planning
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

--- The position of the target now: its body, else its point.
---@param ap AutoPilotComponent
---@return Position|nil
local function targetPosition(ap)
    local targetEntity = ap:getTargetEntity()
    if targetEntity then
        local tRbCmp = targetEntity:get(PhysicsComponents.RigidBody)
        if tRbCmp and tRbCmp:getRigidBody() then return tRbCmp:getRigidBody():getPos() end
        local tTransform = targetEntity:get(PhysicsComponents.Transform)
        if tTransform then return tTransform:getPos() end
    end
    return ap:getTargetPos()
end

--- Starts the engagement's plan: asks the planner for the path from the ship to the target's
--- stand-off point. A ship already within the stand-off distance has no path to plan.
---@param ap AutoPilotComponent
---@param shipEntity Entity
function beginPlan(ap, shipEntity)
    local rb = shipEntity:get(PhysicsComponents.RigidBody):getRigidBody()
    local targetEntity = ap:getTargetEntity()
    local targetRadius = PlanFlight.targetRadius(targetEntity)
    ap:setPlan(PlanFlight.begin(shipEntity, rb, targetPosition(ap), PlanFlight.standOff(targetRadius),
        PlanFlight.arrivalRadius(targetRadius), targetEntity or ap._parentEntity))
end

--- One step of flying the plan's corners. The NavWorld steers to the follower's point; the
--- follower moves the point on, and when it has passed the last corner the phase becomes `Final`.
---@param ap AutoPilotComponent
---@param shipEntity Entity
---@param speed number Game units per second, relative to the carrying well
---@param targetPos Position
---@param zoneMax number
---@param dt number Seconds
local function flyPlan(ap, shipEntity, speed, targetPos, zoneMax, dt)
    local plan = ap:getPlan()
    local rb = shipEntity:get(PhysicsComponents.RigidBody):getRigidBody()
    local nav, agent = ap:getNav(), ap:getAgent()
    NavAgentHelper.setFrameFromWell(nav, agent, shipEntity)
    nav:step(dt)
    nav:copyArrived(ap:getArrived(), 1)
    plan.follower:advance(ap:getArrived()[0], speed, ap:getElapsed(), plan.listener, NavAgentHelper.isCarried(shipEntity))
    nav:copyAxes(ap:getAxes(), NavWorld.AxisCount())

    local input = shipEntity:get(ConstructComponents.ControlInput)
    if input then NavAgentHelper.writeAxes(ap:getAxes(), agent, input) end

    ap.interceptPos = targetPos
    ap.distance = rb:getPos():distance(targetPos)
    ap.eta = math.max(0, plan.goal.time - plan.snapshot.age)

    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    if drive and input and plan.phase == Phase.Plan then
        PlanFlight.driveLeg(plan, drive, input, ConstructComponents.FlightProfile.ProfileOf(shipEntity), zoneMax)
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

    local targetRadius = PlanFlight.targetRadius(targetEntity)
    local safeRange = PlanFlight.standOff(targetRadius)
    local arrivalRadius = PlanFlight.arrivalRadius(targetRadius)

    -- The planner's path: take it when it has come, and fly its corners.
    local plan = ap:getPlan() or { phase = Phase.Direct }
    if plan.phase == Phase.Planning then
        PlanFlight.poll(plan, {
            nav = ap:getNav(), agent = ap:getAgent(), rb = rb, profile = profile,
        })
    end
    ap:setElapsed(ap:getElapsed() + dt)
    if plan.snapshot then plan.snapshot:advance(dt) end
    if plan.phase == Phase.Plan then
        flyPlan(ap, shipEntity, speed, targetPos, zoneMax, dt)
        return
    end

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
        local driveUsed = PlanFlight.driveWillRun(drive, quickDist, safeRange)
        local eta = PlanFlight.travelTime(math.max(0, quickDist - safeRange), arrivalRadius, profile, zoneMax, driveUsed)

        -- Two-pass refinement
        for _ = 1, 2 do
            local lx = targetPos.x + ap.targetVelX * eta
            local ly = targetPos.y + ap.targetVelY * eta
            local lz = targetPos.z + ap.targetVelZ * eta
            local ldx = lx - shipPos.x
            local ldy = ly - shipPos.y
            local ldz = lz - shipPos.z
            local leadDist = math.sqrt(ldx*ldx + ldy*ldy + ldz*ldz)
            eta = PlanFlight.travelTime(math.max(0, leadDist - safeRange), arrivalRadius, profile, zoneMax, driveUsed)
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
    ap.eta = PlanFlight.travelTime(math.max(0, actualDist - safeRange), arrivalRadius, profile, zoneMax,
        PlanFlight.driveWillRun(drive, actualDist, safeRange))

    -- Use actual distance for arrival/drive logic, intercept for steering
    local dist = actualDist

    -- Fly to the stand-off point; the NavWorld steers (it also brakes and stops there) and says
    -- when the ship has arrived. The point moves with the tracked body, and the ship is steered
    -- relative to the well that carries it.
    local nav, agent = ap:getNav(), ap:getAgent()
    PlanFlight.setGoal(nav, agent, frameBody,
        PlanFlight.standOffGoal(shipPos, interceptPos, interceptDist, actualDist, safeRange), arrivalRadius)
    NavAgentHelper.setFrameFromWell(nav, agent, shipEntity)
    nav:step(dt)
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

    -- The drive runs on the planned legs, the final approach and a flight with no usable plan
    -- (which has no detours), never while the plan is still out.
    if drive and input and (plan.phase == Phase.Final or plan.phase == Phase.Direct) then
        PlanFlight.updateDrive(drive, input, profile, zoneMax, dist, safeRange)
    end
end

return AutoPilotSystem
