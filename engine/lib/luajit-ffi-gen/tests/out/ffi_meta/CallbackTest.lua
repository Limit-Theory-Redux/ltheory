-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class CallbackTest
CallbackTest = {}

---@param index integer
---@param callback fun(arg1: number): nil
function CallbackTest:nthPrimitive(index, callback) end

---@param index integer
---@param callback fun(arg1: number): nil
function CallbackTest:nthPrimitiveRef(index, callback) end

---@param index integer
---@param callback fun(arg1: number): nil
function CallbackTest:nthPrimitiveMut(index, callback) end

---@param index integer
---@param callback fun(arg1: number?): nil
function CallbackTest:nthPrimitiveValOpt(index, callback) end

---@param index integer
---@param callback fun(arg1: number?): nil
function CallbackTest:nthPrimitiveRefOpt(index, callback) end

---@param index integer
---@param callback fun(arg1: number?): nil
function CallbackTest:nthPrimitiveMutOpt(index, callback) end

---@param callback fun(): number
function CallbackTest:insertPrimitive(callback) end

---@param index integer
---@param callback fun(arg1: ManagedData): nil
function CallbackTest:nthManaged(index, callback) end

---@param index integer
---@param callback fun(arg1: ManagedData): nil
function CallbackTest:nthManagedRef(index, callback) end

---@param index integer
---@param callback fun(arg1: ManagedData): nil
function CallbackTest:nthManagedMut(index, callback) end

---@param index integer
---@param callback fun(arg1: ManagedData?): nil
function CallbackTest:nthManagedValOpt(index, callback) end

---@param index integer
---@param callback fun(arg1: ManagedData?): nil
function CallbackTest:nthManagedRefOpt(index, callback) end

---@param index integer
---@param callback fun(arg1: ManagedData?): nil
function CallbackTest:nthManagedMutOpt(index, callback) end

---@param callback fun(): ManagedData
function CallbackTest:insertManaged(callback) end

---@param callback fun(arg1: number[]): nil
function CallbackTest:readPrimitiveArray(callback) end

---@param callback fun(arg1: number[]): nil
function CallbackTest:lockPrimitiveArray(callback) end

---@param callback fun(arg1: ManagedData[]): nil
function CallbackTest:readManagedArray(callback) end

---@param callback fun(arg1: ManagedData[]): nil
function CallbackTest:lockManagedArray(callback) end

---@param s string
---@param callback fun(arg1: string): string
---@return string
function CallbackTest.TransformString(s, callback) end

---@param s string
---@param callback fun(arg1: string): string
---@return string
function CallbackTest.TransformStr(s, callback) end

---@param index integer
---@param callback fun(arg1: number, arg2: ManagedData?): number
function CallbackTest:getMultipleAndReplace(index, callback) end

---@param input integer
---@param callback fun(arg1: integer): integer
---@return integer
function CallbackTest.Passthrough(input, callback) end

