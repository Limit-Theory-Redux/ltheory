local NameComponent = require("Modules.Core.Components.NameComponent")

---An entity's name is its kind (`"MoonEntity"`); `Core.Type` only holds the variety within it.
---@class EntityKind
local EntityKind = {}

---True when the entity is alive and its name is exactly `kind`. A name that only contains the
---kind (`"AsteroidBeltEntity"` against `"AsteroidEntity"`) does not match.
---@param entity Entity|nil
---@param kind string
---@return boolean
function EntityKind.Is(entity, kind)
    if not entity then return false end
    local name = entity:get(NameComponent)
    return name ~= nil and name:getName() == kind
end

return EntityKind
