local Material = require("Shared.Rendering.Material")
local Materials = require("Shared.Registries.Materials")
local Pipelines = require("Render.Pipelines")

---@class MaterialTextureSpec
---@field tex Tex1D|Tex2D|Tex3D|TexCube|nil  default texture (shared by every instance); set per instance otherwise
---@field sampler integer|nil                 a `Samplers.*` preset or `Sampler.Get(desc)`; default by texture kind

---@class MaterialTypeState
---@field blend BlendMode          picks the scene bucket (opaque, additive, alpha) and the blending
---@field cull CullFace|nil        default: that of the scene pass of the bucket
---@field depthTest boolean|nil    default: that of the scene pass of the bucket
---@field depthWrite boolean|nil   default: that of the scene pass of the bucket

---@class MaterialTypeConstructor
---@field name string
---@field shader string[]                          { vertex, fragment } shader names for `Cache.Shader`
---@field state MaterialTypeState
---@field defaults table<string, any>|nil          `MaterialParams` fields every instance starts with
---@field textures table<string, MaterialTextureSpec>|nil
---@field perDraw fun(entity: Entity, user: ffi.cdata*)|nil
---                  fills `drawUser` (a `Vec4f[7]`) of each drawn mesh; called for the meshes that survive culling

---@class MaterialType
---@field name string
---@field shader Shader
---@field state MaterialTypeState  with the scene-pass defaults filled in
---@field defaults table<string, any>
---@field textures table<string, MaterialTextureSpec>
---@field perDraw fun(entity: Entity, user: ffi.cdata*)|nil
---@field paramsType ffi.ctype*|nil  the LuaJIT type of the shader's `MaterialParams` block (see `regenerate`)
---@field paramsHash integer         `shader:blockHash('MaterialParams')` the type was made from
---@overload fun(args: MaterialTypeConstructor): MaterialType
local MaterialType = Class("MaterialType")

--- Define and register a material type.
---@param args MaterialTypeConstructor
---@return MaterialType|nil
function MaterialType.new(args)
    if not args.name then
        Log.Warn("No name set for MaterialType")
        return nil
    end
    if Materials[args.name] then
        Log.Warn("Attempting to recreate material type: " .. args.name)
        return Materials[args.name]
    end
    if type(args.shader) ~= "table" or not args.shader[1] or not args.shader[2] then
        Log.Warn("shader = { vertex, fragment } missing for MaterialType: " .. args.name)
        return nil
    end
    if not args.state or args.state.blend == nil then
        Log.Warn("state.blend missing for MaterialType: " .. args.name)
        return nil
    end

    -- Fixed-function state: what the scene pass of the bucket used to push.
    local scene = Pipelines.Scene[args.state.blend] or Pipelines.Alpha
    local function pick(value, default)
        if value ~= nil then return value end
        return default
    end
    local state = {
        blend = args.state.blend,
        cull = pick(args.state.cull, scene.cull),
        depthTest = pick(args.state.depthTest, scene.depthTest),
        depthWrite = pick(args.state.depthWrite, scene.depthWrite),
    }

    local self = setmetatable({
        name = args.name,
        shader = Cache.Shader(args.shader[1], args.shader[2]),
        state = state,
        defaults = args.defaults or {},
        textures = args.textures or {},
        perDraw = args.perDraw,
    }, MaterialType)

    self:regenerate()

    -- Textures every instance shares need their mip chain once.
    for _, spec in pairs(self.textures) do
        if spec.tex then
            Material.EnsureMips(spec.tex, spec.sampler or Samplers.LinearMipRepeatAniso)
        end
    end

    return Materials:new(args.name, self)
end

--- (Re)make the LuaJIT type of the shader's `MaterialParams` block from its
--- current layout. Called when the type is created and after the shader was
--- hot reloaded (`Material.OnShaderReloaded`): a reload gives the shader a new
--- block layout, and with it a new ctype.
function MaterialType:regenerate()
    local shader = self.shader
    if shader:blockSize("MaterialParams") > 0 then
        self.paramsType = shader:blockType("MaterialParams")
        self.paramsHash = shader:blockHash("MaterialParams")
    else
        self.paramsType = nil
        self.paramsHash = 0
    end
end

--- A material with its own parameters and bind group: `instance:params()` for
--- the typed `MaterialParams`, `setTexture(name, tex)`, `commit()`.
---@return Material
function MaterialType:instance()
    return Material(self)
end

return MaterialType
