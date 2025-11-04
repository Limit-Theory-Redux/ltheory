local UniformFuncs = require("Shared.Rendering.UniformFuncs")

---@class AutoShaderVar
---@field uniformName string
---@field uniformInt integer
---@field uniformType UniformType
---@field callbackFn function|nil
---@field perInstance boolean|nil
---@field requiresEntity boolean|nil

---@class AutoShaderVar
---@overload fun(self: AutoShaderVar, uniformName: string, uniformType: UniformType, callbackFn: function|nil, perInstance: boolean, requiresEntity: boolean|nil): AutoShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType,  callbackFn: function|nil, perInstance: boolean, requiresEntity: boolean|nil): AutoShaderVar class external
local AutoShaderVar = Class("AutoShaderVar", function(self, uniformName, uniformType, callbackFn, perInstance, requiresEntity)
    self.uniformName = uniformName
    self.uniformInt = nil
    self.uniformType = uniformType
    self.callbackFn = callbackFn
    self.perInstance = perInstance or false
    self.requiresEntity = requiresEntity or false
end)

---@param callbackFn function
function AutoShaderVar:setCallbackFn(callbackFn)
    self.callbackFn = callbackFn
end

---@param shader Shader
function AutoShaderVar:setUniformInt(shader)
    if shader:hasVariable(self.uniformName) then
        self.uniformInt = shader:getVariable(self.uniformName)
        return true
    else
        Log.Warn("Shader " .. tostring(shader) .. ": Does not have uniform: " .. self.uniformName)
        self.uniformInt = nil
        return false
    end
end

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
