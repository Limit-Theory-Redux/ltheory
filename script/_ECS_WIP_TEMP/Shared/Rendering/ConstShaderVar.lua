local UniformFuncs = require("_ECS_WIP_TEMP.Shared.Rendering.UniformFuncs")
---@class ConstShaderVar
---@field uniformName string
---@field uniformInt integer|nil
---@field uniformType UniformType
---@field uniformValues ffi.ct*[]|nil
---@field callbackFn function|nil

---@class ConstShaderVar
---@overload fun(self: ConstShaderVar, uniformName: string, uniformType: UniformType): ConstShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType): ConstShaderVar class external
local ConstShaderVar = Class(function(self, uniformName, uniformType)
    self.uniformName = uniformName
    self.uniformInt = nil
    self.uniformType = uniformType
    self.uniformValues = nil
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
        Log.Error("Shader " .. shader .. ": Does not have uniform: " .. uniformName)
        return false
    end
end

---@return boolean
function ConstShaderVar:hasUniformValues()
    if self.uniformValues == nil then return false end
    return true
end

---@param values ffi.ct*[]
function ConstShaderVar:setUniformValues(values)
    self.uniformValues = values
end

---@param entity Entity
function ConstShaderVar:setUniformValuesFromEntity(entity)
    self.uniformValues = self.callbackFn(entity)
end

function ConstShaderVar:resetUniformValues()
    self.uniformValues = nil
end

function ConstShaderVar:setShaderVar(shader)
    if self.uniformInt == nil then
        Log.Warn("ConstShaderVar " .. self.uniformName .. " uniformInt not set before setShaderVar")
        self:setUniformInt(shaderState:shader())
    end
    if self.uniformValues == nil then
        Log.Error("ConstShaderVar " .. self.uniformName .. " uniformValues not set before setShaderVar")
    end
    UniformFuncs[self.uniformType].func(shader, self.uniformInt, self.uniformValues)
end

return ConstShaderVar
