---@meta

---@class Directory
Directory = {}

---@param path string
---@return Directory|nil
function Directory.Open(path) end

---@return string|nil
function Directory:getNext() end

---@param cwd string
---@return boolean
function Directory.Change(cwd) end

---@param path string
---@return boolean
function Directory.Create(path) end

---@return string|nil
function Directory.GetCurrent() end

---@param org string
---@param app string
---@return string|nil
function Directory.GetPrefPath(org, app) end

---@param path string
---@return boolean
function Directory.Remove(path) end

