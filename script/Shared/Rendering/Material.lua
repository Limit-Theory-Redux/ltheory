local DynamicShaderVar = require("Shared.Rendering.DynamicShaderVar")
local Texture = require("Shared.Rendering.Texture")
local UniformFuncs = require("Shared.Rendering.UniformFuncs")

---@class Material
---@field vs string -- 'res/shader/vertex/'
---@field fs string -- 'res/shader/fragment/'
---@field blendMode BlendMode
---@field textures table<Texture>
---@field shaderState ShaderState
---@field autoShaderVars table<DynamicShaderVar>
---@field constShaderVars table<DynamicShaderVar>
---@field staticShaderVars table<DynamicShaderVar>

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
    ---@param texInfo TextureInfo
    for texInfo in Iterator(textures) do
        local tex = Texture(texInfo.name, texInfo.tex, texInfo.type, texInfo.setting)
        tex:setTextureToShaderState(self.shaderState)
        insert(self.textures, tex)
    end
end

---@param shaderVars table<ShaderVarInfo>
function Material:addAutoShaderVars(shaderVars)
    for name, shaderVarInfo in pairs(shaderVars) do
        local autoShaderVar = DynamicShaderVar(name, shaderVarInfo.type, shaderVarInfo.value, false,
            shaderVarInfo.perInstance)
        insert(self.autoShaderVars, autoShaderVar)
    end
end

---@param shaderVars table<ShaderVarInfo>
function Material:addConstShaderVars(shaderVars)
    for name, shaderVarInfo in pairs(shaderVars) do
        local constShaderVar = DynamicShaderVar(name, shaderVarInfo.type, shaderVarInfo.value, true, true)
        insert(self.constShaderVars, constShaderVar)
    end
end

---@param name string
---@param type UniformType
---@param value any
function Material:addStaticShaderVar(name, type, value)
    local staticShaderVar = DynamicShaderVar(name, type, value, false, false)
    staticShaderVar:setUniformInt(self.shaderState:shader())
    insert(self.staticShaderVars, staticShaderVar)
end

function Material:reloadShader()
    local shader = Cache.Shader(self.vs, self.fs)
    if not shader then return end

    self.shaderState = ShaderState.Create(shader)
    self.shader = shader

    local function cache(vars)
        for _, v in ipairs(vars) do
            v:setUniformInt(shader)
        end
    end

    cache(self.autoShaderVars)
    cache(self.constShaderVars)
    cache(self.staticShaderVars or {})

    -- Set const vars
    for _, v in ipairs(self.constShaderVars) do
        v:setShaderVar(nil, shader, nil)
    end

    -- Rebind textures
    for _, tex in pairs(self.textures) do
        local name = tex.texName
        if shader:hasVariable(name) then
            local loc = shader:getVariable(name)
            local fnName = ({
                [Enums.UniformType.Tex2D]   = "iSetTex2D",
                [Enums.UniformType.TexCube] = "iSetTexCube",
            })[tex.type]
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
        shaderVar:setShaderVar(eye, shader, entity)
    end
    for _, shaderVar in ipairs(self.staticShaderVars) do
        shaderVar:setShaderVar(eye, shader, entity)
    end
end

function Material:setTexture(texName, tex, texType)
    -- Infer type if not provided
    texType = texType or (ffi.istype("TexCube", tex) and Enums.UniformType.TexCube
        or ffi.istype("Tex2D", tex) and Enums.UniformType.Tex2D
        or ffi.istype("Tex3D", tex) and Enums.UniformType.Tex3D
        or error("Unsupported texture type"))

    local texInfo = {
        name = texName,
        tex = tex,
        type = texType,
        settings = {
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
        c.textures[k] = Texture(k, tex.tex, tex.type, tex.settings)
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
