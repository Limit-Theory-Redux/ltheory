---@meta

---@class InstantTime
InstantTime = {}

---@return InstantTime
function InstantTime.Now() end

---Return time in double milliseconds passed since earlier time.
---@param earlier InstantTime
---@return number
function InstantTime:durationSince(earlier) end

