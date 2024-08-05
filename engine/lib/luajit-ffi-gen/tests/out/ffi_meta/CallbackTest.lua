---@meta

---@class CallbackTest
CallbackTest = {}

---@param index integer
---@param callback function
function CallbackTest:nthPrimitive(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthPrimitiveRef(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthPrimitiveMut(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthPrimitiveValOpt(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthPrimitiveRefOpt(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthPrimitiveMutOpt(index, callback) end

---@param callback function
function CallbackTest:insertPrimitive(callback) end

---@param index integer
---@param callback function
function CallbackTest:nthNoncopyable(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthNoncopyableRef(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthNoncopyableMut(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthNoncopyableValOpt(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthNoncopyableRefOpt(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthNoncopyableMutOpt(index, callback) end

---@param callback function
function CallbackTest:insertNoncopyable(callback) end

---@param callback function
function CallbackTest:readPrimitiveArray(callback) end

---@param callback function
function CallbackTest:lockPrimitiveArray(callback) end

---@param callback function
function CallbackTest:readNoncopyableArray(callback) end

---@param callback function
function CallbackTest:lockNoncopyableArray(callback) end

---@param s string
---@param callback function
---@return string
function CallbackTest.TransformString(s, callback) end

---@param s string
---@param callback function
---@return string
function CallbackTest.TransformStr(s, callback) end

---@param index integer
---@param callback function
function CallbackTest:getMultipleAndReplace(index, callback) end

---@param input integer
---@param callback function
---@return integer
function CallbackTest.Passthrough(input, callback) end

