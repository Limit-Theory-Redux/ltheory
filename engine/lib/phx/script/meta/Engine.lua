---@meta

Engine = Engine

---@return Window
function Engine.window(self) end

---@return Input
function Engine.input(self) end

---@return HmGui
function Engine.hmGui(self) end

function Engine:Abort() end

---@return integer
function Engine:GetBits() end

---@return number
function Engine.getTime(self) end

---@return string
function Engine:GetVersion() end

function Engine.exit(self) end

function Engine:Terminate() end

function Engine:Update() end

