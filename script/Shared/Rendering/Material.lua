local ffi = require("ffi")
local Cache = require("Render.Cache")

-- The Rust `Material` (FFI type, `Material.Create`) is shadowed by the class
-- below within this file.
local GpuMaterial = Material

--- Samplers a texture gets when neither the `MaterialType` nor the call names
--- one (what the old `Texture` class did to every material texture: linear
--- filtering with mips, repeat wrapping, 16x anisotropy for 2D textures).
---@param tex Tex1D|Tex2D|Tex3D|TexCube
---@return integer sampler
local function defaultSampler(tex)
    if ffi.istype("TexCube", tex) then return Samplers.LinearMipClamp end
    if ffi.istype("Tex3D", tex) then return Samplers.LinearMipRepeat end
    if ffi.istype("Tex1D", tex) then return Samplers.LinearRepeat end
    return Samplers.LinearMipRepeatAniso
end

--- Samplers that read the mip chain: a texture sampled with one of them needs
--- its mips generated.
local mipSamplers = {
    [Samplers.LinearMipClamp] = true,
    [Samplers.LinearMipRepeat] = true,
    [Samplers.LinearMipRepeatAniso] = true,
}

---@param tex Tex1D|Tex2D|Tex3D|TexCube
---@param sampler integer
local function ensureMips(tex, sampler)
    if mipSamplers[sampler] and tex.genMipmap then tex:genMipmap() end
end

-- `T*` ctype of a parameter struct, built once per struct type.
local pointerTypes = setmetatable({}, { __mode = "k" })
local function pointerTo(T)
    local pointerType = pointerTypes[T]
    if not pointerType then
        pointerType = ffi.typeof("$ *", T)
        pointerTypes[T] = pointerType
    end
    return pointerType
end

--- Write `value` into the parameter field `p[name]`: numbers directly, vectors
--- component by component (`Vec3f` into a `vec3`, ...).
---@param p ffi.cdata*
---@param name string
---@param value any
local function assignField(p, name, value)
    if type(value) == "number" or type(value) == "boolean" then
        p[name] = value
        return
    end
    local field = p[name]
    local components = ffi.sizeof(field) / 4 -- Vec2f/Vec3f/Vec4f
    field.x = value.x
    if components >= 2 then field.y = value.y end
    if components >= 3 then field.z = value.z end
    if components >= 4 then field.w = value.w end
end

-- Registry of all live materials for hot-reload (weak: never keeps one alive).
local allMaterials = setmetatable({}, { __mode = "k" })

---@class Material
---@field type MaterialType
---@field handle ffi.cdata*   the Rust material (bind group, parameter slice, pipeline)
---@field blend BlendMode     the scene bucket it draws in
---@field perDraw fun(entity: Entity, user: ffi.cdata*)|nil   fills `drawUser` of each drawn mesh
---@field textures table<string, Tex1D|Tex2D|Tex3D|TexCube>   keeps the textures alive
---@field paramsPtr ffi.cdata*|nil  the typed `MaterialParams` struct
---@overload fun(self: Material, matType: MaterialType): Material class internal
---@overload fun(matType: MaterialType): Material class external
local Material = Class("Material", function(self, matType)
    local state = matType.state
    self.type = matType
    self.blend = state.blend
    self.perDraw = matType.perDraw
    self.textures = {}
    self.handle = GpuMaterial.Create(matType.shader, state.blend, state.cull, state.depthTest, state.depthWrite)

    -- Typed view of the parameter block, with the type's defaults in it.
    if self.handle:getParamsSize() > 0 then
        local T = matType.paramsType or matType.shader:blockType("MaterialParams")
        self.paramsPtr = ffi.cast(pointerTo(T), self.handle:paramsPointer())
        for name, value in pairs(matType.defaults) do
            assignField(self.paramsPtr, name, value)
        end
    end

    for name, spec in pairs(matType.textures) do
        if spec.tex then
            self:setTexture(name, spec.tex, spec.sampler, false)
        end
    end

    self.handle:commit()
    allMaterials[self] = true
end)

--- The typed `MaterialParams` struct (`shader:blockType('MaterialParams')`):
--- write its fields, then `commit()`. Errors if the shader has no such block.
---@return ffi.cdata*
function Material:params()
    local p = self.paramsPtr
    if not p then
        error(string.format("Material %s: its shader has no MaterialParams block", tostring(self.type.name)), 2)
    end
    return p
end

--- Bind `tex` to the shader sampler `name` (type-checked against the shader's
--- group-1 layout). Takes effect at the next `commit()`. The mip chain is
--- generated when the sampler reads it, unless `genMips` is false.
---@param name string
---@param tex Tex1D|Tex2D|Tex3D|TexCube
---@param sampler integer|nil  default: the type's sampler for `name`, else by texture kind
---@param genMips boolean|nil
function Material:setTexture(name, tex, sampler, genMips)
    local spec = self.type.textures[name]
    sampler = sampler or (spec and spec.sampler) or defaultSampler(tex)
    if genMips ~= false then ensureMips(tex, sampler) end
    self.handle:setTexture(name, tex:view(), sampler)
    self.textures[name] = tex
end

--- Send the parameters to the GPU and recreate the bind group if a texture
--- changed. One write; call it when something changed, never per draw, and not
--- while a render pass is open.
function Material:commit()
    self.handle:commit()
end

--- The material's shader was hot reloaded: adopt its new layout. The Rust
--- material copies the parameters over by member name into a block of the new
--- size (new members are zero, here they get the type's defaults), keeps the
--- textures by sampler name and gives back its arena slice and bind group; the
--- typed view is recast to the regenerated type, and `commit()` writes the
--- parameters to a new slice and makes the new bind group.
---@return string report
function Material:refresh()
    local report = ffi.string(self.handle:refreshShader())

    if self.handle:getParamsSize() > 0 then
        local T = self.type.paramsType or self.type.shader:blockType("MaterialParams")
        self.paramsPtr = ffi.cast(pointerTo(T), self.handle:paramsPointer())
        for name in (report:match("added=(%S*)") or ""):gmatch("[^,]+") do
            local value = self.type.defaults[name]
            if value ~= nil then assignField(self.paramsPtr, name, value) end
        end
    else
        self.paramsPtr = nil
    end

    self.handle:commit()
    return report
end

--- Called through `Cache.OnShaderReload` after `shader` was reloaded: ask each
--- `MaterialType` that draws with it to regenerate its parameter ctype, then
--- refresh its live materials. Returns the blocks it took care of.
---@param shader Shader
---@param changed table<string, boolean>  uniform blocks whose layout changed
---@return table<string, boolean> handled
function Material.OnShaderReloaded(shader, changed)
    local Materials = require("Shared.Registries.Materials")
    Materials.each(function(matType)
        if matType.shader == shader then
            local before = matType.paramsHash
            matType:regenerate()
            if matType.paramsHash ~= before then
                Log.Info("Material type %s: MaterialParams layout changed, ctype regenerated", tostring(matType.name))
            end
        end
    end)

    local count = 0
    for mat in pairs(allMaterials) do
        if mat.type.shader == shader then
            local report = mat:refresh()
            count = count + 1
            if changed.MaterialParams then
                Log.Info("Material %s refreshed: %s", tostring(mat.type.name), report)
            end
        end
    end
    if count > 0 then
        Log.Info("Shader hot-reload: %d material(s) refreshed", count)
    end
    return { MaterialParams = true }
end

--- Refresh every live material (after `Cache.ReloadShaders`, which already
--- does it for the shaders it reloaded; kept for manual use).
function Material.ReloadAll()
    local count = 0
    for mat in pairs(allMaterials) do
        mat:refresh()
        count = count + 1
    end
    Log.Info("Material hot-reload: %d materials refreshed", count)
    return count
end

--- Used by `MaterialType` to prepare the textures its materials share.
Material.EnsureMips = ensureMips

Cache.OnShaderReload(Material.OnShaderReloaded)

return Material
