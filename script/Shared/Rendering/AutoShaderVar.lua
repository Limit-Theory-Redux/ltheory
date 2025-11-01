local UniformFuncs = require("Shared.Rendering.UniformFuncs")
---@class AutoShaderVar
---@field uniformName string
---@field uniformInt integer
---@field uniformType UniformType
---@field callbackFn function

---@class AutoShaderVar
---@overload fun(self: AutoShaderVar, uniformName: string, uniformType: UniformType, callbackFn: function): AutoShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType,  callbackFn: function): AutoShaderVar class external
local AutoShaderVar = Class("AutoShaderVar", function(self, uniformName, uniformType, callbackFn)
    self.uniformName = uniformName
    self.uniformInt = nil
    self.uniformType = uniformType
    self.callbackFn = callbackFn
end)

function AutoShaderVar:setShaderVar(eye, shader, entity)
    if not self.uniformInt then
        return -- Already warned in reloadShader()
    end

    local values = { self.callbackFn(eye, entity) }
    local func = UniformFuncs[self.uniformType]
    if func then
        func(shader, self.uniformInt, unpack(values))
    end
end

return AutoShaderVar
