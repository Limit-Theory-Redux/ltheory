local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar") --!temp path
local ConstShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.ConstShaderVar") --!temp path
local Texture = require("_ECS_WIP_TEMP.Shared.Rendering.Texture") --!temp path

---@class Material
---@field vertex string -- 'res/shader/vertex/'
---@field fragment string -- 'res/shader/fragment/'
---@field blendMode BlendMode
---@field textures table<Texture>
---@field shaderState ShaderState
---@field autoShaderVars table<AutoShaderVar>
---@field constShaderVars table<ConstShaderVar>

---@class Material
---@overload fun(self: Material, materialDefinition: MaterialDefinition|nil): Material class internal
---@overload fun(materialDefinition: MaterialDefinition|nil): Material class external
local Material = Class(function(self, materialDefinition)
    if materialDefinition then
        self.vertex = materialDefinition.vertex
        self.fragment = materialDefinition.fragment
        self.blendMode = materialDefinition.blendMode
        local shader = Cache.Shader(self.vertexName, self.fragmentName)
        self.shaderState = ShaderState.Create(shader)

        self:addTextures(materialDefinition.textures)
        self:addAutoShaderVars(materialDefinition.autoShaderVars)
        self:addConstShaderVars(materialDefinition.constShaderVars)
    end
end)

---@param textures table<TextureInfo>
function Material:addTextures(textures)
    for _, texture in ipairs(textures) do
        local tex = Texture(texture.texName, texture.tex, texture.texType, texture.texSetting)
        tex:setTextureToShaderState(self.shaderState)
        insert(self.textures, tex)
    end
end

function Material:addAutoShaderVars(autoShaderVars)
    for _, autoShaderVar in ipairs(autoShaderVars) do
        local shaderVar = AutoShaderVar(autoShaderVar.uniformName, autoShaderVar.uniformType, autoShaderVar.callbackFn)
        shaderVar:setUniformInt(self.shaderState:shader())
        insert(self.autoShaderVars, shaderVar)
    end
end

function Material:addConstShaderVars(constShaderVars)
    for _, constShaderVar in ipairs(constShaderVars) do
        local shaderVar = ConstShaderVar(constShaderVar.uniformName, constShaderVar.uniformType, constShaderVar.callbackFn)
        shaderVar:setUniformInt(self.shaderState:shader())
        insert(self.constShaderVars, shaderVar)
    end
end

function Material:reload()
    if self.shaderState then self.shaderState:free() end
    local shader = Cache.Shader(self.vertex, self.fragment)
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

function Material:getUnsetConstShaderVars() 
    local shaderVars = {}
    for _, shaderVar in ipairs(self.constShaderVars) do
        if not shaderVar:hasUniformValues() then insert(shaderVars, shaderVar) end
    end
    return shaderVars
end

function Material:setAllConstUniformValues(entity)
    for _, shaderVar in ipairs(self.constShaderVars) do
        shaderVar:setUniformValues(entity)
    end
end

function Material:setAllShaderVars(renderState, entity)
    local shader = self.shaderState:shader()
    for _, shaderVar in ipairs(self.autoShaderVars) do
        shaderVar:setShaderVar(renderState, shader, entity)
    end
    for _, shaderVar in ipairs(self.constShaderVars) do
        shaderVar:setShaderVar(shader)
    end

end

---@return ShaderState
function Material:getShaderState()
    return self.shaderState
end

---@return string
function Material:getVertex()
    return self.vertex
end

---@return string
function Material:getFragment()
    return self.fragment
end

return Material