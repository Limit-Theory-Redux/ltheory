---@meta

---@class ArraysTest
ArraysTest = {}

---@param data integer
---@param data_size integer
function ArraysTest:setPrimitiveSlice(data, data_size) end

---@param out integer
---@param out_size integer
function ArraysTest:getPrimitiveSlice(out, out_size) end

---@param data ManagedData
---@param data_size integer
function ArraysTest:setManagedSlice(data, data_size) end

---@param out ManagedData
---@param out_size integer
function ArraysTest:getManagedSlice(out, out_size) end

---@param data integer
---@param data_size integer
function ArraysTest:movePrimitiveArray(data, data_size) end

---@param data integer
---@param data_size integer
function ArraysTest:setPrimitiveArray(data, data_size) end

---@param out integer
---@param out_size integer
function ArraysTest:getPrimitiveArray(out, out_size) end

---@param data ManagedData
---@param data_size integer
function ArraysTest:moveManagedArray(data, data_size) end

---@param data ManagedData
---@param data_size integer
function ArraysTest:setManagedArray(data, data_size) end

---@param out ManagedData
---@param out_size integer
function ArraysTest:getManagedArray(out, out_size) end

