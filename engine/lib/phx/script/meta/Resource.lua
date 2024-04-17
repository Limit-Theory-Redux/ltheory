---@meta

Resource = Resource

---@param ty ResourceType
---@param name string
---@return boolean
function Resource.Exists(ty, name) end

---@param ty ResourceType
---@param name string
---@return string
function Resource.GetPath(ty, name) end

---@param ty ResourceType
---@param name string
---@return Bytes
function Resource.LoadBytes(ty, name) end

---@param ty ResourceType
---@param name string
---@return string
function Resource.LoadString(ty, name) end

