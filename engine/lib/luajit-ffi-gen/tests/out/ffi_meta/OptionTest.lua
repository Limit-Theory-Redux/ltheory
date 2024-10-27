---@meta

---@class OptionTest
OptionTest = {}

---@param val integer?
function OptionTest:setPrimitive(val) end

---@param val integer?
function OptionTest:setPrimitiveRef(val) end

---@param val integer?
function OptionTest:setPrimitiveMut(val) end

---@return integer?
function OptionTest:getPrimitive() end

---@return integer?
function OptionTest:getPrimitiveRef() end

---@return integer?
function OptionTest:getPrimitiveMut() end

---@param val ManagedData?
function OptionTest:setManaged(val) end

---@param val ManagedData?
function OptionTest:setManagedRef(val) end

---@param val ManagedData?
function OptionTest:setManagedMut(val) end

---@return ManagedData?
function OptionTest:getManaged() end

---@return ManagedData?
function OptionTest:getManagedRef() end

---@return ManagedData?
function OptionTest:getManagedMut() end

---@param val CopyableData?
function OptionTest:setCopyable(val) end

---@param val CopyableData?
function OptionTest:setCopyableRef(val) end

---@param val CopyableData?
function OptionTest:setCopyableMut(val) end

---@return CopyableData?
function OptionTest:getCopyable() end

---@return CopyableData?
function OptionTest:getCopyableRef() end

---@return CopyableData?
function OptionTest:getCopyableMut() end

---@param val string?
function OptionTest:setStr(val) end

---@param val string?
function OptionTest:setString(val) end

---@param val string?
function OptionTest:setStringRef(val) end

---@return string?
function OptionTest:getStr() end

---@return string?
function OptionTest:getString() end

---@return string?
function OptionTest:getStringRef() end

