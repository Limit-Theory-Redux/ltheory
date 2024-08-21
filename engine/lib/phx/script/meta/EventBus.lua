---@meta

---@class EventBus
EventBus = {}

---@return number
function EventBus:getTimeScale() end

---@param scaleFactor number
function EventBus:setTimeScale(scaleFactor) end

---@param eventId integer
---@return boolean
function EventBus:hasRustPayload(eventId) end

---@param eventId integer
---@param eventName string
---@param frameStage FrameStage
---@param rustPayload boolean
function EventBus:register(eventId, eventName, frameStage, rustPayload) end

---@param eventId integer
function EventBus:unregister(eventId) end

---@param eventId integer
---@param entityId integer?
---@return integer
---@overload fun(self: table, eventType: integer, ctxTable: table|nil, callbackFunc: function): integer
function EventBus:subscribe(eventId, entityId) end

---@param tunnelId integer
function EventBus:unsubscribe(tunnelId) end

---@param eventId integer
---@param entityId integer?
---@param payload Payload?
---@overload fun(self: table, eventType: integer, ctxTable: table|nil, payload: Payload|nil)
function EventBus:send(eventId, entityId, payload) end

function EventBus:startEventIteration() end

-- Iterates over events of the frame.
-- Returns `None`/`nil` when there are no more events.
---@return EventData?
function EventBus:nextEvent() end

function EventBus:printFrameStageMap() end

