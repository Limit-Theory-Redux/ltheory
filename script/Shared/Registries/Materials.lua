---@class Materials
local Materials = {}
Materials.__index = Materials

local registry = {}

-- Register a material
function Materials:new(name, material)
    if registry[name] then
        Log.Warn("Material already registered: " .. name)
        return registry[name]
    end

    material.name = name
    registry[name] = material

    -- Cloning via e.g. Materials.PlanetSurface()
    local template = material
    local mt = {
        __index = template,
        __call = function(_, ...) return template:clone() end
    }

    local proxy = setmetatable({}, mt)
    registry[name] = proxy

    return proxy
end

-- Global access
setmetatable(Materials, {
    __index = function(_, key)
        return registry[key]
    end,
    __newindex = function()
        error("Cannot assign to Materials registry. Use Materials:new()")
    end
})

return Materials
