---@meta

---@class EventBus
EventBus = {}

---@param eventName string
---@param priority integer
---@param updatePass UpdatePass
function EventBus:register(eventName, priority, updatePass) end

---@param eventName string
function EventBus:unregister(eventName) end

---@param eventName string
---@param entityId integer
---@return integer
function EventBus:subscribe(eventName, entityId) end

---@param tunnelId integer
function EventBus:unsubscribe(tunnelId) end

---@param eventName string
---@param entityId integer
function EventBus:send(eventName, entityId) end

function EventBus:printUpdatePassMap() end

