---@meta

---@class TimeStamp
TimeStamp = {}

---@return TimeStamp
function TimeStamp.Now() end

---@param seconds number
---@return TimeStamp
function TimeStamp.GetFuture(seconds) end

---@param endTime TimeStamp
---@return number
function TimeStamp:getDifference(endTime) end

---Number of seconds elapsed since this timestamp.
---@return number
function TimeStamp:getElapsed() end

---@return number
function TimeStamp:getElapsedMs() end

---@param seconds number
---@return TimeStamp
function TimeStamp:getRelative(seconds) end

---@return number
function TimeStamp:toDouble() end

---@return integer
function TimeStamp:toSeconds() end

