---@meta

Timer = Timer

---@return Timer
function Timer:Create() end

---@return number
function Timer.getAndReset(self) end

---@return number
function Timer.getElapsed(self) end

function Timer.reset(self) end

