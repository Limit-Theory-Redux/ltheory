---@meta

---@class EventBus
EventBus = {}

---@param eventName string
---@param priority integer
---@param updatePass UpdatePass
function EventBus:register(eventName, priority, updatePass) end

---@param eventName string
function EventBus:unregister(eventName) end

---@param updatePass UpdatePass
---@param engine Engine
function EventBus:dispatch(updatePass, engine) end

---@param engine Engine
function EventBus:dispatchAll(engine) end

---@param eventName string
---@return integer
function EventBus:subscribe(eventName) end

---@param tunnelId integer
function EventBus:unsubscribe(tunnelId) end

function EventBus:printUpdatePassMap() end

