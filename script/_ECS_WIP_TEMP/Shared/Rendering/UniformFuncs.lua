local UniformFuncs = {}
UniformFuncs.__index = UniformFuncs

---@param uniformType UniformType
---@param func function
function UniformFuncs:new(uniformType, func)
    self[uniformType] = func
    return self[uniformType]
end

return UniformFuncs
