---@class AutoShaderVar
---@field uniformName string
---@field uniformInt integer
---@field renderFn function

---@param self AutoShaderVar
---@param uniformName string
---@param renderFn function
---@class AutoShaderVar
local AutoShaderVar = Class(function(self, uniformName, renderFn)
    ---@cast self AutoShaderVar
    self.uniformName = uniformName
--    self.params = params
    self.uniformInt = -1 -- -1 = unset
    self.renderFn = renderFn
end)

---@param uniformInt integer
function AutoShaderVar:setUniformInt(uniformInt)
    self.uniformInt = uniformInt
end

--!temp NOT FINAL IMPLEMENTATION, FOR TESTING
---@param shaderState ShaderState
---@param args table
function AutoShaderVar:render(shaderState, args) 
    self.renderFn(shaderState, self.uniformInt, args)
end

return AutoShaderVar