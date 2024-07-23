---@meta

---@class EventBusImpl
EventBusImpl = {}

---@return number
function EventBusImpl:getTimeScale() end

---@param scaleFactor number
function EventBusImpl:setTimeScale(scaleFactor) end

---@param eventName string
---@param priority integer
---@param frameStage FrameStage
---@param withFrameStageMessage boolean
function EventBusImpl:register(eventName, priority, frameStage, withFrameStageMessage) end

---@param eventName string
function EventBusImpl:unregister(eventName) end

---@param eventName string
---@param entityId integer
---@return integer
---@overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
function EventBusImpl:subscribe(eventName, entityId) end

---@param tunnelId integer
function EventBusImpl:unsubscribe(tunnelId) end

---@param eventName string
---@param entityId integer
---@overload fun(self: table, eventName: string, ctxTable: table|nil)
function EventBusImpl:send(eventName, entityId) end

---@return EventData
function EventBusImpl:getNextEvent() end

function EventBusImpl:printFrameStageMap() end

