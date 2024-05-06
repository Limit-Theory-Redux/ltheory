---@meta

---@class GuiPropertyValue
GuiPropertyValue = {}

---@return HmGuiPropertyType
function GuiPropertyValue:getType() end

---@param value boolean
---@return GuiPropertyValue
function GuiPropertyValue.FromBool(value) end

---@return boolean
function GuiPropertyValue:getBool() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromI8(value) end

---@return integer
function GuiPropertyValue:getI8() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromU8(value) end

---@return integer
function GuiPropertyValue:getU8() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromI16(value) end

---@return integer
function GuiPropertyValue:getI16() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromU16(value) end

---@return integer
function GuiPropertyValue:getU16() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromI32(value) end

---@return integer
function GuiPropertyValue:getI32() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromU32(value) end

---@return integer
function GuiPropertyValue:getU32() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromI64(value) end

---@return integer
function GuiPropertyValue:getI64() end

---@param value integer
---@return GuiPropertyValue
function GuiPropertyValue.FromU64(value) end

---@return integer
function GuiPropertyValue:getU64() end

---@param value number
---@return GuiPropertyValue
function GuiPropertyValue.FromF32(value) end

---@return number
function GuiPropertyValue:getF32() end

---@param value number
---@return GuiPropertyValue
function GuiPropertyValue.FromF64(value) end

---@return number
function GuiPropertyValue:getF64() end

---@param value Vec2
---@return GuiPropertyValue
function GuiPropertyValue.FromVec2(value) end

---@return Vec2
function GuiPropertyValue:getVec2() end

---@param value Vec3
---@return GuiPropertyValue
function GuiPropertyValue.FromVec3(value) end

---@return Vec3
function GuiPropertyValue:getVec3() end

---@param value Vec4
---@return GuiPropertyValue
function GuiPropertyValue.FromVec4(value) end

---@return Vec4
function GuiPropertyValue:getVec4() end

---@param value IVec2
---@return GuiPropertyValue
function GuiPropertyValue.FromIvec2(value) end

---@return IVec2
function GuiPropertyValue:getIvec2() end

---@param value IVec3
---@return GuiPropertyValue
function GuiPropertyValue.FromIvec3(value) end

---@return IVec3
function GuiPropertyValue:getIvec3() end

---@param value IVec4
---@return GuiPropertyValue
function GuiPropertyValue.FromIvec4(value) end

---@return IVec4
function GuiPropertyValue:getIvec4() end

---@param value UVec2
---@return GuiPropertyValue
function GuiPropertyValue.FromUvec2(value) end

---@return UVec2
function GuiPropertyValue:getUvec2() end

---@param value UVec3
---@return GuiPropertyValue
function GuiPropertyValue.FromUvec3(value) end

---@return UVec3
function GuiPropertyValue:getUvec3() end

---@param value UVec4
---@return GuiPropertyValue
function GuiPropertyValue.FromUvec4(value) end

---@return UVec4
function GuiPropertyValue:getUvec4() end

---@param value DVec2
---@return GuiPropertyValue
function GuiPropertyValue.FromDvec2(value) end

---@return DVec2
function GuiPropertyValue:getDvec2() end

---@param value DVec3
---@return GuiPropertyValue
function GuiPropertyValue.FromDvec3(value) end

---@return DVec3
function GuiPropertyValue:getDvec3() end

---@param value DVec4
---@return GuiPropertyValue
function GuiPropertyValue.FromDvec4(value) end

---@return DVec4
function GuiPropertyValue:getDvec4() end

---@param value Color
---@return GuiPropertyValue
function GuiPropertyValue.FromColor(value) end

---@return Color
function GuiPropertyValue:getColor() end

---@param value Box3
---@return GuiPropertyValue
function GuiPropertyValue.FromBox3(value) end

---@return Box3
function GuiPropertyValue:getBox3() end

---@param value string
---@return GuiPropertyValue
function GuiPropertyValue.FromString(value) end

---@return string
function GuiPropertyValue:getString() end

---@param value Font
---@return GuiPropertyValue
function GuiPropertyValue.FromFont(value) end

---@return Font
function GuiPropertyValue:getFont() end

