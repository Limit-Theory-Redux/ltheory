---@meta

---@class EventData
EventData = {}

---@return number
function EventData:deltaTime() end

---@return FrameStage
function EventData:frameStage() end

---@return integer
function EventData:tunnelId() end

---@return EventPayload?
function EventData:payload() end

