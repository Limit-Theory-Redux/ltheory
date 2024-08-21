-- Constructed Types
local typeCounter = 0

---@class Type
Enums.Type = {}

local typeNames = {}
for k, v in pairs(Enums.Type) do
    typeNames[v] = k
end

---@param typeInt integer
---@return string
function Enums.Type:getName(typeInt)
    return typeNames[typeInt]
end

---@param name string
---@return integer TypeInt
function Enums.Type:createType(name)
    typeCounter = typeCounter + 1
    Enums.Type[name] = typeCounter
    return typeCounter
end
