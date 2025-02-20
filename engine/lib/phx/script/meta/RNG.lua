-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RNG
RNG = {}

---@param seed integer
---@return RNG
function RNG.Create(seed) end

---@param s string
---@return RNG
function RNG.FromStr(s) end

---@return RNG
function RNG.FromTime() end

function RNG:rewind() end

---@param probability number
---@return boolean
function RNG:chance(probability) end

---@return integer
function RNG:get31() end

---@return integer
function RNG:get32() end

---@return integer
function RNG:get64() end

---@return number
function RNG:getAngle() end

---@param lower integer
---@param upper integer
---@return integer
function RNG:getInt(lower, upper) end

---@return RNG
function RNG:getRNG() end

---@return number
function RNG:getUniform() end

---@param lower number
---@param upper number
---@return number
function RNG:getUniformRange(lower, upper) end

---@param k integer
---@return number
function RNG:getErlang(k) end

---@return number
function RNG:getExp() end

---@return number
function RNG:getGaussian() end

---@param out Vec2f
function RNG:getAxis2(out) end

---@param out Vec3f
function RNG:getAxis3(out) end

---@param out Vec2f
function RNG:getDir2(out) end

---@param out Vec3f
function RNG:getDir3(out) end

---@param out Vec2f
function RNG:getDisc(out) end

---@return number
function RNG:getSign() end

---@param out Vec3f
function RNG:getSphere(out) end

---@param out Vec2f
---@param lower number
---@param upper number
function RNG:getVec2(out, lower, upper) end

---@param out Vec3f
---@param lower number
---@param upper number
function RNG:getVec3(out, lower, upper) end

---@param out Vec4f
---@param lower number
---@param upper number
function RNG:getVec4(out, lower, upper) end

---@param out Quat
function RNG:getQuat(out) end

