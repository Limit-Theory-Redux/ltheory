local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar") --!temp path
local Texture = require("_ECS_WIP_TEMP.Shared.Rendering.Texture") --!temp path

---@class Material
---@field materialName string -- Might use for Specific Material Cache?
---@field vertexName string -- 'res/shader/vertex/'
---@field fragmentName string -- 'res/shader/fragment/'
---@field textures table<Texture>
---@field shaderState ShaderState
---@field autoShaderVars table<AutoShaderVar>

---@class Material
---@overload fun(self: Material): Material class internal
---@overload fun(): Material class external
local Material = Class(function(self)
    self.textures = {}
    self.autoShaderVars = {}
end)

function Material:initialize(vertexName, fragmentName)
    self.vertexName = vertexName
    self.fragmentName = fragmentName
    --TODO: Replace use of Cache.Shader
    local shader = Cache.Shader(vertexName, fragmentName)
    self.shaderState = ShaderState.Create(shader)
end

---@param uniformName string
---@param renderFn function
---@return AutoShaderVar|nil
function Material:addAutoShaderVar(uniformName, renderFn)
    if self.shaderState:shader():hasVariable(uniformName) then
        local autoShaderVar = AutoShaderVar(uniformName, renderFn)
        autoShaderVar:setUniformInt(self.shaderState:shader():getVariable(uniformName))
        insert(self.autoShaderVars, autoShaderVar)
        return autoShaderVar
    else 
        Log.Error("Shader " .. self.materialName .. ", vertex/" .. self.vertexName .. ", fragment/" .. self.fragmentName .. ": Does not have uniform: " .. uniformName)
    end
    return nil
end

---@param textureName string
---@param tex Tex
---@param textureType TextureType
function Material:addTexture(textureName, tex, textureType) 
    local texture = Texture(textureName, tex, textureType)
    insert(self.textures, texture)
end

---@return ShaderState
function Material:getShaderState()
    return self.shaderState
end

---@return string
function Material:getMaterialName()
    return self.materialName
end

---@return string
function Material:getVertexName()
    return self.vertexName
end

---@return string
function Material:getFragmentName()
    return self.fragmentName
end

---@return table<AutoShaderVar>
function Material:getAllAutoShaderVars()
    return self.autoShaderVars
end

---@return table<Texture>
function Material:getAllTextures()
    return self.textures
end

return Material