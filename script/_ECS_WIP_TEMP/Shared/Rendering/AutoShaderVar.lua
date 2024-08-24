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

---@param renderState RenderState
---@param shaderState ShaderState
---@param entity Entity
function AutoShaderVar:updateShaderVar(renderState, shaderState, entity)
    self.callbackFn(renderState, shaderState, self.uniformInt, entity)
end

return AutoShaderVar
