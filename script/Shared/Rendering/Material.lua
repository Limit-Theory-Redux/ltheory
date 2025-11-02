local AutoShaderVar = require("Shared.Rendering.AutoShaderVar")
local ConstShaderVar = require("Shared.Rendering.ConstShaderVar")
local Texture = require("Shared.Rendering.Texture")
local UniformFuncs = require("Shared.Rendering.UniformFuncs")

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
local Material = Class("Material", function(self, vs_name, fs_name, blendMode)
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
        local autoShaderVar = AutoShaderVar(shaderVarInfo.uniformName, shaderVarInfo.uniformType, shaderVarInfo.callbackFn,
            shaderVarInfo.perInstance)
        insert(self.autoShaderVars, autoShaderVar)
    end
end

---@param shaderVars table<ShaderVarInfo>
function Material:addConstShaderVars(shaderVars)
    for _, shaderVarInfo in ipairs(shaderVars) do
        local constShaderVar = ConstShaderVar(shaderVarInfo.uniformName, shaderVarInfo.uniformType, true)
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

function Material:reloadShader()
    local shader = Cache.Shader(self.vs, self.fs)
    if not shader then return end

    self.shaderState = ShaderState.Create(shader)
    self.shader = shader

    local function cache(vars)
        for _, v in ipairs(vars) do
            if shader:hasVariable(v.uniformName) then
                v.uniformInt = shader:getVariable(v.uniformName)
            else
                Log.Warn("Shader " .. tostring(shader) .. ": Does not have uniform: " .. v.uniformName)
                v.uniformInt = nil
            end
        end
    end

    cache(self.autoShaderVars)
    cache(self.constShaderVars)
    cache(self.staticShaderVars or {})

    -- Set const vars
    for _, v in ipairs(self.constShaderVars) do
        if v.uniformInt then
            local values = { v.callbackFn() }
            local func = UniformFuncs[v.uniformType]
            if func then
                func(shader, v.uniformInt, unpack(values))
            end
        end
    end

    -- Rebind textures
    for _, tex in pairs(self.textures) do
        local name = tex.texName
        if shader:hasVariable(name) then
            local loc = shader:getVariable(name)
            local fnName = ({
                [Enums.UniformType.Tex2D]   = "iSetTex2D",
                [Enums.UniformType.TexCube] = "iSetTexCube",
            })[tex.texType]
            if fnName and shader[fnName] then
                shader[fnName](shader, loc, tex.tex)
            end
        end
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
        shaderVar:setShaderVar(shader, entity)
    end
    for _, shaderVar in ipairs(self.staticShaderVars) do
        shaderVar:setShaderVar(shader)
    end
end

function Material:setTexture(name, tex, texType)
    -- Infer type if not provided
    texType = texType or (ffi.istype("TexCube", tex) and Enums.UniformType.TexCube
        or ffi.istype("Tex2D", tex) and Enums.UniformType.Tex2D
        or ffi.istype("Tex3D", tex) and Enums.UniformType.Tex3D
        or error("Unsupported texture type"))

    local texInfo = {
        texName = name,
        tex = tex,
        texType = texType,
        texSettings = {
            genMipMap = true,
            magFilter = TexFilter.Linear,
            minFilter = TexFilter.LinearMipLinear,
            anisotropy = 16,
            wrapS = TexWrapMode.Repeat,
            wrapT = TexWrapMode.Repeat,
            wrapR = TexWrapMode.Repeat,
        }
    }

    self:addTextures({ texInfo })
end

---@return Material ClonedMaterial
function Material:clone()
    local c = Material(self.vs, self.fs, self.blendMode)
    c.textures = {}
    c.autoShaderVars = { unpack(self.autoShaderVars or {}) }
    c.constShaderVars = { unpack(self.constShaderVars or {}) }

    for k, tex in pairs(self.textures or {}) do
        c.textures[k] = Texture(k, tex.tex, tex.texType, tex.texSettings)
        c.textures[k]:setTextureToShaderState(c.shaderState)
    end

    return c
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

---@return BlendMode
function Material:getBlendMode()
    return self.blendMode
end

return Material
