---@meta

---@class Trigger
Trigger = {}

---@param halfExtents Vec3f
---@return Trigger
function Trigger.CreateBox(halfExtents) end

---@param parent RigidBody
---@param offset Vec3f
function Trigger:attach(parent, offset) end

---@param parent RigidBody
function Trigger:detach(parent) end

---@param result Box3f [out]
function Trigger:getBoundingBox(result) end

---@return integer
function Trigger:getContentsCount() end

-- Will only include the parent object when a compound is within the trigger.
---@param i integer
---@return RigidBody
function Trigger:getContents(i) end

---@param mask integer
function Trigger:setCollisionMask(mask) end

---@param pos Position
function Trigger:setPos(pos) end

---@param pos Position
function Trigger:setPosLocal(pos) end

---@param result Position [out]
function Trigger:getPos(result) end

---@param result Position [out]
function Trigger:getPosLocal(result) end

---@return RigidBody
function Trigger:getParent() end

