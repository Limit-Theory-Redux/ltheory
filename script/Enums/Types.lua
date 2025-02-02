-- Constructed Types
local typeCounter = 0
local typeNames = {}

local typeMetatable = {
    __tostring = function(ty)
        return Enums.Type:getName(ty)
    end,
}

---@class Type
---@field id integer

Enums.Type = {}

---@param ty Type
---@return string
function Enums.Type:getName(ty)
    return typeNames[ty.id]
end

---@param name string
---@return Type
function Enums.Type:createType(name)
    if Enums.Type[name] then
        return Enums.Type[name]
    end
    
    typeCounter = typeCounter + 1
    typeNames[typeCounter] = name
    
    local ty = setmetatable({
        id = typeCounter
    }, typeMetatable)
    
    Enums.Type[name] = ty

    return ty
end
