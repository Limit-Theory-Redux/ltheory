---@class Texture
---@field textureName string --"texDiffuse", "texNormal", uniform name of the texture
---@field textureType TextureType --"Tex1D", "Tex2D", ...
---@field tex Tex

---@param self Texture
---@param textureName string
---@param tex Tex
---@param textureType TextureType
---@class Texture
---@overload fun(self: Texture, textureName: string, tex: Tex, textureType: TextureType): Texture class internal
---@overload fun(textureName: string, tex: Tex, textureType: TextureType): Texture class external
local Texture = Class(function(self, textureName, tex, textureType)
    self.textureName = textureName
    self.tex = tex
    self.textureType = textureType
    self.tex:acquire(tex)
    self.tex:setTextureState()
end)

function Texture:setTextureState()
    self.tex:genMipmap()
    self.tex:setMagFilter(TexFilter.Linear)
    self.tex:setMinFilter(TexFilter.LinearMipLinear)
    self.tex:setAnisotropy(16)
    self.tex:setWrapMode(TexWrapMode.Repeat)
end

return Texture