

---@class Material
---@field materialName string
---@field textures table<Texture>
---@field shaderState ShaderState
---@field shaderVars table<ShaderVar>
---@field imWorld integer
---@field imWorldIT integer
---@field iScale integer
---@param materialName string
local Material = Class(function(self, materialName)
    ---@cast self Material
    self.materialName = materialName
    self.textures = {}
    self.shaderState = nil
    self.shaderVars = {}
    self.imWorld = nil
    self.imWorldIT = nil
    self.iScale = nil
end)

---@class autoShaderVar
---@field uniformName string
---@field callbackFn function
---@field valueType type

---@class constShaderVar
---@field uniformName string
---@field value any
---@field valueType type
---@param self constShaderVar
---@param uniformName string
---@param value any
---@param valueType 
local constShaderVar = Class(function(self, uniformName, value, valueType)
    ---@cast self constShaderVar
    self.uniformName = uniformName
    self.value = value
    self.valueType = valueType
    assert(self:typeCheck())
end)
--!TODO unclear if this TypeChecking will work with ffi types
function constShaderVar:typeCheck() 
    ---@cast self constShaderVar
    if type(self.value) == self.valueType then return true
    else return false end
end

---@class Texture
---@field textureName string --"texDiffuse", "texNormal", uniform name of the texture
---@field tex Tex
---@param textureName string
---@param tex Tex
local Texture = Class(function(self, textureName, tex)
    self.textureName = textureName
    self.tex = tex
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

---@class ShaderVar 
---@field uniformName string
---@field renderFn function