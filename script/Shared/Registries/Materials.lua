local Materials = {}
Materials.__index = Materials

---@param name string
---@param material Material
function Materials:new(name, material)
    self[name] = material
    -- When Materials.name(), Clone Material
    local mt = getmetatable(self[name])
    mt.__call = function(matDef)
        return matDef:clone()
    end
    setmetatable(self[name], mt)

    return self[name]
end

return Materials
