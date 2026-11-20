-- Flies a ship along the corners of a system plan through a `NavWorld` agent: each leg is a Seek to
-- the entry of the next corner's arc (or the corner, when sharp) at the corner's speed, each arc is
-- flown by pure pursuit under the corner's speed cap. Corner points are taken from where their
-- frame bodies are now, except a well's entry, which is sought where it is planned to be on arrival;
-- the arcs are the ones the planner cleared, kept at their planned offsets. A drive run's corner
-- (`through`, no arc) is passed a lead (`speed / drag`) before it, where the drive turn begins.
-- The follower stops after the last corner; the owner flies the leg to the goal.
-- Design: doc/script/nav/system-planner.md
local SystemPlanner = require("Shared.Helpers.SystemPlanner")

-- A ship on an arc is steered at the point of the arc it would reach in this many seconds at the
-- corner's speed, at most this many radians ahead of it (pure pursuit).
local ARC_LOOKAHEAD_S   = 0.3
local MAX_ARC_LOOKAHEAD = 0.6

---A frame body a plan's waypoints can be offsets from.
---@class PlanFrame
---@field rb RigidBody The body now
---@field positionAt fun(t: number): table Where the body is `t` seconds from the plan's snapshot, `{ x, y, z }`

---A circular arc the ship flies through a corner.
---@class PlanArc
---@field entry table { x, y, z }
---@field exit table { x, y, z }
---@field centre table { x, y, z }
---@field normal table Unit { x, y, z }: the ship turns anticlockwise about it
---@field turn number Radians from the entry to the exit
---@field radius number Game units

---What the ship did on one arc, for the owner's log.
---@class PlanArcFlight
---@field index integer Index of the corner in the plan
---@field radius number Game units, as planned
---@field speed number Game units per second, as planned
---@field enteredAt number Time the ship reached the entry
---@field entrySpeed number Game units per second
---@field peakSpeed number Largest speed on the arc, game units per second
---@field maxError number Largest distance from the arc's circle, game units
---@field maxShare number Largest share of the forward thrust the planned radius needs at the speed flown

---Receives what the follower does. Called with the listener as the first argument.
---@class PlanFollowerListener
---@field pointPassed fun(self: table, index: integer, speed: number, time: number) A corner's entry (or sharp corner) was reached
---@field arcFlown fun(self: table, flight: PlanArcFlight, time: number) An arc is over
---@field legChanged fun(self: table, time: number) The ship was sent on to another corner, or finished
---@field finished fun(self: table, time: number) The last corner is passed

---@class PlanFollower
---@field nav NavWorld
---@field agent integer
---@field rb RigidBody The ship's body
---@field frames PlanFrame[] Indexed by a waypoint's `frame`
---@field points PlanWaypoint[] The corners, in order
---@field goal PlanWaypoint The plan's last waypoint, which the last corner's arc leaves toward
---@field start table { x, y, z } where the ship was when the plan was made
---@field passRadius number A corner is reached within this distance, game units
---@field accel number The ship's forward acceleration, game units per second squared
---@field drag number The ship's linear drag rate
---@field index integer The corner being flown to
---@field arc PlanArcFlight|nil The arc being flown; nil on a leg
---@field finished boolean Whether the last corner has been passed
---@field carried boolean Whether a gravity well carried the ship at the last `advance`
---@field target Position The point the ship is steering to: the next corner's arc entry or sharp corner, or the arc's exit
local PlanFollower = {}
PlanFollower.__index = PlanFollower

local function sub(a, b) return { x = a.x - b.x, y = a.y - b.y, z = a.z - b.z } end
local function scale(a, s) return { x = a.x * s, y = a.y * s, z = a.z * s } end
local function dot(a, b) return a.x * b.x + a.y * b.y + a.z * b.z end
local function length(a) return math.sqrt(dot(a, a)) end
local function cross(a, b)
    return { x = a.y * b.z - a.z * b.y, y = a.z * b.x - a.x * b.z, z = a.x * b.y - a.y * b.x }
end

---@param v table { x, y, z }
---@return Position
local function toPosition(v) return Position(v.x, v.y, v.z) end

---Starts flying the corners of a plan, and sends the ship to the first.
---@param spec { nav: NavWorld, agent: integer, rb: RigidBody, frames: PlanFrame[], points: PlanWaypoint[], goal: PlanWaypoint, start: table, passRadius: number, accel: number, drag: number }
---@return PlanFollower
function PlanFollower.new(spec)
    assert(#spec.points > 0, "PlanFollower: a plan with no corner has nothing to follow")
    local follower = setmetatable({
        nav = spec.nav, agent = spec.agent, rb = spec.rb, frames = spec.frames, points = spec.points,
        goal = spec.goal, start = spec.start, passRadius = spec.passRadius, accel = spec.accel,
        drag = spec.drag, index = 1, finished = false, carried = false,
    }, PlanFollower)
    follower:seekLeg(follower:currentArc())
    return follower
end

---The world point of a waypoint now: its offset from its frame body, or the point itself.
---@param waypoint PlanWaypoint
---@return Position
function PlanFollower:pointNow(waypoint)
    local offset = waypoint.offset
    if not waypoint.frame then return offset end
    local centre = self.frames[waypoint.frame].rb:getPos()
    return Position(centre.x + offset.x, centre.y + offset.y, centre.z + offset.z)
end

---A waypoint's world point at its planned time, which is where the planner placed it.
---@param waypoint PlanWaypoint
---@return Position
function PlanFollower:pointPlanned(waypoint)
    local offset = waypoint.offset
    if not waypoint.frame then return offset end
    local centre = self.frames[waypoint.frame].positionAt(waypoint.time)
    return Position(centre.x + offset.x, centre.y + offset.y, centre.z + offset.z)
end

---The arc of the corner the ship is flying to. The corner is where it is now; its neighbours (the
---start for the first corner, the goal for the last) keep the planned offset from it, so the arc is
---the one the planner cleared even when the corner or a neighbour rides on a moving body.
---@return PlanArc|nil arc Nil for a sharp corner
function PlanFollower:currentArc()
    local index = self.index
    local waypoint = self.points[index]
    if waypoint.radius <= 0 then return nil end
    local corner = self:pointNow(waypoint)
    local plannedCorner = self:pointPlanned(waypoint)
    local function around(planned)
        return Position(corner.x + planned.x - plannedCorner.x, corner.y + planned.y - plannedCorner.y,
            corner.z + planned.z - plannedCorner.z)
    end
    local before = around(index == 1 and toPosition(self.start) or self:pointPlanned(self.points[index - 1]))
    local after = around(self:pointPlanned(self.points[index + 1] or self.goal))
    local ex, ey, ez, xx, xy, xz, ox, oy, oz, kx, ky, kz, turn = SystemPlanner.cornerArc(
        before.x, before.y, before.z, corner.x, corner.y, corner.z, after.x, after.y, after.z, waypoint.radius)
    if not ex then return nil end
    return {
        entry = { x = ex, y = ey, z = ez }, exit = { x = xx, y = xy, z = xz },
        centre = { x = ox, y = oy, z = oz }, normal = { x = kx, y = ky, z = kz },
        turn = turn, radius = waypoint.radius,
    }
end

---The point the ship is to be at now, which its progress is judged against: the entry of the arc or
---the sharp corner it is flying to, the end of the arc it is on, then the goal.
---@return Position
function PlanFollower:point()
    if self.finished then return self:pointNow(self.goal) end
    local arc = self:currentArc()
    if not arc then return self:pointNow(self.points[self.index]) end
    return toPosition(self.arc and arc.exit or arc.entry)
end

---Whether the leg being flown is one the plan flies under the travel drive. An arc never is.
---@return boolean
function PlanFollower:drives()
    return not self.finished and not self.arc and self.points[self.index].drive == true
end

---The distance the drive run that the ship is on has left, from the ship by the waypoints to the run's
---end: the goal, or the first waypoint the drive is released at. Equal to the distance to `target`
---unless the corner being flown to is one the drive goes on through (`through`), which adds the leg
---after it. Meant for a leg flown under the drive (`drives`), whose corner is sharp.
---@return number Game units
function PlanFollower:runRemaining()
    local remaining = self.rb:getPos():distance(self.target)
    local index = self.index
    while self.points[index].through do
        local here = self:pointNow(self.points[index])
        local nextWaypoint = self.points[index + 1] or self.goal
        remaining = remaining + here:distance(self:pointNow(nextWaypoint))
        index = index + 1
        if not self.points[index] then break end
    end
    return remaining
end

---The share of its forward thrust a ship needs to fly a circle of `radius` at `speed`. With forward
---acceleration `a` and linear drag `k` that is the centripetal `s²/r` across and `k·s` along the
---velocity: `√(s⁴/r² + k²s²) / a`, the quantity the Orbit behaviour holds at a named constant.
---@param speed number Game units per second
---@param radius number Game units
---@return number
function PlanFollower:turnShare(speed, radius)
    local k = self.drag
    return math.sqrt(speed ^ 4 / (radius * radius) + k * k * speed * speed) / self.accel
end

---Whether corner `index` is the entry into a body's well: a sharp corner on a frame body that the
---leg before it reaches from outside the frame and the leg after it flies in. The planner places it
---where the straight system leg meets the moving body.
---@param index integer
---@return boolean
function PlanFollower:isEntry(index)
    local waypoint = self.points[index]
    local frame = waypoint.frame
    if not frame or waypoint.radius > 0 then return false end
    local before = self.points[index - 1]
    local after = self.points[index + 1] or self.goal
    return after.frame == frame and not (before and before.frame == frame)
end

---Sends the ship along its leg to the entry of `arc`, or to the sharp corner when there is none,
---to pass it at the corner's speed. A well's entry is sought where it is planned to be when the
---ship arrives, which keeps the system leg straight; a pursuit of the point where it is now curves
---the ship's velocity as the body moves on. A ship the well already carries moves with the body, so
---there the entry is where it is now.
---@param arc PlanArc|nil The arc of the corner being flown to
function PlanFollower:seekLeg(arc)
    local waypoint = self.points[self.index]
    local corner
    if not self.carried and self:isEntry(self.index) then
        corner = self:pointPlanned(waypoint)
    else
        corner = self:pointNow(waypoint)
    end
    self.target = arc and toPosition(arc.entry) or corner
    self.nav:setSeek(self.agent, self.target, self.passRadius, waypoint.speed)
end

---Whether the ship has reached the point at which it turns at the sharp corner it is flying to.
---A drive run goes through such a corner at a speed that covers more than the pass radius in a
---frame, so the ship can step over the corner without coming within reach of it; being past the
---plane through the corner, normal to the leg, counts as passing it. Only a corner the drive is
---kept at (`through`) is passed this way: any other is reached at a speed the pass radius is sized
---for. A corner that is not a well's entry is a drive turn: the ship turns its nose to the new leg
---`SystemPlanner.driveTurnLead` before the corner, so that its velocity, which follows the nose
---through the drag, meets the new leg (the path the planner cleared).
---@param speed number Game units per second now
---@return boolean
function PlanFollower:reachedCorner(speed)
    local waypoint = self.points[self.index]
    if not waypoint.through or waypoint.radius > 0 then return false end
    local corner = self:pointNow(waypoint)
    local before = self.index == 1 and toPosition(self.start) or self:pointNow(self.points[self.index - 1])
    local leg = sub(corner, before)
    local left = dot(leg, sub(corner, self.rb:getPos())) / length(leg)
    local lead = self:isEntry(self.index) and 0 or SystemPlanner.driveTurnLead(speed, self.drag)
    return left <= lead
end

---Steps on to the corner after the one passed; after the last, the follower is finished.
---@param time number
---@param listener PlanFollowerListener
function PlanFollower:nextPoint(time, listener)
    self.index = self.index + 1
    -- The next leg is a new course: the detour held for the one before is not for it.
    self.nav:refresh(self.agent)
    if not self.points[self.index] then
        self.finished = true
        listener:finished(time)
    end
    listener:legChanged(time)
end

---Starts the arc of the corner the ship has reached the entry of: it is held to the corner's speed.
---@param speed number Game units per second now
---@param time number
function PlanFollower:beginArc(speed, time)
    local waypoint = self.points[self.index]
    self.arc = {
        index = self.index, radius = waypoint.radius, speed = waypoint.speed, enteredAt = time,
        entrySpeed = speed, peakSpeed = speed, maxError = 0, maxShare = 0,
    }
    self.nav:setSpeedCap(self.agent, waypoint.speed)
end

---Where the ship is on `arc`: the angle about the centre from the entry, and its distance from
---the arc's circle.
---@param arc PlanArc
---@param position table { x, y, z }
---@return number angle Radians, positive toward the exit
---@return number error Game units
local function arcPosition(arc, position)
    local fromCentre = sub(position, arc.centre)
    local fromEntry = sub(arc.entry, arc.centre)
    local angle = math.atan2(dot(arc.normal, cross(fromEntry, fromCentre)), dot(fromEntry, fromCentre))
    local across = dot(arc.normal, fromCentre)
    local inPlane = length(sub(fromCentre, scale(arc.normal, across)))
    return angle, math.sqrt((inPlane - arc.radius) ^ 2 + across ^ 2)
end

---Steers a ship on `arc` at the point of it a short way ahead (pure pursuit), and keeps what the
---arc asks of the controller. The arc is over when the ship has swept its angle.
---@param arc PlanArc
---@param speed number Game units per second
---@return boolean onArc False once the ship is past the exit
function PlanFollower:flyArc(arc, speed)
    local flight = self.arc
    local angle, error = arcPosition(arc, self.rb:getPos())
    flight.peakSpeed = math.max(flight.peakSpeed, speed)
    flight.maxError = math.max(flight.maxError, error)
    flight.maxShare = math.max(flight.maxShare, self:turnShare(speed, arc.radius))
    if angle >= arc.turn then return false end
    local ahead = math.min(MAX_ARC_LOOKAHEAD, flight.speed * ARC_LOOKAHEAD_S / arc.radius)
    local x, y, z = SystemPlanner.pointOnArc(arc.centre.x, arc.centre.y, arc.centre.z,
        arc.entry.x, arc.entry.y, arc.entry.z, arc.normal.x, arc.normal.y, arc.normal.z,
        math.min(arc.turn, math.max(angle, 0) + ahead))
    self.nav:setSeek(self.agent, Position(x, y, z), self.passRadius, math.huge)
    return true
end

---Ends the arc: the speed cap is lifted, and the ship goes on to the next corner.
---@param time number
---@param listener PlanFollowerListener
function PlanFollower:endArc(time, listener)
    self.nav:setSpeedCap(self.agent, math.huge)
    listener:arcFlown(self.arc, time)
    self.arc = nil
    self:nextPoint(time, listener)
end

---Flies the ship along its corners: on a leg it is sent to the arc's entry (or the sharp corner) at
---the corner's speed, the points taken from where their bodies are now; on reaching it, it takes
---the arc, then the next leg. Called after each navigation step until `finished`.
---@param arrived boolean The `NavWorld`'s arrival flag for the ship
---@param speed number Game units per second
---@param time number
---@param listener PlanFollowerListener
---@param carried boolean|nil Whether a gravity well carries the ship (it moves with the well's body)
function PlanFollower:advance(arrived, speed, time, listener, carried)
    self.carried = carried == true
    local arc = self:currentArc()
    if self.arc then
        if arc and self:flyArc(arc, speed) then
            self.target = toPosition(arc.exit)
            return
        end
        self:endArc(time, listener)
        if self.finished then return end
        arc = self:currentArc()
    elseif arrived or self:reachedCorner(speed) then
        listener:pointPassed(self.index, speed, time)
        if arc then
            self:beginArc(speed, time)
            if self:flyArc(arc, speed) then
                self.target = toPosition(arc.exit)
                return
            end
            self:endArc(time, listener)
        else
            self:nextPoint(time, listener)
        end
        if self.finished then return end
        arc = self:currentArc()
    end
    self:seekLeg(arc)
end

return PlanFollower
