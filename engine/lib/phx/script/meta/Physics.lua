-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Physics
Physics = {}

---@return Physics
function Physics.Create() end

-- Sets the typical object size in game units that Rapier scales its internal
-- tolerances by, such as the allowed overlap and the speed below which a body may
-- sleep. Rapier's default is 1. Panics unless positive.
---@param lengthUnit number
function Physics:setLengthUnit(lengthUnit) end

-- Sets the natural frequency of contact constraints, in hertz: higher values push
-- overlapping bodies apart faster. Rapier's default is 30. Panics unless positive.
---@param frequency number
function Physics:setContactNaturalFrequency(frequency) end

-- Adds this rigid body to this physics world if it doesn't exist, otherwise do nothing.
-- 
-- Automatically adds all attached Triggers. Automatically adds all
-- attached children and their Triggers.
---@param rigidBody RigidBody
function Physics:addRigidBody(rigidBody) end

-- Removes this rigid body from this physics world if it's added, otherwise do nothing.
-- 
-- Automatically removes all attached Triggers. Automatically removes all
-- attached children and their Triggers.
---@param rigidBody RigidBody
function Physics:removeRigidBody(rigidBody) end

---@param trigger Trigger
function Physics:addTrigger(trigger) end

---@param trigger Trigger
function Physics:removeTrigger(trigger) end

---@param dt number
function Physics:update(dt) end

-- This will fill the collision object with the collision information.
-- 
-- Will include results for both child and parent RigidBodys that are
-- colliding. Will not include Triggers.
---@param iterator Collision
---@return boolean
function Physics:getNextCollision(iterator) end

-- The first body the ray hits between `t_min` and `t_max`, including non-collidable ones.
---@param ray Ray
---@param result RayCastResult [out]
function Physics:rayCast(ray, result) end

-- The first body the ray hits between `t_min` and `t_max`, skipping the compound
-- `exclude` belongs to, and hitting only bodies whose collision group shares a bit
-- with `mask`.
---@param ray Ray
---@param exclude RigidBody?
---@param mask integer
---@param result RayCastResult [out]
function Physics:rayCastFiltered(ray, exclude, mask, result) end

-- The bodies a sphere at `pos` overlaps, unsorted, including child bodies.
-- 
-- The arrays in the result are valid until the next call to sphere_cast.
---@param pos Position
---@param radius number
---@param result ShapeCastResult [out]
function Physics:sphereCast(pos, radius, result) end

-- The bodies a box at `pos` overlaps, unsorted, including child bodies.
-- 
-- The arrays in the result are valid until the next call to box_cast.
---@param pos Position
---@param rot Quat
---@param halfExtents Vec3f
---@param result ShapeCastResult [out]
function Physics:boxCast(pos, rot, halfExtents, result) end

---@param pos Position
---@param radius number
---@return boolean
function Physics:sphereOverlap(pos, radius) end

---@param pos Position
---@param rot Quat
---@param halfExtents Vec3f
---@return boolean
function Physics:boxOverlap(pos, rot, halfExtents) end

-- The first body a sphere of `radius` touches while its centre moves from `from` to
-- `to`, skipping the compound `exclude` belongs to, and hitting only bodies whose
-- collision group shares a bit with `mask`. A sphere that starts overlapping a body
-- hits it at distance 0.
---@param from Position
---@param to Position
---@param radius number
---@param exclude RigidBody?
---@param mask integer
---@param result SphereSweepResult [out]
function Physics:sphereSweep(from, to, radius, exclude, mask, result) end

-- The bodies within `radius` of `pos`, nearest first, skipping the compound `exclude`
-- belongs to, and including only bodies whose collision group shares a bit with `mask`.
-- Child bodies are listed on their own; a body without a user id is listed with id 0.
-- 
-- The arrays in the result are valid until the next call to query_radius.
---@param pos Position
---@param radius number
---@param exclude RigidBody?
---@param mask integer
---@param result RadiusQueryResult [out]
function Physics:queryRadius(pos, radius, exclude, mask, result) end

function Physics:drawBoundingBoxesLocal() end

function Physics:drawBoundingBoxesWorld() end

---@param r Renderer
---@param shader Shader
---@param eye Position
function Physics:drawWireframes(r, shader, eye) end

---@param r Renderer
---@param shader Shader
---@param eye Position
---@param maxRange number
function Physics:drawWireframesInRange(r, shader, eye, maxRange) end

