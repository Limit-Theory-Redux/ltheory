-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Collection of named payloads.
-- Elements ordered by insertion order.
---@class PayloadTable
PayloadTable = {}

---@return PayloadTable
function PayloadTable.Create() end

-- Returns number of elements in the table.
---@return integer
function PayloadTable:len() end

-- Checks if table is empty.
---@return boolean
function PayloadTable:isEmpty() end

-- Checks if table contains an element with name 'name'.
---@param name string
---@return boolean
function PayloadTable:contains(name) end

-- Returns the name of the element at position 'index'.
-- Returns `None`/`nil` if index is bigger than the number of elements in the table.
---@param index integer
---@return string?
function PayloadTable:getName(index) end

-- Returns payload at position 'index'.
-- Returns `None`/`nil` if index is bigger than the number of elements in the table.
---@param index integer
---@return Payload?
function PayloadTable:getPayload(index) end

-- Returns payload by name 'name'.
-- Returns `None`/`nil` if index is bigger than the number of elements in the table.
---@param name string
---@return Payload?
function PayloadTable:getPayloadByName(name) end

-- Add new element to the table.
---@param name string
---@param value Payload
function PayloadTable:add(name, value) end

