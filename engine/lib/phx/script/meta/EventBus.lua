---@meta

---@class EventBus
EventBus = {}

---@return number
function EventBus:getTimeScale() end

---@param scaleFactor number
function EventBus:setTimeScale(scaleFactor) end

---@param eventId integer
---@param eventName string
---@param frameStage FrameStage
---@param withFrameStageMessage boolean
function EventBus:register(eventId, eventName, frameStage, withFrameStageMessage) end

---@param eventId integer
function EventBus:unregister(eventId) end

---@param eventId integer
---@param entityId integer|nil
---@return integer
---@overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
function EventBus:subscribe(eventId, entityId) end

---@param tunnelId integer
function EventBus:unsubscribe(tunnelId) end

---@param eventId integer
---@param entityId integer
---@param payload EventPayload|nil
---@overload fun(self: table, eventName: string, ctxTable: table|nil, payload: EventPayload|nil)
function EventBus:send(eventId, entityId, payload) end

function EventBus:startEventIteration() end

-- Iterates over events of the frame.
-- Returns `None`/`nil` when there are no more events.
---@return EventData|nil
function EventBus:nextEvent() end

function EventBus:printFrameStageMap() end

