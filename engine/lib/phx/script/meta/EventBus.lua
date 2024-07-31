---@meta

---@class EventBus
EventBus = {}

---@return number
function EventBus:getTimeScale() end

---@param scaleFactor number
function EventBus:setTimeScale(scaleFactor) end

---@param eventName string
---@param priority integer
---@param frameStage FrameStage
---@param withFrameStageMessage boolean
function EventBus:register(eventName, priority, frameStage, withFrameStageMessage) end

---@param eventName string
function EventBus:unregister(eventName) end

---@param eventName string
---@param entityId integer
---@return integer
---@overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
function EventBus:subscribe(eventName, entityId) end

---@param tunnelId integer
function EventBus:unsubscribe(tunnelId) end

---@param eventName string
---@param entityId integer
---@param payload EventPayload
---@overload fun(self: table, eventName: string, ctxTable: table|nil, payload: EventPayload|nil)
function EventBus:send(eventName, entityId, payload) end

---@return EventData
function EventBus:getNextEvent() end

function EventBus:printFrameStageMap() end

