-- Constructed Types
local typeCounter = 0
local typeNames = {}

---@class Type
Enums.Type = {}

---@param typeInt integer
---@return string
function Enums.Type:getName(typeInt)
    return typeNames[typeInt]
end

---@param name string
---@return integer TypeInt
function Enums.Type:createType(name)
    typeCounter = typeCounter + 1
    typeNames[typeCounter] = name
    Enums.Type[name] = typeCounter
    return typeCounter
end
