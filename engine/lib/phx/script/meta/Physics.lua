---@meta

---@class Physics
Physics = {}

---@return Physics
function Physics.Create() end

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

---@param ray Ray
---@param result RayCastResult [out]
function Physics:rayCast(ray, result) end

-- Results are unsorted and will include child objects.
-- 
-- The array stored inside ShapeCastResult is valid until the next call to sphere_cast.
---@param sphere Sphere
---@param result ShapeCastResult [out]
function Physics:sphereCast(sphere, result) end

-- Results are unsorted and will include child objects.
-- 
-- The array stored inside ShapeCastResult is valid until the next call to box_cast.
---@param pos Vec3f
---@param rot Quat
---@param halfExtents Vec3f
---@param result ShapeCastResult [out]
function Physics:boxCast(pos, rot, halfExtents, result) end

---@param sphere Sphere
---@return boolean
function Physics:sphereOverlap(sphere) end

---@param pos Vec3f
---@param rot Quat
---@param halfExtents Vec3f
---@return boolean
function Physics:boxOverlap(pos, rot, halfExtents) end

function Physics:drawBoundingBoxesLocal() end

function Physics:drawBoundingBoxesWorld() end

function Physics:drawWireframes() end

