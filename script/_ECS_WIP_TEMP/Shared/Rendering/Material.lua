local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar")   --!temp path
local ConstShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.ConstShaderVar") --!temp path
local Texture = require("_ECS_WIP_TEMP.Shared.Rendering.Texture")               --!temp path

---@class Material
---@field vs string -- 'res/shader/vertex/'
---@field fs string -- 'res/shader/fragment/'
---@field blendMode BlendMode
---@field textures table<Texture>
---@field shaderState ShaderState
---@field autoShaderVars table<AutoShaderVar>
---@field constShaderVars table<ConstShaderVar>
---@field staticShaderVars table<ConstShaderVar>

---@class Material
---@overload fun(self: Material, vs_name: string, fs_name: string, blendMode: BlendMode): Material class internal
---@overload fun(vs_name: string, fs_name: string, blendMode: BlendMode): Material class external
local Material = Class(function(self, vs_name, fs_name, blendMode)
    self.vs = vs_name
    self.fs = fs_name
    self.blendMode = blendMode
    self.textures = {}
    self.autoShaderVars = {}
    self.constShaderVars = {}
    self.staticShaderVars = {}

    -- Create Shader and ShaderState
    local shader = Cache.Shader(self.vs, self.fs)
    self.shaderState = ShaderState.Create(shader)
end)

---@param textures table<TextureInfo>
function Material:addTextures(textures)
    for _, texture in ipairs(textures) do
        local tex = Texture(texture.texName, texture.tex, texture.texType, texture.texSetting)
        tex:setTextureToShaderState(self.shaderState)
        insert(self.textures, tex)
    end
end

---@param shaderVars table<ShaderVarInfo>
function Material:addAutoShaderVars(shaderVars)
    for _, shaderVarInfo in ipairs(shaderVars) do
        local autoShaderVar = AutoShaderVar(shaderVarInfo.uniformName, shaderVarInfo.uniformType, shaderVarInfo.callbackFn)
        autoShaderVar:setUniformInt(self.shaderState:shader())
        insert(self.autoShaderVars, autoShaderVar)
    end
end

---@param shaderVars table<ShaderVarInfo>
function Material:addConstShaderVars(shaderVars)
    for _, shaderVarInfo in ipairs(shaderVars) do
        local constShaderVar = ConstShaderVar(shaderVarInfo.uniformName, shaderVarInfo.uniformType, true)
        constShaderVar:setUniformInt(self.shaderState:shader())
        constShaderVar:setCallbackFn(shaderVarInfo.callbackFn)
        insert(self.constShaderVars, constShaderVar)
    end
end

---@param uniformName string
---@param uniformType UniformType
function Material:addStaticShaderVar(uniformName, uniformType, callbackFn)
    local staticShaderVar = ConstShaderVar(uniformName, uniformType, false)
    staticShaderVar:setUniformInt(self.shaderState:shader())
    staticShaderVar:setCallbackFn(callbackFn)
    insert(self.staticShaderVars, staticShaderVar)
end

function Material:reload()
    if self.shaderState then self.shaderState:free() end
    local shader = Cache.Shader(self.vs, self.fs)
    self.shaderState = ShaderState.Create(shader)

    for _, texture in ipairs(self.textures) do
        texture:setTextureToShaderState(self.shaderState)
    end
    for _, shaderVar in ipairs(self.autoShaderVars) do
        shaderVar:setUniformInt(shader)
    end
    for _, shaderVar in ipairs(self.constShaderVars) do
        shaderVar:setUniformInt(shader)
        shaderVar:resetUniformValues()
    end
end

---Set Uniform Values for Materials Shader
---@param eye Position Camera Position
---@param entity Entity
function Material:setAllShaderVars(eye, entity)
    local shader = self.shaderState:shader()
    for _, shaderVar in ipairs(self.autoShaderVars) do
        shaderVar:setShaderVar(eye, shader, entity)
    end
    for _, shaderVar in ipairs(self.constShaderVars) do
        shaderVar:setShaderVar(shader)
    end
    for _, shaderVar in ipairs(self.staticShaderVars) do
        shaderVar:setShaderVar(shader)
    end
end

---@return Material ClonedMaterial
function Material:clone()
    local cloneMaterial = Material(self.vs, self.fs, self.blendMode)
    cloneMaterial.textures = self.textures
    cloneMaterial.autoShaderVars = self.autoShaderVars
    cloneMaterial.constShaderVars = self.constShaderVars
    return cloneMaterial
end

---@return ShaderState
function Material:getShaderState()
    return self.shaderState
end

---@return string
function Material:getVertex()
    return self.vs
end

---@return string
function Material:getFragment()
    return self.fs
end

return Material
