local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ApproachSpec = require("Shared.Types.ApproachSpec")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local Docking = require("Modules.Docking.Components")

-- Design: doc/script/docking.md

---Docks ships at stations. A ship that asks to dock holds a free port of the station; it flies a
---navigation `MoveTo` to the port's approach point (a `Point` approach, which moves with the
---station), then its controls are claimed (`Nav.claim`), its body is made kinematic and it glides
---into the port, turning to face out. While docked it is held in the port as the station moves.
---Undocking glides it back to the approach point, gives its body back to physics and its controls
---back to its orders. The glides run at PreSim, before the physics step moves the bodies.
---@class DockingSystem
local DockingSystem = Class("DockingSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function DockingSystem:registerVars()
    self.profiler = QuickProfiler("DockingSystem", false, false)
    -- How fast a ship glides in and out of a port, and the bounds of a glide's duration.
    self.glideSpeed = ScaleConfig:toGameUnits(250, "starSystem")
    self.minGlideS = 3
    self.maxGlideS = 20
end

function DockingSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function DockingSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---@param entityOrId Entity|integer|nil
---@return Entity|nil
local function alive(entityOrId)
    if entityOrId == nil then return nil end
    local entity = rawtype(entityOrId) == "number" and Entity(entityOrId) or entityOrId
    return Registry:hasEntity(entity) and entity or nil
end

---@param entity Entity
---@return RigidBody|nil
local function bodyOf(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    return rbCmp and rbCmp:getRigidBody() or nil
end

---@param offset ApproachOffset
---@return ApproachSpec
local function pointSpec(offset)
    return ApproachSpec.Of { kind = Enums.ApproachKind.Point, offsetRadii = offset }
end

---Whether a port is held: by a ship that exists and is docking at this port of this station. A
---port whose holder is gone or docks elsewhere is freed.
---@param port DockingPort
---@param stationId integer
---@param index integer
---@return boolean
local function isHeld(port, stationId, index)
    local occupant = alive(port.occupant)
    local docking = occupant and occupant:get(Docking.Docking)
    if docking and docking:getStation() == stationId and docking:getPort() == index then return true end
    port.occupant = nil
    return false
end

---@param station Entity
---@param ports DockingPortsComponent
---@return integer|nil index
local function freePort(station, ports)
    for index, port in ipairs(ports:getPorts()) do
        if not isHeld(port, station.id, index) then return index end
    end
    return nil
end

---@param station Entity
---@return integer free, integer total Ports of the station
function DockingSystem:countPorts(station)
    local ports = station:get(Docking.DockingPorts)
    if not ports then return 0, 0 end
    local free = 0
    for index, port in ipairs(ports:getPorts()) do
        if not isHeld(port, station.id, index) then free = free + 1 end
    end
    return free, #ports:getPorts()
end

---Asks for the ship to dock at the station: it takes a free port and flies to it.
---@param ship Entity A ship that takes navigation orders
---@param station Entity A station with `DockingPortsComponent` and a rigid body
---@return boolean asked False when the station has no free port, or the ship docks already
function DockingSystem:requestDock(ship, station)
    if ship:get(Docking.Docking) then return false end
    local ports = station:get(Docking.DockingPorts)
    if not ports or not bodyOf(station) then return false end
    local index = freePort(station, ports)
    if not index then return false end

    local port = ports:getPort(index)
    port.occupant = ship.id
    local docking = ship:add(Docking.Docking(station.id, index))
    docking.order = Nav.order(ship, Nav.Orders.MoveTo { target = station, approach = pointSpec(port.approachRadii) })
    return true
end

---Asks for a docked ship to leave its port. A ship still on its way to the port gives it up at once.
---@param ship Entity
---@return boolean leaving
function DockingSystem:undock(ship)
    local docking = ship:get(Docking.Docking)
    if not docking then return false end
    local phase = docking:getPhase()
    if phase == Enums.DockingPhase.Docked or phase == Enums.DockingPhase.Entering then
        local station = alive(docking:getStation())
        if not station then
            self:release(ship, docking)
            return true
        end
        self:beginGlide(ship, docking, station, Enums.DockingPhase.Leaving)
        return true
    end
    if phase == Enums.DockingPhase.Approaching or phase == Enums.DockingPhase.Failed then
        self:release(ship, docking)
        return true
    end
    return phase == Enums.DockingPhase.Leaving
end

---@param ship Entity
---@return DockingPhase|nil phase Nil for a ship that isn't docking
function DockingSystem:phaseOf(ship)
    local docking = ship:get(Docking.Docking)
    return docking and docking:getPhase() or nil
end

---@param ship Entity
---@return boolean
function DockingSystem:isDocked(ship)
    return self:phaseOf(ship) == Enums.DockingPhase.Docked
end

---Ends a docking at once: the order is cancelled or released, the port freed, and the ship's body
---and controls given back. A ship in a failed docking needs this before it docks again.
---@param ship Entity
---@param docking DockingComponent|nil
function DockingSystem:release(ship, docking)
    docking = docking or ship:get(Docking.Docking)
    if not docking then return end
    self:dropOrder(docking)
    local phase = docking:getPhase()
    if phase == Enums.DockingPhase.Entering or phase == Enums.DockingPhase.Docked or phase == Enums.DockingPhase.Leaving then
        self:freeBody(ship)
    end
    local station = alive(docking:getStation())
    local ports = station and station:get(Docking.DockingPorts)
    local port = ports and ports:getPort(docking:getPort())
    if port and port.occupant == ship.id then port.occupant = nil end
    ship:remove(Docking.Docking)
end

---@param docking DockingComponent
function DockingSystem:dropOrder(docking)
    local order = docking.order
    docking.order = nil
    if not order or not Registry:hasEntity(order) then return end
    if order:get(Nav.Components.OrderStatus):hasEnded() then
        Nav.release(order)
    else
        -- The navigation system ends it at its next update; it is released then.
        Nav.cancel(order)
        self.cancelled = self.cancelled or {}
        self.cancelled[#self.cancelled + 1] = order
    end
end

---@param ship Entity
function DockingSystem:freeBody(ship)
    local body = bodyOf(ship)
    if body then body:setKinematic(false) end
    Nav.unclaim(ship)
end

---The station's axes, and the place of an offset in station radii along them.
---@param station Entity
---@return Position stationPos, Quat stationRot, number radius
local function frameOf(station, dt)
    local body = bodyOf(station)
    local pos, rot = body:getPos(), body:getRot()
    -- The station moves in this frame's physics step too: lead it by its velocity, so a held ship
    -- keeps its place instead of trailing a step behind.
    local velocity = body:getVelocity()
    pos = Position(pos.x + velocity.x * dt, pos.y + velocity.y * dt, pos.z + velocity.z * dt)
    return pos, rot, body:getBoundingRadius()
end

---Where a point given in game units along the station's axes is in the world.
---@param pos Position
---@param rot Quat
---@param local3 { right: number, up: number, forward: number }
---@return Position
local function toWorld(pos, rot, local3)
    local right, up, forward = rot:getRight(), rot:getUp(), rot:getForward()
    return Position(
        pos.x + right.x * local3.right + up.x * local3.up + forward.x * local3.forward,
        pos.y + right.y * local3.right + up.y * local3.up + forward.y * local3.forward,
        pos.z + right.z * local3.right + up.z * local3.up + forward.z * local3.forward)
end

---@param pos Position
---@param rot Quat
---@param world Position
---@return { right: number, up: number, forward: number }
local function toLocal(pos, rot, world)
    local dx, dy, dz = world.x - pos.x, world.y - pos.y, world.z - pos.z
    local right, up, forward = rot:getRight(), rot:getUp(), rot:getForward()
    return {
        right = dx * right.x + dy * right.y + dz * right.z,
        up = dx * up.x + dy * up.y + dz * up.z,
        forward = dx * forward.x + dy * forward.y + dz * forward.z,
    }
end

---@param offset ApproachOffset In radii
---@param radius number
local function scaled(offset, radius)
    return { right = offset.right * radius, up = offset.up * radius, forward = offset.forward * radius }
end

---The orientation of a ship in the port, relative to the station: nose out of the port.
---@param port DockingPort
---@return Quat
local function portFacing(port)
    local o = port.offsetRadii
    local length = math.sqrt(o.right * o.right + o.up * o.up + o.forward * o.forward)
    return Quat.FromLook(Vec3f(o.right / length, o.up / length, o.forward / length), Vec3f(0, 1, 0))
end

---Starts a glide from where the ship is: into the port (`Entering`) or out to the approach point (`Leaving`).
---@param ship Entity
---@param docking DockingComponent
---@param station Entity
---@param phase DockingPhase
function DockingSystem:beginGlide(ship, docking, station, phase)
    local pos, rot, radius = frameOf(station, 0)
    local body = bodyOf(ship)
    docking.from = toLocal(pos, rot, body:getPos())
    docking.fromRot = rot:inverse():mul(body:getRot())
    docking:setPhase(phase)
    local port = station:get(Docking.DockingPorts):getPort(docking:getPort())
    local goal = scaled(phase == Enums.DockingPhase.Entering and port.offsetRadii or port.approachRadii, radius)
    local dx, dy, dz = goal.right - docking.from.right, goal.up - docking.from.up, goal.forward - docking.from.forward
    docking.duration = math.max(self.minGlideS, math.min(self.maxGlideS, math.sqrt(dx * dx + dy * dy + dz * dz) / self.glideSpeed))
end

---@param dt number Seconds
function DockingSystem:update(dt)
    self.profiler:start()
    self:releaseCancelled()
    local dockings = {}
    for ship, docking in Registry:view(Docking.Docking) do dockings[#dockings + 1] = { ship, docking } end
    for _, entry in ipairs(dockings) do
        self:updateDocking(entry[1], entry[2], dt)
    end
    self.profiler:stop()
end

function DockingSystem:releaseCancelled()
    if not self.cancelled then return end
    local waiting = {}
    for _, order in ipairs(self.cancelled) do
        if Registry:hasEntity(order) then
            if order:get(Nav.Components.OrderStatus):hasEnded() then Nav.release(order) else waiting[#waiting + 1] = order end
        end
    end
    self.cancelled = #waiting > 0 and waiting or nil
end

---@param t number 0 to 1
---@return number eased
local function smoothstep(t)
    t = math.max(0, math.min(1, t))
    return t * t * (3 - 2 * t)
end

---@param ship Entity
---@param docking DockingComponent
---@param dt number
function DockingSystem:updateDocking(ship, docking, dt)
    local phase = docking:getPhase()
    if phase == Enums.DockingPhase.Failed then return end
    local station = alive(docking:getStation())
    if not station or not bodyOf(station) then
        -- The station is gone: whatever the phase, the ship flies on its own.
        self:dropOrder(docking)
        if phase ~= Enums.DockingPhase.Approaching then self:freeBody(ship) end
        docking:setPhase(Enums.DockingPhase.Failed)
        return
    end

    if phase == Enums.DockingPhase.Approaching then
        local status = Nav.status(docking.order)
        if status == Enums.NavOrderStatus.Done then
            Nav.release(docking.order)
            docking.order = nil
            Nav.claim(ship)
            bodyOf(ship):setKinematic(true)
            self:beginGlide(ship, docking, station, Enums.DockingPhase.Entering)
        elseif status == Enums.NavOrderStatus.Failed or status == Enums.NavOrderStatus.Cancelled then
            Nav.release(docking.order)
            docking.order = nil
            local port = station:get(Docking.DockingPorts):getPort(docking:getPort())
            if port and port.occupant == ship.id then port.occupant = nil end
            docking:setPhase(Enums.DockingPhase.Failed)
        end
        return
    end

    local pos, rot, radius = frameOf(station, dt)
    local port = station:get(Docking.DockingPorts):getPort(docking:getPort())
    local inPort = scaled(port.offsetRadii, radius)
    local facing = portFacing(port)

    if phase == Enums.DockingPhase.Docked then
        bodyOf(ship):setKinematicTarget(toWorld(pos, rot, inPort), rot:mul(facing))
        return
    end

    docking.progress = math.min(1, docking.progress + dt / docking.duration)
    local s = smoothstep(docking.progress)
    local goal = phase == Enums.DockingPhase.Entering and inPort or scaled(port.approachRadii, radius)
    local from = docking.from
    local at = {
        right = from.right + (goal.right - from.right) * s,
        up = from.up + (goal.up - from.up) * s,
        forward = from.forward + (goal.forward - from.forward) * s,
    }
    local relative = phase == Enums.DockingPhase.Entering and docking.fromRot:slerp(facing, s) or facing
    bodyOf(ship):setKinematicTarget(toWorld(pos, rot, at), rot:mul(relative))

    if docking.progress >= 1 then
        if phase == Enums.DockingPhase.Entering then
            docking:setPhase(Enums.DockingPhase.Docked)
        else
            self:release(ship, docking)
        end
    end
end

return DockingSystem()
