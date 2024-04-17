---@meta

---@class Physics
Physics = {}

---@return Physics
function Physics.Create() end

---Adds this rigid body to this physics world if it doesn't exist, otherwise do nothing.
---
---Automatically adds all attached Triggers. Automatically adds all
---attached children and their Triggers.
---@param rigid_body RigidBody
function Physics:addRigidBody(rigid_body) end

---Removes this rigid body from this physics world if it's added, otherwise do nothing.
---
---Automatically removes all attached Triggers. Automatically removes all
---attached children and their Triggers.
---@param rigid_body RigidBody
function Physics:removeRigidBody(rigid_body) end

---@param trigger Trigger
function Physics:addTrigger(trigger) end

---@param trigger Trigger
function Physics:removeTrigger(trigger) end

---@param dt number
function Physics:update(dt) end

---This will fill the collision object with the collision information.
---
---Will include results for both child and parent RigidBodys that are
---colliding. Will not include Triggers.
---@param iterator Collision
---@return boolean
function Physics:getNextCollision(iterator) end

---@param ray Ray
---@param result RayCastResult [out]
function Physics:rayCast(ray, result) end

---Results are unsorted and will include child objects.
---
---The array stored inside ShapeCastResult is valid until the next call to sphere_cast.
---@param sphere Sphere
---@param result ShapeCastResult [out]
function Physics:sphereCast(sphere, result) end

---Results are unsorted and will include child objects.
---
---The array stored inside ShapeCastResult is valid until the next call to box_cast.
---@param pos Vec3
---@param rot Quat
---@param half_extents Vec3
---@param result ShapeCastResult [out]
function Physics:boxCast(pos, rot, half_extents, result) end

---@param sphere Sphere
---@return boolean
function Physics:sphereOverlap(sphere) end

---@param pos Vec3
---@param rot Quat
---@param half_extents Vec3
---@return boolean
function Physics:boxOverlap(pos, rot, half_extents) end

function Physics:drawBoundingBoxesLocal() end

function Physics:drawBoundingBoxesWorld() end

function Physics:drawWireframes() end

