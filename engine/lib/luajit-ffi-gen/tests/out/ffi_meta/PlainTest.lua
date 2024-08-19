---@meta

---@class PlainTest
PlainTest = {}

function PlainTest:func1() end

function PlainTest:func2() end

function PlainTest:privateFunc1() end

function PlainTest.FUNC3() end

---@param val integer
function PlainTest:setU32(val) end

---@return integer
function PlainTest:getU32() end

---@param val number
function PlainTest:setF32Ref(val) end

---@return number
function PlainTest:getF32() end

---@param val ManagedData
function PlainTest:setManaged(val) end

---@param val ManagedData
function PlainTest:setManagedRef(val) end

---@param val ManagedData
function PlainTest:setManagedMut(val) end

---@return ManagedData
function PlainTest:getManaged() end

---@param result ManagedData [out]
function PlainTest:getManagedViaOutParam(result) end

---@return ManagedData
function PlainTest:getManagedRef() end

---@return ManagedData
function PlainTest:getManagedMut() end

---@param val CopyableData
function PlainTest:setCopyable(val) end

---@param val CopyableData
function PlainTest:setCopyableRef(val) end

---@param val CopyableData
function PlainTest:setCopyableMut(val) end

---@return CopyableData
function PlainTest:getCopyable() end

---@param result CopyableData [out]
function PlainTest:getCopyableViaOutParam(result) end

---@return CopyableData
function PlainTest:getCopyableRef() end

---@return CopyableData
function PlainTest:getCopyableMut() end

---@param val string
function PlainTest:setStr(val) end

---@param val string
function PlainTest:setString(val) end

---@param val string
function PlainTest:setStringRef(val) end

---@return string
function PlainTest:getStr() end

---@return string
function PlainTest:getString() end

---@return string
function PlainTest:getStringRef() end

