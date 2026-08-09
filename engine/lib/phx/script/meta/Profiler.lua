-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Profiler
Profiler = {}

-- True if profiling is currently enabled (lock-free read).
---@return boolean
function Profiler.IsEnabled() end

-- Request a profiling toggle. The actual enable/disable happens on the
-- main thread at the next safe point (`Profiler::pending_toggle`).
function Profiler.RequestToggle() end

-- Check-and-clear the toggle request. Called once per frame from the
-- main thread's safe point (Application:onPreRender) - returns true
-- exactly once per dashboard click, mirroring the F10 path.
---@return boolean
function Profiler.PendingToggle() end

-- Enables profiling and initializes the profiler state
function Profiler.Enable() end

-- Disables profiling and processes results
function Profiler.Disable() end

-- Starts a new profiling scope
---@param name string
function Profiler.Begin(name) end

-- Ends the current profiling scope
function Profiler.End() end

---@param name string
---@param value integer
function Profiler.SetValue(name, value) end

-- Records frame timing for each active scope
function Profiler.LoopMarker() end

-- Prints backtrace of active scopes
function Profiler.Backtrace() end

