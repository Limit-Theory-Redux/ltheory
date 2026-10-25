--- Registry of the `MaterialType`s defined in `Shared/Definitions/MaterialDefs.lua`.
---
---     local Materials = require("Shared.Registries.Materials")
---     local mat = Materials.PlanetSurface:instance()
---
--- A `MaterialType` is the shared part of a material (shader, fixed-function
--- state, parameter defaults, default textures, per-draw callback); an
--- `instance()` owns its own parameters and bind group.
---@class Materials
local Materials = {}
Materials.__index = Materials

---@type table<string, MaterialType>
local registry = {}

--- Register a material type (called by `MaterialType`).
---@param name string
---@param matType MaterialType
---@return MaterialType
function Materials:new(name, matType)
    if registry[name] then
        Log.Warn("Material type already registered: " .. name)
        return registry[name]
    end
    registry[name] = matType
    return matType
end

--- Call `fn(matType)` for every registered material type.
---@param fn fun(matType: MaterialType)
function Materials.each(fn)
    for _, matType in pairs(registry) do fn(matType) end
end

-- Global access
setmetatable(Materials, {
    __index = function(_, key)
        return registry[key]
    end,
    __newindex = function()
        error("Cannot assign to Materials registry. Use MaterialType { name = ... }")
    end
})

return Materials
