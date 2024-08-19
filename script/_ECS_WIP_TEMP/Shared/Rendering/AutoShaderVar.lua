---@class AutoShaderVar
---@field uniformName string
---@field uniformInt integer
---@field callbackFn function

---@param self AutoShaderVar
---@param uniformName string
---@param callbackFn function
---@class AutoShaderVar
---@overload fun(self: AutoShaderVar, uniformName: string, callbackFn: function): AutoShaderVar class internal
---@overload fun(uniformName: string, callbackFn: function): AutoShaderVar class external
local AutoShaderVar = Class(function(self, uniformName, callbackFn)
    self.uniformName = uniformName
    -- self.params = params
    self.uniformInt = -1 -- -1 = unset
    self.callbackFn = callbackFn
end)

---@param uniformInt integer
function AutoShaderVar:setUniformInt(uniformInt)
    self.uniformInt = uniformInt
end

--!temp NOT FINAL IMPLEMENTATION, FOR TESTING
---@param shaderState ShaderState
function AutoShaderVar:setShaderVar(shaderState, ...)
    self.callbackFn(shaderState, self.uniformInt, ...)
end

return AutoShaderVar
