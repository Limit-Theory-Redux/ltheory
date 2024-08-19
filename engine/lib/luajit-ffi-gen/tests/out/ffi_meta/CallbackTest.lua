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
function CallbackTest:nthManaged(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthManagedRef(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthManagedMut(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthManagedValOpt(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthManagedRefOpt(index, callback) end

---@param index integer
---@param callback function
function CallbackTest:nthManagedMutOpt(index, callback) end

---@param callback function
function CallbackTest:insertManaged(callback) end

---@param callback function
function CallbackTest:readPrimitiveArray(callback) end

---@param callback function
function CallbackTest:lockPrimitiveArray(callback) end

---@param callback function
function CallbackTest:readManagedArray(callback) end

---@param callback function
function CallbackTest:lockManagedArray(callback) end

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

