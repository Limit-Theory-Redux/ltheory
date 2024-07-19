---@meta

---@class EventBus
EventBus = {}

---@return boolean
function EventBus:isReady() end

---@param eventName string
---@param priority EventPriority
---@param updatePass UpdatePass
---@param withUpdatePassMessage boolean
function EventBus:register(eventName, priority, updatePass, withUpdatePassMessage) end

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
---@param updatePass UpdatePass
function EventBus:send(eventName, entityId, updatePass) end

---@return EventData
function EventBus:getNextEvent() end

function EventBus:printUpdatePassMap() end

