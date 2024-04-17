---@meta

TimeStamp = TimeStamp

---@return TimeStamp
function TimeStamp.Now() end

---@param seconds number
---@return TimeStamp
function TimeStamp.GetFuture(seconds) end

---@param end TimeStamp
---@return number
function TimeStamp:getDifference(self, end) end

---Number of seconds elapsed since this timestamp.
---@return number
function TimeStamp:getElapsed(self) end

---@return number
function TimeStamp:getElapsedMs(self) end

---@param seconds number
---@return TimeStamp
function TimeStamp:getRelative(self, seconds) end

---@return number
function TimeStamp:toDouble(self) end

---@return integer
function TimeStamp:toSeconds(self) end

