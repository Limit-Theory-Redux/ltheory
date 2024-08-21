-- Archetypes

---@enum Type
Enums.Type = {
    EntityInfo = 1,
    ComponentInfo = 2,
    -- ...
}

local typeNames = {}
for k, v in pairs(Enums.Type) do
    typeNames[v] = k
end

---@param type Type
---@return string
function Enums.Type:getName(type)
    return typeNames[type]
end
