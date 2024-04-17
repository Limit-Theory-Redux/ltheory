---@meta

---@class Directory
Directory = {}

---@param path string
---@return Directory
function Directory.Open(path) end

---@return string
function Directory:getNext(self) end

---@param cwd string
---@return boolean
function Directory.Change(cwd) end

---@param path string
---@return boolean
function Directory.Create(path) end

---@return string
function Directory.GetCurrent() end

---@param org string
---@param app string
---@return string
function Directory.GetPrefPath(org, app) end

---@param path string
---@return boolean
function Directory.Remove(path) end

