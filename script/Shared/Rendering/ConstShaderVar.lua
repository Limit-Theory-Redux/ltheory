local UniformFuncs = require("Shared.Rendering.UniformFuncs")

---@class ConstShaderVar
---@field uniformName string
---@field uniformInt integer|nil
---@field uniformType UniformType
---@field requiresEntity boolean
---@field callbackFn function|nil

---@class ConstShaderVar
---@overload fun(self: ConstShaderVar, uniformName: string, uniformType: UniformType, requiresEntity: boolean): ConstShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType, requiresEntity: boolean): ConstShaderVar class external
local ConstShaderVar = Class("ConstShaderVar", function(self, uniformName, uniformType, requiresEntity)
    self.uniformName = uniformName
    self.uniformInt = nil
    self.uniformType = uniformType
    self.requiresEntity = requiresEntity
    self.callbackFn = nil
end)

---@param callbackFn function
function ConstShaderVar:setCallbackFn(callbackFn)
    self.callbackFn = callbackFn
end

---@param shader Shader
function ConstShaderVar:setUniformInt(shader)
    if shader:hasVariable(self.uniformName) then
        self.uniformInt = shader:getVariable(self.uniformName)
        return true
    else
        Log.Warn("Shader " .. tostring(shader) .. ": Does not have uniform: " .. self.uniformName)
        return false
    end
end

function ConstShaderVar:setShaderVar(shader, entity)
    if not self.uniformInt then return end
    local values = { self.callbackFn() }
    local func = UniformFuncs[self.uniformType]
    if func then
        func(shader, self.uniformInt, unpack(values))
    end
end

return ConstShaderVar
