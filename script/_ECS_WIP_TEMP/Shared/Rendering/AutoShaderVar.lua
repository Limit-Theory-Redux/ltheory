local UniformFuncs = require("_ECS_WIP_TEMP.Shared.Rendering.UniformFuncs")
---@class AutoShaderVar
---@field uniformName string
---@field uniformInt integer
---@field uniformType UniformType
---@field callbackFn function

---@class AutoShaderVar
---@overload fun(self: AutoShaderVar, uniformName: string, uniformType: UniformType, callbackFn: function): AutoShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType,  callbackFn: function): AutoShaderVar class external
local AutoShaderVar = Class(function(self, uniformName, uniformType, callbackFn)
    self.uniformName = uniformName
    self.uniformInt = nil
    self.uniformType = uniformType
    self.callbackFn = callbackFn
end)

---@param shader Shader
---@return boolean
function AutoShaderVar:setUniformInt(shader)
    if shader:hasVariable(self.uniformName) then
        self.uniformInt = shader:getVariable(self.uniformName)
        return true
    else
        Log.Error("Shader " .. tostring(shader) .. ": Does not have uniform: " .. self.uniformName)
        return false
    end
end

---@param eye Position Camera Position
---@param shader Shader
---@param entity Entity
function AutoShaderVar:setShaderVar(eye, shader, entity)
    if not self.uniformInt then
        Log.Warn("Uniform " .. self.uniformName .. " int not set before updateShaderVar")
        self:setUniformInt(shader)
    end
    UniformFuncs[self.uniformType](shader, self.uniformInt, self.callbackFn(eye, entity))
end

return AutoShaderVar
