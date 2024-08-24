local Material = require('_ECS_WIP_TEMP.Shared.Rendering.Material') --!temp
local GenericAutoShaderVar = require('_ECS_WIP_TEMP.Shared.Rendering.GenericAutoShaderVars')

---@class AsteroidMaterial: Material
---@overload fun(self: AsteroidMaterial): AsteroidMaterial class internal
---@overload fun(): AsteroidMaterial class external
local AsteroidMaterial = Subclass(Material, function(self)
    self:initialize("wvp", "material/asteroid")
end)

---@param vertexName string
---@param fragmentName string
---@return AsteroidMaterial
function AsteroidMaterial:initialize(vertexName, fragmentName)
    -- Set Material Variables
    self.vertexName = vertexName
    self.fragmentName = fragmentName

    -- Create Material Shader
    --TODO: Replace use of Cache.Shader
    local shader = Cache.Shader(vertexName, fragmentName)
    self.shaderState = ShaderState.Create(shader)

    -- Set Textures
    --TODO: Replace use of Cache.Texture
    self:addTexture("diffuse", Cache.Texture('rock'), Enums.TextureType.Tex2D)
    
    -- Set Auto Shader Vars
    self:addAutoShaderVar("mWorld", GenericAutoShaderVar.mWorldFunc)
    self:addAutoShaderVar("mWorldIT", GenericAutoShaderVar.mWorldITFunc)
    self:addAutoShaderVar("scale", GenericAutoShaderVar.scaleFunc)
    return self
end


return AsteroidMaterial