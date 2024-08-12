---@meta

---@class EventPayloadTable
EventPayloadTable = {}

---@return EventPayloadTable
function EventPayloadTable.Create() end

---@return integer
function EventPayloadTable:len() end

---@return boolean
function EventPayloadTable:isEmpty() end

---@param name string
---@return boolean
function EventPayloadTable:contains(name) end

---@param index integer
---@return string|nil
function EventPayloadTable:getName(index) end

---@param index integer
---@return EventPayload|nil
function EventPayloadTable:getPayload(index) end

---@param name string
---@param value EventPayload
function EventPayloadTable:add(name, value) end

