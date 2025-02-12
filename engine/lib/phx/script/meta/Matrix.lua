-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Matrix
Matrix = {}

---@param other Matrix
---@return boolean
function Matrix:equal(other) end

---@param other Matrix
---@return boolean
function Matrix:approximatelyEqual(other) end

---@return Matrix
function Matrix:inverse() end

---@return Matrix
function Matrix:inverseTranspose() end

---@param other Matrix
---@return Matrix
function Matrix:sum(other) end

---@return Matrix
function Matrix:transpose() end

function Matrix:iInverse() end

---@param scale number
function Matrix:iScale(scale) end

function Matrix:iTranspose() end

---@return Matrix
function Matrix.Identity() end

---@param pos Vec3f
---@param at Vec3f
---@param up Vec3f
---@return Matrix
function Matrix.LookAt(pos, at, up) end

---@param pos Vec3f
---@param look Vec3f
---@param up Vec3f
---@return Matrix
function Matrix.LookUp(pos, look, up) end

---@param degreesFovy number
---@param aspect number
---@param n number
---@param f number
---@return Matrix
function Matrix.Perspective(degreesFovy, aspect, n, f) end

---@param other Matrix
---@return Matrix
function Matrix:product(other) end

---@param rads number
---@return Matrix
function Matrix.RotationX(rads) end

---@param rads number
---@return Matrix
function Matrix.RotationY(rads) end

---@param rads number
---@return Matrix
function Matrix.RotationZ(rads) end

---@param sx number
---@param sy number
---@param sz number
---@return Matrix
function Matrix.Scaling(sx, sy, sz) end

---@param sx number
---@param sy number
---@param sz number
---@param ry number
---@param rp number
---@param rr number
---@param tx number
---@param ty number
---@param tz number
---@return Matrix
function Matrix.SRT(sx, sy, sz, ry, rp, rr, tx, ty, tz) end

---@param tx number
---@param ty number
---@param tz number
---@return Matrix
function Matrix.Translation(tx, ty, tz) end

---@param yaw number
---@param pitch number
---@param roll number
---@return Matrix
function Matrix.YawPitchRoll(yaw, pitch, roll) end

---@param in0 Box3f
---@return Box3f
function Matrix:mulBox(in0) end

---@param d Vec3f
---@return Vec3f
function Matrix:mulDir(d) end

---@param p Vec3f
---@return Vec3f
function Matrix:mulPoint(p) end

---@param v Vec4f
---@return Vec4f
function Matrix:mulVec(v) end

---@return Vec3f
function Matrix:getForward() end

---@return Vec3f
function Matrix:getRight() end

---@return Vec3f
function Matrix:getUp() end

---@return Vec3f
function Matrix:getPos() end

---@param row integer
---@return Vec4f
function Matrix:getRow(row) end

---@param x Vec3f
---@param y Vec3f
---@param z Vec3f
---@return Matrix
function Matrix.FromBasis(x, y, z) end

---@param pos Vec3f
---@param rot Quat
---@return Matrix
function Matrix.FromPosRot(pos, rot) end

---@param pos Vec3f
---@param rot Quat
---@param scale number
---@return Matrix
function Matrix.FromPosRotScale(pos, rot, scale) end

---@param pos Vec3f
---@param x Vec3f
---@param y Vec3f
---@param z Vec3f
---@return Matrix
function Matrix.FromPosBasis(pos, x, y, z) end

---@param q Quat
---@return Matrix
function Matrix.FromQuat(q) end

---@return Quat
function Matrix:toQuat() end

function Matrix:print() end

---@return string
function Matrix:toString() end

