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

---@param result Vec2f [out]
function RNG:getAxis2(result) end

---@param result Vec3f [out]
function RNG:getAxis3(result) end

---@param result Vec2f [out]
function RNG:getDir2(result) end

---@param result Vec3f [out]
function RNG:getDir3(result) end

---@param result Vec2f [out]
function RNG:getDisc(result) end

---@return number
function RNG:getSign() end

---@param result Vec3f [out]
function RNG:getSphere(result) end

---@param lower number
---@param upper number
---@param result Vec2f [out]
function RNG:getVec2(lower, upper, result) end

---@param lower number
---@param upper number
---@param result Vec3f [out]
function RNG:getVec3(lower, upper, result) end

---@param lower number
---@param upper number
---@param result Vec4f [out]
function RNG:getVec4(lower, upper, result) end

---@param result Quat [out]
function RNG:getQuat(result) end

