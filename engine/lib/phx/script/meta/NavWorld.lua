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

-- The number of agents.
---@return integer
function NavWorld:agentCount() end

-- Adds an agent steered from `body` and returns its id: 0 for the first agent, then
-- consecutive. The drags are in 1/s; the thrusts (forward, right, up) and torques (pitch, yaw,
-- roll) are the forces and torques the ship applies along and about its own axes at full
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

-- Sends the agent through `target` at top speed, without braking.
--
-- # Panics
-- If `id` isn't an agent.
---@param id integer
---@param target Position
---@param arrivalRadius number
function NavWorld:setSeek(id, target, arrivalRadius) end

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

-- Reads every agent and tracked body from physics, senses obstacles, and computes each
-- agent's control axes and arrival.
--
-- # Panics
-- If an agent's body or a tracked body has left the physics world.
function NavWorld:step() end

-- Copies the control axes of the last step into `out`: one value per axis for each agent, in
-- agent order and in the order of `NavAxis`.
--
-- # Panics
-- If `out` isn't exactly the number of agents times the number of axes long.
---@param out number[]
---@param out_size integer
function NavWorld:copyAxes(out, out_size) end

-- Copies, for each agent in order, whether it was within its goal's arrival radius (at least
-- two of its own radii) at the last step.
--
-- # Panics
-- If `out` isn't exactly one value per agent long.
---@param out boolean[]
---@param out_size integer
function NavWorld:copyArrived(out, out_size) end
