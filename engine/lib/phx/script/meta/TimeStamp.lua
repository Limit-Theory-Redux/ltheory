---@meta

---@class TimeStamp
TimeStamp = {}

---Get current timestamp.
---@return TimeStamp
function TimeStamp.Now() end

---Get timestamp `seconds` in a future from now.
---@param seconds number
---@return TimeStamp
function TimeStamp.GetFuture(seconds) end

---Get difference between 2 timestamps in double seconds.
---@param endTime TimeStamp
---@return number
function TimeStamp:getDifference(endTime) end

---Number of seconds elapsed since this timestamp.
---@return number
function TimeStamp:getElapsed() end

---Number of milliseconds elapsed since this timestamp.
---@return number
function TimeStamp:getElapsedMs() end

---Get timestamp `seconds` in a future relative to current one.
---@param seconds number
---@return TimeStamp
function TimeStamp:getRelative(seconds) end

---Get duration since Unix epoch in double seconds.
---@return number
function TimeStamp:toDouble() end

---Get duration since Unix epoch in unsigned seconds.
---@return integer
function TimeStamp:toSeconds() end

