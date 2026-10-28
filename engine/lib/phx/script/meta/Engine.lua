-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Engine
Engine = {}

---@return Window
function Engine:window() end

---@return Input
function Engine:input() end

---@return EventBus
function Engine:eventBus() end

---@return TaskQueue
function Engine:taskQueue() end

---@return HmGui
function Engine:hmGui() end

---@return Renderer
function Engine:renderer() end

function Engine.Abort() end

---@return integer
function Engine.GetBits() end

-- Return time passed since engine start.
---@return number
function Engine:getTime() end

---@return string
function Engine.GetVersion() end

-- Ends the run with the success exit code (0).
function Engine:exit() end

-- Ends the run with `code` as the process exit code. No frame Lua runs
-- after the request; `AppClose` still does. The first non-zero code wins.
---@param code integer
function Engine:exitWithCode(code) end

function Engine.Terminate() end

function Engine.Update() end

