local Material = require("Shared.Rendering.Material")
local Materials = require("Shared.Registries.Materials")

---@class TextureInfo
---@field name string
---@field tex Tex1D|Tex2D|Tex3D
---@field type UniformType
---@field settings TextureSetting

---@class ShaderVarInfo
---@field type UniformType
---@field value any
---@field perInstance boolean|nil

---@class MaterialDefinitionConstructor
---@field name string
---@field vs_name string
---@field fs_name string
---@field blendMode BlendMode
---@field textures table<TextureInfo>|nil
---@field autoShaderVars table<ShaderVarInfo>|nil
---@field constShaderVars table<ShaderVarInfo>|nil

---@class MaterialDefinition
---@field name string
---@field vs_name string
---@field fs_name string
---@field blendMode BlendMode
---@field textures table<Texture>
---@field autoShaderVars table<DynamicShaderVar>
---@field constShaderVars table<DynamicShaderVar>
---@overload fun(args: MaterialDefinitionConstructor): MaterialDefinition
local MaterialDefinition = Class("MaterialDefinition")

function MaterialDefinition.new(args)
    if not args.name then
        Log.Warn("No name Set for MaterialDefinition")
        return nil
    elseif Materials[args.name] then
        Log.Warn("Attempting to Recreate Material: " .. args.name)
        return Materials[args.name]
    end

    if not args.vs_name then
        Log.Warn("vs_name missing for MaterialDefinition: " .. args.name)
        return nil
    end

    if not args.fs_name then
        Log.Warn("fs_name missing for MaterialDefinition: " .. args.name)
        return nil
    end

    if not args.blendMode then
        Log.Warn("blendMode missing for MaterialDefinition: " .. args.name)
        return nil
    end

    local newMaterial = Material(args.vs_name, args.fs_name, args.blendMode)
    if args.textures then newMaterial:addTextures(args.textures) end
    if args.autoShaderVars then newMaterial:addAutoShaderVars(args.autoShaderVars) end
    if args.constShaderVars then newMaterial:addConstShaderVars(args.constShaderVars) end

    -- Add new Material to Materials registry
    newMaterial:reloadShader() -- reload to cache uniform ints
    Materials:new(args.name, newMaterial)

    return setmetatable({
        name = args.name,
        vs_name = args.vs_name,
        fs_name = args.fs_name,
        blendMode = args.blendMode,
        textures = args.textures,
        autoShaderVars = args.autoShaderVars,
        constShaderVars = args.constShaderVars
    }, MaterialDefinition)
end

return MaterialDefinition
