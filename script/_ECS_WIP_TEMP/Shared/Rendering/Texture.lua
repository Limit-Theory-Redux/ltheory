--Note:
--[[
Going off of old Material.lua we have to use :setTex2D on the shaderState which requires a string comparison. Shouldn't we be able to set w/ an int?
Other Places like Dust.lua creates a local Tex2D and uses setTex2D on the Shader

Where should we use Shader vs. ShaderState, why would can't we store uniformInt for ShaderState? 
Isn't ShaderState just a copy of Shader w/ extra info?
Aren't we reusing shaders anyways?
]]--
---@class TextureSetting
---@field magFilter TexFilter
---@field minFilter TexFilter
---@field anisotropy integer
---@field wrapMode TexWrapMode
---@field genMipMap boolean

---@class Texture
---@field texName string --"texDiffuse", "texNormal", uniform name of the texture
---@field texType UniformType
---@field tex Tex
---@field texSettings TextureSetting

---@class Texture
---@overload fun(self: Texture, texName: string, tex: Tex, texType: UniformType, texSettings: TextureSetting|nil): Texture class internal
---@overload fun(texName: string, tex: Tex, texType: UniformType, texSettings: TextureSetting|nil): Texture class external
local Texture = Class(function(self, texName, tex, texType, texSettings)
    self.texName = texName
    self.tex = tex
    self.texType = texType
    if texSettings == nil then
        self.texSettings = {
            magFilter = TexFilter.Linear,
            minFilter = TexFilter.LinearMipLinear,
            anisotropy = 16,
            wrapMode = TexWrapMode.Repeat,
            genMipMap = true
        }
    else
        self.texSettings = texSettings
    end
    self.tex:acquire()
    self:setTextureState()
end)

function Texture:setTextureState()
    if self.texSettings.genMipMap then self.tex:genMipmap() end
    self.tex:setMagFilter(self.texSettings.magFilter)
    self.tex:setMinFilter(self.texSettings.minFilter)
    self.tex:setAnisotropy(self.texSettings.anisotropy)
    self.tex:setWrapMode(self.texSettings.wrapMode)
end

---@param shaderState ShaderState
function Texture:setTextureToShaderState(shaderState)
    local setTex = {
        [Enums.UniformType.Tex1D] = function()
            shaderState:setTex1D(self.texName, self.tex) end,
        [Enums.UniformType.Tex2D] = function()
            shaderState:setTex2D(self.texName, self.tex) end,
        [Enums.UniformType.Tex3D] = function()
            shaderState:setTex3D(self.texName, self.tex) end,
        [Enums.UniformType.TexCube] = function()
            shaderState:setTexCube(self.texName, self.tex) end
    }
    setTex[self.texType]()
end

return Texture