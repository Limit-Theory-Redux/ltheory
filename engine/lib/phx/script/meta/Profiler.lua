-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Profiler
Profiler = {}

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

