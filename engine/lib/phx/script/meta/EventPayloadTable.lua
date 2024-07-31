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

---@param name string
---@return EventPayload|nil
function EventPayloadTable:get(name) end

---@param name string
---@param value EventPayload
function EventPayloadTable:add(name, value) end

---@param name string
function EventPayloadTable:remove(name) end

