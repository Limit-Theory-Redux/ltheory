-- Temporary seek-and-arrive steering for the navigation sandbox: turns a ship toward a
-- fixed point and slows it so that drag alone would stop it there, by writing its
-- ControlInput. Nothing but the sandbox uses it; the navigation behaviours replace it.
-- Design: doc/script/navigation-sandbox.md
local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")

---@class SeekArrive
local SeekArrive = {}

-- Axis input per radian of heading error; full input from about half a radian.
local TURN_GAIN        = 2
-- Thrust axes reach full input at this fraction of the profile's speed of velocity error.
local THRUST_ERROR_SPAN = 0.1
-- Forward thrust only when the nose is within this cosine of the wanted velocity change.
local THRUST_ALIGNMENT = 0.7
-- Below this squared length, a direction is too short to normalise.
local TINY_SQ          = 1e-24

local function dot(ax, ay, az, bx, by, bz)
    return ax * bx + ay * by + az * bz
end

local function clampAxis(v)
    return Math.Clamp(v, -1, 1)
end

---Pitch and yaw that turn the nose from `fwd` toward the unit direction `(cx, cy, cz)`.
---@return ControlAxis pitch, ControlAxis yaw
local function turnToward(rot, cx, cy, cz)
    local fwd, rt, up = rot:getForward(), rot:getRight(), rot:getUp()
    -- The rotation axis that brings the nose onto the course, and the angle between them.
    local ax = fwd.y * cz - fwd.z * cy
    local ay = fwd.z * cx - fwd.x * cz
    local az = fwd.x * cy - fwd.y * cx
    local sinAngle = math.sqrt(dot(ax, ay, az, ax, ay, az))
    local angle = math.atan2(sinAngle, dot(fwd.x, fwd.y, fwd.z, cx, cy, cz))
    if sinAngle * sinAngle < TINY_SQ then
        -- On course, or exactly reversed: any axis across the nose turns it.
        ax, ay, az, sinAngle = rt.x, rt.y, rt.z, 1
    end
    ax, ay, az = ax / sinAngle, ay / sinAngle, az / sinAngle
    -- Positive pitch turns about the ship's right axis; positive yaw about its down axis.
    local pitch = clampAxis(TURN_GAIN * angle * dot(ax, ay, az, rt.x, rt.y, rt.z))
    local yaw = clampAxis(-TURN_GAIN * angle * dot(ax, ay, az, up.x, up.y, up.z))
    return pitch, yaw
end

---Write the ship's control input for this frame to fly toward `target` and stop there.
---@param ship Entity A ship with a rigid body, a flight profile and a control input
---@param target Position
function SeekArrive.steer(ship, target)
    local rb = ship:get(PhysicsComponents.RigidBody):getRigidBody()
    local input = ship:get(ConstructComponents.ControlInput)
    local profile = ConstructComponents.FlightProfile.ProfileOf(ship)

    local pos, vel, rot = rb:getPos(), rb:getVelocity(), rb:getRot()
    local dx, dy, dz = target.x - pos.x, target.y - pos.y, target.z - pos.z
    local dist = math.sqrt(dot(dx, dy, dz, dx, dy, dz))
    input:clearAxes()
    if dist * dist < TINY_SQ then return end

    -- Coasting under linear drag k from speed v covers v / k, so speed dist * k stops at the target.
    local wantedSpeed = math.min(profile.maxSpeed, dist * profile.linearDrag)
    local ux, uy, uz = dx / dist, dy / dist, dz / dist
    local ex = ux * wantedSpeed - vel.x
    local ey = uy * wantedSpeed - vel.y
    local ez = uz * wantedSpeed - vel.z
    local errorLength = math.sqrt(dot(ex, ey, ez, ex, ey, ez))

    -- Too fast toward the target: keep the nose on it; drag and reverse thrust slow the ship.
    local cx, cy, cz = ux, uy, uz
    if dot(ex, ey, ez, ux, uy, uz) > 0 and errorLength * errorLength > TINY_SQ then
        cx, cy, cz = ex / errorLength, ey / errorLength, ez / errorLength
    end
    local pitch, yaw = turnToward(rot, cx, cy, cz)
    input:setPitch(pitch)
    input:setYaw(yaw)

    local span = THRUST_ERROR_SPAN * profile.maxSpeed
    local fwd, rt, up = rot:getForward(), rot:getRight(), rot:getUp()
    if dot(fwd.x, fwd.y, fwd.z, cx, cy, cz) > THRUST_ALIGNMENT then
        input:setThrustForward(clampAxis(dot(ex, ey, ez, fwd.x, fwd.y, fwd.z) / span))
    end
    input:setThrustRight(clampAxis(dot(ex, ey, ez, rt.x, rt.y, rt.z) / span))
    input:setThrustUp(clampAxis(dot(ex, ey, ez, up.x, up.y, up.z) / span))
end

---Leave the ship without input, so drag brings it to rest.
---@param ship Entity
function SeekArrive.release(ship)
    ship:get(ConstructComponents.ControlInput):clearAxes()
end

return SeekArrive
