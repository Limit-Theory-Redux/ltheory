---@meta

---@class EngineImpl
EngineImpl = {}

---@return WindowImpl
function EngineImpl:window() end

---@return InputImpl
function EngineImpl:input() end

---@return EventBusImpl
function EngineImpl:eventBus() end

---@return HmGui
function EngineImpl:hmGui() end

function EngineImpl.Abort() end

---@return integer
function EngineImpl.GetBits() end

-- Return time passed since engine start.
---@return number
function EngineImpl:getTime() end

---@return string
function EngineImpl.GetVersion() end

function EngineImpl:exit() end

function EngineImpl.Terminate() end

function EngineImpl.Update() end

