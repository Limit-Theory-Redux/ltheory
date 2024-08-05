---@meta

---@class ImplTest
ImplTest = {}

function ImplTest:func1() end

function ImplTest:func2() end

function ImplTest:privateFunc1() end

function ImplTest.FUNC3() end

---@param val integer
function ImplTest:setU32(val) end

---@return integer
function ImplTest:getU32() end

---@param val number
function ImplTest:setF32(val) end

---@return number
function ImplTest:getF32() end

---@param val string
function ImplTest:setStr(val) end

---@return string
function ImplTest:getStr() end

---@param val Data
function ImplTest:setData(val) end

---@param val Data
function ImplTest:takeData(val) end

---@param val Data
function ImplTest:takeBoxedData(val) end

---@return Data
function ImplTest:getData() end

---@param result Data [out]
function ImplTest:getDataViaOutParam(result) end

---@return Data
function ImplTest:getDataRef() end

---@return Data
function ImplTest:getBoxedData() end

---@return Data
function ImplTest:getDataMut() end

---@param val integer|nil
function ImplTest:setOpt(val) end

---@return integer|nil
function ImplTest:getOptU32() end

---@return Data|nil
function ImplTest:getOptData() end

---@param val integer|nil
function ImplTest:setOptRef(val) end

---@param val integer|nil
function ImplTest:setOptMut(val) end

---@return integer
function ImplTest.RetResVal() end

---@return integer
function ImplTest.RetResErr() end

---@return integer|nil
function ImplTest.RetResOptVal() end

---@param c CopyableData
function ImplTest:setCopyable(c) end

---@param c CopyableData
function ImplTest:setCopyableByRef(c) end

---@param c CopyableData
function ImplTest:setCopyableByMutRef(c) end

---@return CopyableData
function ImplTest:getCopyable() end

---@param result CopyableData [out]
function ImplTest:getCopyableViaOutParam(result) end

---@return CopyableData
function ImplTest:getBoxedCopyable() end

---@return CopyableData|nil
function ImplTest:getOptCopyable() end

---@param val string|nil
function ImplTest:setOptStr(val) end

---@param val string|nil
function ImplTest:setOptString(val) end

---@param val string|nil
function ImplTest:setOptStringRef(val) end

---@return string|nil
function ImplTest:getOptStr() end

---@return string|nil
function ImplTest:getOptString() end

---@return string|nil
function ImplTest:getOptStringRef() end

