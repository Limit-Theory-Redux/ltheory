---@meta

---@class PayloadTable
PayloadTable = {}

---@return PayloadTable
function PayloadTable.Create() end

---@return integer
function PayloadTable:len() end

---@return boolean
function PayloadTable:isEmpty() end

---@param name string
---@return boolean
function PayloadTable:contains(name) end

---@param index integer
---@return string?
function PayloadTable:getName(index) end

---@param index integer
---@return Payload?
function PayloadTable:getPayload(index) end

---@param name string
---@param value Payload
function PayloadTable:add(name, value) end

