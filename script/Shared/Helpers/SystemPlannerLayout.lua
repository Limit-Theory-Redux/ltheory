-- System planner kernel, part 1 of 6: the plan's status codes, the layouts of its buffers and the
-- small helpers every part shares. Pure data and functions, no scratch state.
-- Design: doc/script/nav/system-planner.md.

local abs = math.abs

local Layout = {}

---Result of a plan. The waypoints written are valid for every status except `InvalidInput`,
---`EndpointInsideBody`, `GoalOutrun` and `CapacityExceeded` (where they are the ones written
---before the stop).
Layout.Status = {
    Ok                 = 0,
    ---A leg still crosses a safe sphere at the depth limit; that leg is left straight.
    DepthExceeded      = 1,
    CapacityExceeded   = 2,
    ---The start or the goal lies inside a body's hull.
    EndpointInsideBody = 3,
    InvalidInput       = 4,
    ---A leg crosses a safe sphere and no tangent exists (an end lies inside it); left straight.
    NoDetour           = 5,
    ---The goal's frame body can move as fast as the ship can fly to the body's well (or, for a
    ---goal outside a well, to the goal), so the ship may never catch it.
    GoalOutrun         = 6,
    ---The arrival time at a goal on a moving body did not settle within the iteration limit; the
    ---last iteration's plan is kept, and its times may be off.
    GoalNotSettled     = 7,
    ---A leg needed more drive timeline steps than the limit; it is planned without the drive.
    DriveNotTimed      = 8,
}

-- Body buffer: BODY_STRIDE doubles per body, in the order of these offsets. A parent comes before
-- its children. A body with an orbit radius above zero circles its parent's position (its own
-- position without a parent); any other body stays at its position. A body with a zone radius
-- above zero caps the travel drive near it (`TravelDriveMath.zoneCap`): its zone cap is the
-- multiplier at its centre.
Layout.BODY_STRIDE      = 12
Layout.PARENT           = 0 -- index of the parent body, -1 for none
Layout.POS_X            = 1
Layout.POS_Y            = 2
Layout.POS_Z            = 3
Layout.ORBIT_RADIUS     = 4
Layout.PHASE            = 5
Layout.ANGULAR_SPEED    = 6
Layout.INCLINATION      = 7
Layout.BODY_RADIUS      = 8
Layout.SAFE_RADIUS      = 9
Layout.ZONE_RADIUS      = 10
Layout.ZONE_CAP         = 11
-- Output buffer: OUT_STRIDE doubles per waypoint, the goal last. A waypoint is the index of its
-- frame body (SYSTEM_FRAME for the system) and the offset from that body's centre, in world axes,
-- and `t`, the time the ship reaches it. The frame's position at `t` plus the offset is the point.
-- Radius and speed describe the corner there: the radius of the arc that rounds it (zero for a
-- sharp corner) and the speed the ship is to pass at. `drive` is 1 when the leg that ends at the
-- waypoint is flown under the travel drive, which is released before the waypoint. `through` is 1
-- when the drive is not released there: the leg that ends at the waypoint and the leg after it are
-- one drive run, and the waypoint's speed is the run's.
Layout.OUT_FRAME        = 0
Layout.OUT_X            = 1
Layout.OUT_Y            = 2
Layout.OUT_Z            = 3
Layout.OUT_T            = 4
Layout.OUT_RADIUS       = 5
Layout.OUT_SPEED        = 6
Layout.OUT_DRIVE        = 7
Layout.OUT_THROUGH      = 8
Layout.OUT_STRIDE       = 9
Layout.SYSTEM_FRAME     = -1
-- Drive buffer: DRIVE_STRIDE doubles, in the order of these offsets. The multiplier in open
-- space, the seconds the drive charges at normal speed, and the rates at which its multiplier
-- rises (per second, times the multiplier above 1) and falls (the rate of an exponential), and
-- the length below which a leg is never flown under the drive.
Layout.DRIVE_OPEN_CAP   = 0
Layout.DRIVE_CHARGE     = 1
Layout.DRIVE_RAMP_UP    = 2
Layout.DRIVE_RAMP_DOWN  = 3
Layout.DRIVE_MIN_LENGTH = 4
Layout.DRIVE_STRIDE     = 5
-- Zone buffer: ZONE_STRIDE doubles per cost zone, in the order of these offsets. A zone is a sphere
-- of `radius` at the offset (x, y, z) from the centre of body `frame` (the point itself for
-- SYSTEM_FRAME), and each metre travelled inside costs `cost` extra. A zone has no hull: a leg is
-- bent around it only where that is cheaper than crossing it, and it never fails a plan.
Layout.ZONE_FRAME       = 0
Layout.ZONE_X           = 1
Layout.ZONE_Y           = 2
Layout.ZONE_Z           = 3
Layout.ZONE_RADIUS_AT   = 4
Layout.ZONE_COST        = 5
Layout.ZONE_STRIDE      = 6

Layout.Body = {
    Parent = Layout.PARENT, X = Layout.POS_X, Y = Layout.POS_Y, Z = Layout.POS_Z,
    OrbitRadius = Layout.ORBIT_RADIUS, Phase = Layout.PHASE, AngularSpeed = Layout.ANGULAR_SPEED,
    Inclination = Layout.INCLINATION, Radius = Layout.BODY_RADIUS, SafeRadius = Layout.SAFE_RADIUS,
    ZoneRadius = Layout.ZONE_RADIUS, ZoneCap = Layout.ZONE_CAP,
}
Layout.Out = {
    Frame = Layout.OUT_FRAME, X = Layout.OUT_X, Y = Layout.OUT_Y, Z = Layout.OUT_Z, T = Layout.OUT_T,
    Radius = Layout.OUT_RADIUS, Speed = Layout.OUT_SPEED, Drive = Layout.OUT_DRIVE,
    Through = Layout.OUT_THROUGH,
}
Layout.Drive = {
    OpenCap = Layout.DRIVE_OPEN_CAP, Charge = Layout.DRIVE_CHARGE, RampUp = Layout.DRIVE_RAMP_UP,
    RampDown = Layout.DRIVE_RAMP_DOWN, MinLength = Layout.DRIVE_MIN_LENGTH,
}
Layout.Zone = {
    Frame = Layout.ZONE_FRAME, X = Layout.ZONE_X, Y = Layout.ZONE_Y, Z = Layout.ZONE_Z,
    Radius = Layout.ZONE_RADIUS_AT, Cost = Layout.ZONE_COST,
}

-- Distances below this make a direction undefined.
Layout.MIN_LENGTH = 1e-9
Layout.HALF       = 0.5

---Whether `value` is a number that is neither NaN nor infinite.
function Layout.isFinite(value)
    return value == value and abs(value) ~= math.huge
end

return Layout
