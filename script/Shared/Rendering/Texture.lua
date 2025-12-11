---@class TextureSetting
---@field magFilter   TexFilter
---@field minFilter   TexFilter
---@field anisotropy   integer
---@field wrapS        TexWrapMode
---@field wrapT        TexWrapMode
---@field wrapR        TexWrapMode
---@field genMipMap    boolean

---@class Texture
---@field texName string
---@field tex     Tex1D|Tex2D|Tex3D|TexCube
---@field texType UniformType
---@field texSettings TextureSetting
local Texture = Class("Texture", function(self, texName, tex, texType, texSettings)
    self.texName     = texName
    self.tex         = tex
    self.texType     = texType or Enums.UniformType.Tex2D

    -- Default settings
    self.texSettings = texSettings or {
        magFilter  = TexFilter.Linear,
        minFilter  = TexFilter.LinearMipLinear,
        anisotropy = 16,
        wrapS      = TexWrapMode.Repeat,
        wrapT      = TexWrapMode.Repeat,
        wrapR      = TexWrapMode.Repeat,
        genMipMap  = true,
    }

    self:setTextureState()
end)

-- Apply sampler state — per texture type
function Texture:setTextureState()
    if not self.tex then return end

    local tex = self.tex
    local s = self.texSettings

    -- Tex2D
    if self.texType == Enums.UniformType.Tex2D and ffi.istype('Tex2D', tex) then
        ---@cast tex Tex2D
        if s.genMipMap then tex:genMipmap() end
        if s.magFilter then tex:setMagFilter(s.magFilter) end
        if s.minFilter then tex:setMinFilter(s.minFilter) end
        if s.anisotropy then tex:setAnisotropy(s.anisotropy) end
        if s.wrapS then tex:setWrapMode(s.wrapS) end
        return
    end

    -- TexCube
    if self.texType == Enums.UniformType.TexCube and ffi.istype('TexCube', tex) then
        ---@cast tex TexCube
        if s.genMipMap then tex:genMipmap() end
        if s.magFilter then tex:setMagFilter(s.magFilter) end
        if s.minFilter then tex:setMinFilter(s.minFilter) end
        -- Note: TexCube does NOT support anisotropy or per-axis wrap
        return
    end

    -- Tex3D
    if self.texType == Enums.UniformType.Tex3D and ffi.istype('Tex3D', tex) then
        ---@cast tex Tex3D
        if s.genMipMap then tex:genMipmap() end
        if s.magFilter then tex:setMagFilter(s.magFilter) end
        if s.minFilter then tex:setMinFilter(s.minFilter) end
        --if s.anisotropy then tex:setAnisotropy(s.anisotropy) end
        if s.wrapS then tex:setWrapMode(s.wrapS) end
        if s.wrapT then tex:setWrapMode(s.wrapT) end
        if s.wrapR then tex:setWrapMode(s.wrapR) end
        return
    end

    -- Tex1D
    if self.texType == Enums.UniformType.Tex1D and ffi.istype('Tex1D', tex) then
        ---@cast tex Tex1D
        if s.genMipMap then tex:genMipmap() end
        if s.magFilter then tex:setMagFilter(s.magFilter) end
        if s.minFilter then tex:setMinFilter(s.minFilter) end
        --if s.anisotropy then tex:setAnisotropy(s.anisotropy) end
        if s.wrapS then tex:setWrapMode(s.wrapS) end
        return
    end

    Log.Warn("Texture:setTextureState() - unsupported texture type: " .. tostring(self.texType))
end

-- Bind to ShaderState
function Texture:setTextureToShaderState(shaderState)
    if not self.tex then return end

    local fn = ({
        [Enums.UniformType.Tex1D]   = shaderState.setTex1D,
        [Enums.UniformType.Tex2D]   = shaderState.setTex2D,
        [Enums.UniformType.Tex3D]   = shaderState.setTex3D,
        [Enums.UniformType.TexCube] = shaderState.setTexCube,
    })[self.texType]

    if fn then
        fn(shaderState, tostring(self.texName), self.tex) --* i think this should probably not use tostring() but ok for now?*
    else
        Log.Warn("ShaderState missing setter for UniformType: " .. tostring(self.texType))
    end
end

return Texture
