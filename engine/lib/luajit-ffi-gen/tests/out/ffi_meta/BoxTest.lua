---@meta

---@class BoxTest
BoxTest = {}

---@param val integer
function BoxTest:setPrimitive(val) end

---@return integer
function BoxTest:getPrimitive() end

---@param val ManagedData
function BoxTest:setManaged(val) end

---@return ManagedData
function BoxTest:getManaged() end

---@param val CopyableData
function BoxTest:setCopyable(val) end

---@return CopyableData
function BoxTest:getCopyable() end

