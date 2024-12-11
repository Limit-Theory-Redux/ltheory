local UniformFuncs = require("_ECS_WIP_TEMP.Shared.Rendering.UniformFuncs")

---@class ConstShaderVar
---@field uniformName string
---@field uniformInt integer|nil
---@field uniformType UniformType
---@field requiresEntity boolean
---@field callbackFn function|nil

---@class ConstShaderVar
---@overload fun(self: ConstShaderVar, uniformName: string, uniformType: UniformType, requiresEntity: boolean): ConstShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType, requiresEntity: boolean): ConstShaderVar class external
local ConstShaderVar = Class(function(self, uniformName, uniformType, requiresEntity)
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

---@param shader Shader
---@param entity Entity|nil
function ConstShaderVar:setShaderVar(shader, entity)
    if not self.uniformInt then
        Log.Warn("ConstShaderVar " .. self.uniformName .. " uniformInt not set before setShaderVar")
        self:setUniformInt(shaderState:shader())
    end

    -- ignore var if uniform is nil
    if not self.uniformInt then
        return
    end

    if self.requiresEntity and not entity then
        Log.Error("ConstShaderVar Requires Entity, No Entity given.")
    end

    if entity then
        UniformFuncs[self.uniformType](shader, self.uniformInt, self.callbackFn(entity))
    else
        UniformFuncs[self.uniformType](shader, self.uniformInt, self.callbackFn())
    end
end

return ConstShaderVar
