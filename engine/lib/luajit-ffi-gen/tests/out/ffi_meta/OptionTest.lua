---@meta

---@class OptionTest
OptionTest = {}

---@param val integer|nil
function OptionTest:setPrimitive(val) end

---@param val integer|nil
function OptionTest:setPrimitiveRef(val) end

---@param val integer|nil
function OptionTest:setPrimitiveMut(val) end

---@return integer|nil
function OptionTest:getPrimitive() end

---@return integer|nil
function OptionTest:getPrimitiveRef() end

---@return integer|nil
function OptionTest:getPrimitiveMut() end

---@param val ManagedData|nil
function OptionTest:setManaged(val) end

---@param val ManagedData|nil
function OptionTest:setManagedRef(val) end

---@param val ManagedData|nil
function OptionTest:setManagedMut(val) end

---@return ManagedData|nil
function OptionTest:getManaged() end

---@return ManagedData|nil
function OptionTest:getManagedRef() end

---@return ManagedData|nil
function OptionTest:getManagedMut() end

---@param val CopyableData|nil
function OptionTest:setCopyable(val) end

---@param val CopyableData|nil
function OptionTest:setCopyableRef(val) end

---@param val CopyableData|nil
function OptionTest:setCopyableMut(val) end

---@return CopyableData|nil
function OptionTest:getCopyable() end

---@return CopyableData|nil
function OptionTest:getCopyableRef() end

---@return CopyableData|nil
function OptionTest:getCopyableMut() end

---@param val string|nil
function OptionTest:setStr(val) end

---@param val string|nil
function OptionTest:setString(val) end

---@param val string|nil
function OptionTest:setStringRef(val) end

---@return string|nil
function OptionTest:getStr() end

---@return string|nil
function OptionTest:getString() end

---@return string|nil
function OptionTest:getStringRef() end

