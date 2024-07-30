---@meta

---@class EventData
EventData = {}

---@return number
function EventData:getDeltaTime() end

---@return FrameStage
function EventData:getFrameStage() end

---@return integer
function EventData:getTunnelId() end

---@return EventPayload|nil
function EventData:getPayload() end

