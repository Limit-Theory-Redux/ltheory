---@meta

Physics = Physics

---@return Physics
function Physics:Create() end

---Adds this rigid body to this physics world if it doesn't exist, otherwise do nothing.
---
---Automatically adds all attached Triggers. Automatically adds all
---attached children and their Triggers.
---@param rigid_body RigidBody
function Physics.addRigidBody(self, rigid_body) end

---Removes this rigid body from this physics world if it's added, otherwise do nothing.
---
---Automatically removes all attached Triggers. Automatically removes all
---attached children and their Triggers.
---@param rigid_body RigidBody
function Physics.removeRigidBody(self, rigid_body) end

---@param trigger Trigger
function Physics.addTrigger(self, trigger) end

---@param trigger Trigger
function Physics.removeTrigger(self, trigger) end

---@param dt number
function Physics.update(self, dt) end

---This will fill the collision object with the collision information.
---
---Will include results for both child and parent RigidBodys that are
---colliding. Will not include Triggers.
---@param iterator Collision
---@return boolean
function Physics.getNextCollision(self, iterator) end

---@param ray Ray
---@param result RayCastResult [out]
function Physics.rayCast(self, ray, result) end

---Results are unsorted and will include child objects.
---
---The array stored inside ShapeCastResult is valid until the next call to sphere_cast.
---@param sphere Sphere
---@param result ShapeCastResult [out]
function Physics.sphereCast(self, sphere, result) end

---Results are unsorted and will include child objects.
---
---The array stored inside ShapeCastResult is valid until the next call to box_cast.
---@param pos Vec3
---@param rot Quat
---@param half_extents Vec3
---@param result ShapeCastResult [out]
function Physics.boxCast(self, pos, rot, half_extents, result) end

---@param sphere Sphere
---@return boolean
function Physics.sphereOverlap(self, sphere) end

---@param pos Vec3
---@param rot Quat
---@param half_extents Vec3
---@return boolean
function Physics.boxOverlap(self, pos, rot, half_extents) end

function Physics.drawBoundingBoxesLocal(self) end

function Physics.drawBoundingBoxesWorld(self) end

function Physics.drawWireframes(self) end

