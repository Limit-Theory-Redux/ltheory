-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class NavWorld
NavWorld = {}

-- Creates an empty world that reads its bodies from `physics`.
---@param physics Physics
---@return NavWorld
function NavWorld.Create(physics) end

-- The number of control axes per agent: the stride of the axes buffer.
---@return integer
function NavWorld.AxisCount() end

-- The number of slots: the agents and the free slots between them. It is the number of
-- entries, per agent, in the buffers `copy_axes` and `copy_arrived` fill.
---@return integer
function NavWorld:slotCount() end

-- Adds an agent steered from `body` and returns its id, its slot: the lowest one that no
-- agent holds, so 0 for the first agent and then consecutive until an agent is removed.
-- The drags are in 1/s; the thrusts (forward, right, up) and torques (pitch, yaw,
-- roll) are the forces and torques the agent applies along and about its own axes at full
-- input. Accelerations follow from the body's mass and inertia at each step, and the top
-- speed is the forward acceleration over the linear drag.
--
-- # Panics
-- If `body` is not a root body of this world's physics.
---@param body RigidBody
---@param linearDrag number
---@param angularDrag number
---@param thrustForward number
---@param thrustRight number
---@param thrustUp number
---@param torquePitch number
---@param torqueYaw number
---@param torqueRoll number
---@return integer
function NavWorld:addAgent(body, linearDrag, angularDrag, thrustForward, thrustRight, thrustUp, torquePitch, torqueYaw, torqueRoll) end

-- Removes the agent and frees its id, which the next [`add_agent`](NavWorld::add_agent) may
-- hand out again, so the caller drops it. The agent's body is left alone in physics, where
-- the other agents sense it as an obstacle from the next step on. An agent that follows or
-- pursues the body keeps doing so, but no longer leaves the body out of its avoidance.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
function NavWorld:removeAgent(id) end

-- Turns sensing on or off. With a non-zero `mask`, each step finds the bodies whose collision
-- group shares a bit with it within each agent's sensing range and treats them as
-- obstacles; 0 turns it off (the default). Agents and their own parts are never sensed.
-- Bodies are found as of the last `Physics::update`.
---@param mask integer
function NavWorld:setSensing(mask) end

-- Sends the agent to `target`, in world coordinates, and stops it within the arrival radius
-- of it (game units). The radius is at least two of the agent's own radii in every
-- behaviour below.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
---@param target Position
---@param arrivalRadius number
function NavWorld:setArrive(id, target, arrivalRadius) end

-- Sends the agent to the point `offset` (game units, world axes) from `body`'s centre, which
-- moves with `body`, and stops it on that point: the goal has the body's velocity, so the
-- agent comes to rest relative to the body. `body` is read from physics at each step. It is
-- only a frame: the agent still avoids it. The offset keeps its world direction while the
-- body turns. Pair it with [`set_frame_velocity`](NavWorld::set_frame_velocity) when the
-- body is carried along by something that drag doesn't act on.
--
-- # Panics
-- If `id` isn't an agent, or `body` isn't a root body of this world's physics.
---@param id integer
---@param body RigidBody
---@param offset Vec3d
---@param arrivalRadius number
function NavWorld:setArriveRelative(id, body, offset, arrivalRadius) end

-- Sets the velocity (game units per second, world axes) that carries the agent along with its
-- surroundings, such as a gravity well's, until it is set again; zero by default. The agent
-- is steered relative to it: drag, top speed, braking and avoidance all follow the velocity
-- the agent has relative to it. Set it before each step it changes.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
---@param velocity Vec3d
function NavWorld:setFrameVelocity(id, velocity) end

-- Makes the agent's nose hold toward `point`, in world coordinates, instead of the facing its
-- behaviour gives, until it is set again or cleared ([`clear_facing`](NavWorld::clear_facing)).
-- It is a point and not a body, so the caller sets it each step for a target that moves. The
-- nose still follows the course while the agent speeds up toward it harder than its side
-- thrusters can, and a point within the agent's own radius gives no facing. A new behaviour
-- leaves the facing as it is.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
---@param point Position
function NavWorld:setFacing(id, point) end

-- Gives the agent's facing back to its behaviour.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
function NavWorld:clearFacing(id) end

-- Sends the agent through `target` at top speed and does not stop it there. It slows so as to
-- pass `target` at `pass_speed` (game units per second, relative to the agent's frame), on
-- the braking curve Arrive uses; an infinite `pass_speed`, or one at or above the agent's top
-- speed, never slows it.
--
-- # Panics
-- If `id` isn't an agent, or `pass_speed` is negative or NaN.
---@param id integer
---@param target Position
---@param arrivalRadius number
---@param passSpeed number
function NavWorld:setSeek(id, target, arrivalRadius, passSpeed) end

-- Limits the speed the agent is planned to fly at, in every behaviour and in avoidance, to
-- `cap` (game units per second, relative to the agent's frame; see
-- [`set_frame_velocity`](NavWorld::set_frame_velocity)), until it is set again. An agent
-- above the cap is braked down to it at full thrust. An infinite cap, the default, lifts
-- the limit.
--
-- # Panics
-- If `id` isn't an agent, or `cap` is negative or NaN.
---@param id integer
---@param cap number
function NavWorld:setSpeedCap(id, cap) end

-- Keeps the agent where its body is now.
--
-- # Panics
-- If `id` isn't an agent, or its body has left the physics world.
---@param id integer
---@param arrivalRadius number
function NavWorld:setHold(id, arrivalRadius) end

-- Meets `target` `stand_off` from its hull (game units), matching its velocity. `target` is
-- read from physics at each step, and the agent doesn't avoid it (it still avoids everything
-- else; if `target` is an agent, it ignores this one while this one is behind it, and avoids
-- it with the half share of any steering agent otherwise).
--
-- # Panics
-- If `id` isn't an agent, or `target` isn't a root body of this world's physics.
---@param id integer
---@param target RigidBody
---@param standOff number
---@param arrivalRadius number
function NavWorld:setPursue(id, target, standOff, arrivalRadius) end

-- Holds the slot `right`, `up`, `forward` (game units) away from `leader`, in the leader's
-- own axes, matching its velocity. `leader` is read from physics at each step, and the agent
-- doesn't avoid it (it still avoids everything else; if `leader` is an agent, it ignores
-- this one while this one is behind it, and avoids it with the half share of any steering
-- agent otherwise).
--
-- # Panics
-- If `id` isn't an agent, or `leader` isn't a root body of this world's physics.
---@param id integer
---@param leader RigidBody
---@param right number
---@param up number
---@param forward number
---@param arrivalRadius number
function NavWorld:setFollow(id, leader, right, up, forward, arrivalRadius) end

-- Circles `centre` at `radius` (game units) in the plane through it normal to `normal`,
-- anticlockwise about `normal`, moving with the centre. `centre` is read from physics at each
-- step, and the agent still avoids it, so an orbit that passes through it is bent.
--
-- # Panics
-- If `id` isn't an agent, `centre` isn't a root body of this world's physics, `radius` isn't
-- positive, or `normal` is zero or not finite.
---@param id integer
---@param centre RigidBody
---@param radius number
---@param normal Vec3d
---@param arrivalRadius number
function NavWorld:setOrbit(id, centre, radius, normal, arrivalRadius) end

-- Leaves the agent without input.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
function NavWorld:clearBehaviour(id) end

-- Sets how agents avoid obstacles: the look-ahead in seconds, the largest clearance in agent
-- radii (the clearance itself grows with the closing speed) and the aim factor (see
-- `doc/engine/nav.md`).
---@param horizonS number
---@param clearanceRadii number
---@param aimFactor number
function NavWorld:setAvoidanceTuning(horizonS, clearanceRadii, aimFactor) end

-- Sets how often the agent's avoidance, and the sensing it needs, runs, in seconds; 0 (the
-- default) runs it at every step. The behaviour, the controller and the arrival test run at
-- every step either way, and between runs the agent keeps the detour the last run gave it.
-- Agents with one period run in different steps. A goal change doesn't bring the next run
-- forward: call [`refresh`](NavWorld::refresh) for that.
--
-- # Panics
-- If `id` isn't an agent, or `seconds` is negative or not finite.
---@param id integer
---@param seconds number
function NavWorld:setAvoidancePeriod(id, seconds) end

-- Makes the agent's avoidance run at the next step and drops the detour it kept. Call it when
-- the agent's goal changes to somewhere else, so the new course isn't flown on the old
-- detour; the goal setters don't, because a caller may repeat them every frame.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
function NavWorld:refresh(id) end

-- The distance (game units) from the agent's centre within which it senses bodies now, from
-- its velocity relative to its frame: the longer it takes to stop, the farther it looks. It
-- is the range [`step`](NavWorld::step) queries, whatever the sensing mask.
--
-- # Panics
-- If `id` isn't an agent, or its body has left the physics world.
---@param id integer
---@return number
function NavWorld:sensingRange(id) end

-- Reads every agent and tracked body from physics, senses obstacles for the agents whose
-- avoidance is due, and computes each agent's control axes and arrival. `dt` is the time in
-- seconds since the last step.
--
-- # Panics
-- If an agent's body or a tracked body has left the physics world, or `dt` is negative or
-- not finite.
---@param dt number
function NavWorld:step(dt) end

-- Copies the control axes of the last step into `out`: one value per axis for each slot, in
-- slot order and in the order of `NavAxis`. A free slot's are zero.
--
-- # Panics
-- If `out` isn't exactly the number of slots times the number of axes long.
---@param out number[]
---@param out_size integer
function NavWorld:copyAxes(out, out_size) end

-- Copies, for each slot in order, whether its agent was within its goal's arrival radius (at
-- least two of its own radii) at the last step. A free slot's is false.
--
-- # Panics
-- If `out` isn't exactly one value per slot long.
---@param out boolean[]
---@param out_size integer
function NavWorld:copyArrived(out, out_size) end
