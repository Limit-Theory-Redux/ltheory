-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Quat
Quat = {}

---@param x number
---@param y number
---@param z number
---@param w number
---@return Quat
function Quat.Create(x, y, z, w) end

---@return Quat
function Quat.Identity() end

---@param result Vec3f [out]
function Quat:getAxisX(result) end

---@param result Vec3f [out]
function Quat:getAxisY(result) end

---@param result Vec3f [out]
function Quat:getAxisZ(result) end

---@param result Vec3f [out]
function Quat:getForward(result) end

---@param result Vec3f [out]
function Quat:getRight(result) end

---@param result Vec3f [out]
function Quat:getUp(result) end

---@return Quat
function Quat:canonicalize() end

function Quat:iCanonicalize() end

---@param p Quat
---@return number
function Quat:dot(p) end

---@param p Quat
---@return boolean
function Quat:equal(p) end

---@param p Quat
---@return boolean
function Quat:approximatelyEqual(p) end

---@return Quat
function Quat:inverse() end

function Quat:iInverse() end

---@param p Quat
---@param t number
---@return Quat
function Quat:lerp(p, t) end

---@param p Quat
---@param t number
function Quat:iLerp(p, t) end

---@param p Quat
---@return Quat
function Quat:mul(p) end

---@param p Quat
function Quat:iMul(p) end

---@param v Vec3f
---@param result Vec3f [out]
function Quat:mulV(v, result) end

---@return Quat
function Quat:normalize() end

function Quat:iNormalize() end

---@param scale number
---@return Quat
function Quat:scale(scale) end

---@param scale number
function Quat:iScale(scale) end

---@param p Quat
---@param t number
---@return Quat
function Quat:slerp(p, t) end

---@param p Quat
---@param t number
function Quat:iSlerp(p, t) end

---@param axis Vec3f
---@param radians number
---@return Quat
function Quat.FromAxisAngle(axis, radians) end

---@param yaw number
---@param pitch number
---@param roll number
---@return Quat
function Quat.FromEuler(yaw, pitch, roll) end

---@param forward Vec3f
---@param up Vec3f
---@return Quat
function Quat.FromLook(forward, up) end

---@param eye Vec3f
---@param target Vec3f
---@param up Vec3f
---@return Quat
function Quat.LookAt(eye, target, up) end

---@param from Vec3f
---@param to Vec3f
---@return Quat
function Quat.FromRotateTo(from, to) end

---@return string
function Quat:toString() end

---@return Error
function Quat:validate() end

