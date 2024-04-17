---@meta

Trigger = Trigger

---@param half_extents Vec3
---@return Trigger
function Trigger:CreateBox(half_extents) end

---@param parent RigidBody
---@param offset Vec3
function Trigger.attach(self, parent, offset) end

---@param parent RigidBody
function Trigger.detach(self, parent) end

---@param result Box3 [out]
function Trigger.getBoundingBox(self, result) end

---@return integer
function Trigger.getContentsCount(self) end

---Will only include the parent object when a compound is within the trigger.
---@param i integer
---@return RigidBody
function Trigger.getContents(self, i) end

---@param mask integer
function Trigger.setCollisionMask(self, mask) end

---@param pos Vec3
function Trigger.setPos(self, pos) end

---@param pos Vec3
function Trigger.setPosLocal(self, pos) end

---@param result Vec3 [out]
function Trigger.getPos(self, result) end

---@param result Vec3 [out]
function Trigger.getPosLocal(self, result) end

---@return RigidBody
function Trigger.getParent(self) end

