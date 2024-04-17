---@meta

---@class Sound
Sound = {}

---@param path string
---@param is_looping boolean
---@return Sound
function Sound.Load(path, is_looping) end

---@return number
function Sound:getDuration() end

---@return string
function Sound:getPath() end

