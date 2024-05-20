---@meta

---@class Engine
Engine = {}

---@return Window
function Engine:window() end

---@return Input
function Engine:input() end

---@return HmGui
function Engine:hmGui() end

function Engine.Abort() end

---@return integer
function Engine.GetBits() end

---Return time passed since engine start.
---@return number
function Engine:elapsedTime() end

---Return time marker of the current frame.
---@return InstantTime
function Engine:frameTime() end

---Return delta time between current and previous frames in double milliseconds.
---@return number
function Engine:deltaTime() end

---@return string
function Engine.GetVersion() end

function Engine:exit() end

function Engine.Terminate() end

function Engine.Update() end

