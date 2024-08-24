local Material = require('_ECS_WIP_TEMP.Shared.Rendering.Material') --!temp

---@class DebugMaterial: Material
---@overload fun(self: DebugMaterial): DebugMaterial class internal
---@overload fun(): DebugMaterial class external
local DebugMaterial = Subclass(Material, function(self)
    self:initialize("wvp", "material/devmat")
end)

---@param vertexName string
---@param fragmentName string
---@return DebugMaterial
function DebugMaterial:initialize(vertexName, fragmentName)
    -- Set Material Variables
    self.vertexName = vertexName
    self.fragmentName = fragmentName

    -- Create Material Shader
    --TODO: Replace use of Cache.Shader
    local shader = Cache.Shader(vertexName, fragmentName)
    self.shaderState = ShaderState.Create(shader)

    -- Set Textures

    -- Set Auto Shader Vars
    self:addAutoShaderVar("mWorld", GenericAutoShaderVar.mWorldFunc)
    self:addAutoShaderVar("mWorldIT", GenericAutoShaderVar.mWorldITFunc)
    self:addAutoShaderVar("scale", GenericAutoShaderVar.scaleFunc)
    return self
end

return DebugMaterial