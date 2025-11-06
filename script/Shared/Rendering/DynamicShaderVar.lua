local UniformFuncs = require("Shared.Rendering.UniformFuncs")

---@class DynamicShaderVar
---@field uniformName string
---@field uniformInt integer
---@field uniformType UniformType
---@field value any
---@field perInstance boolean|nil
---@field requiresEntity boolean|nil

---@class DynamicShaderVar
---@overload fun(self: DynamicShaderVar, uniformName: string, uniformType: UniformType, value: any, perInstance: boolean, requiresEntity: boolean|nil): DynamicShaderVar class internal
---@overload fun(uniformName: string, uniformType: UniformType,  value: any, perInstance: boolean, requiresEntity: boolean|nil): DynamicShaderVar class external
local DynamicShaderVar = Class("DynamicShaderVar", function(self, uniformName, uniformType, value, perInstance, requiresEntity)
    self.uniformName = uniformName
    self.uniformInt = nil
    self.uniformType = uniformType
    self.value = value
    self.perInstance = perInstance or false
    self.requiresEntity = requiresEntity or false

    -- local cls = getmetatable(self)
    if type(self.value) == "function" then
        ---@param eye Position|nil
        ---@param entity Entity|nil
        ---@returns table<any>
        self.getValues = function(self, eye, entity) return { self.value(eye, entity) } end
    elseif type(self.value) == "table" then
        ---@param eye Position|nil
        ---@param entity Entity|nil
        ---@returns table<any>
        self.getValues = function(self, _, _) return self.value end
    else
        ---@param eye Position|nil
        ---@param entity Entity|nil
        ---@returns table<any>
        self.getValues = function(self, _, _) return { self.value } end
    end
end)

---@param shader Shader
function DynamicShaderVar:setUniformInt(shader)
    if shader:hasVariable(self.uniformName) then
        self.uniformInt = shader:getVariable(self.uniformName)
        -- Log.Debug(tostring(shader:name()) .. "/" .. self.uniformName .. ": " .. self.uniformInt)
        return true
    else
        Log.Warn("Shader " .. tostring(shader:name()) .. " does not have uniform: " .. self.uniformName)
        self.uniformInt = nil
        return false
    end
end

function DynamicShaderVar:setShaderVar(eye, shader, entity)
    if not self.uniformInt then
        return -- Already warned in reloadShader()
    end

    local func = UniformFuncs[self.uniformType]
    if func then
        local values = self:getValues(eye, entity)
        func(shader, self.uniformInt, unpack(values))
    end
end

return DynamicShaderVar
