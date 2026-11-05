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
-- consecutive. The speed limit is in game units per second and the linear drag in 1/s.
-- 
-- # Panics
-- If `body` is not a root body of this world's physics.
---@param body RigidBody
---@param maxSpeed number
---@param linearDrag number
---@return integer
function NavWorld:addAgent(body, maxSpeed, linearDrag) end

-- Adds a sphere that every agent steers around: the body's position, velocity and bounding
-- radius at each step.
-- 
-- # Panics
-- If `body` is not a root body of this world's physics.
---@param body RigidBody
function NavWorld:addObstacle(body) end

-- Sends the agent to `target`, in world coordinates, and stops it within the arrival radius
-- of it (game units).
-- 
-- # Panics
-- If `id` isn't an agent.
---@param id integer
---@param target Position
---@param arrivalRadius number
function NavWorld:setGoal(id, target, arrivalRadius) end

-- Leaves the agent without input.
-- 
-- # Panics
-- If `id` isn't an agent.
---@param id integer
function NavWorld:clearGoal(id) end

-- Sets how agents avoid obstacles: the look-ahead in seconds, the largest clearance in agent
-- radii (the clearance itself grows with the closing speed) and the aim factor (see
-- `doc/engine/nav.md`).
---@param horizonS number
---@param clearanceRadii number
---@param aimFactor number
function NavWorld:setAvoidanceTuning(horizonS, clearanceRadii, aimFactor) end

-- Reads every registered body from physics and computes each agent's control axes and
-- arrival.
-- 
-- # Panics
-- If a registered body has left the physics world.
function NavWorld:step() end

-- Copies the control axes of the last step into `out`: one value per axis for each agent, in
-- agent order and in the order of `NavAxis`.
-- 
-- # Panics
-- If `out` isn't exactly the number of agents times the number of axes long.
---@param out number[]
---@param out_size integer
function NavWorld:copyAxes(out, out_size) end

-- Copies, for each agent in order, whether it was within its goal's arrival radius at the
-- last step.
-- 
-- # Panics
-- If `out` isn't exactly one value per agent long.
---@param out boolean[]
---@param out_size integer
function NavWorld:copyArrived(out, out_size) end

