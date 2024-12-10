local Material = require("_ECS_WIP_TEMP.Shared.Rendering.Material")
local Materials = require("_ECS_WIP_TEMP.Shared.Registries.Materials")

local MaterialDefinition = {}
MaterialDefinition.__index = MaterialDefinition

---@class Type
---@field MaterialDefinition integer

local typeInt = Enums.Type:createType("MaterialDefinition")

local sharedMeta = {
    __index = MaterialDefinition,
    __type = typeInt,
    __tostring = function(self)
        return Enums.Type:getName(typeInt)
    end
}

local classMeta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class MaterialDefinition
---@field name string
---@field vs_name string
---@field fs_name string
---@field blendMode BlendMode
---@field textures table<Texture>
---@field autoShaderVars table<AutoShaderVar>
---@field constShaderVars table<ConstShaderVar>

---@class MaterialDefinitionConstructor
---@field name string
---@field vs_name string
---@field fs_name string
---@field blendMode BlendMode
---@field textures table<TextureInfo>|nil
---@field autoShaderVars table<ShaderVarInfo>|nil
---@field constShaderVars table<ShaderVarInfo>|nil

---@class TextureInfo
---@field texName string
---@field tex Tex
---@field texType UniformType
---@field texSettings TextureSetting

---@class ShaderVarInfo
---@field uniformName string
---@field uniformType UniformType
---@field callbackFn function

---@private
---@param args MaterialDefinitionConstructor
---@return MaterialDefinition|nil
function MaterialDefinition:new(args)
    if not args.name then
        Log.Warn("No name Set for MaterialDefinition")
        return nil
    elseif Material[args.name] then
        Log.Warn("Attempting to Recreate Material: " .. args.name)
        return Material[args.name]
    end

    if not args.vs_name or not args.fs_name or not args.blendMode then
        Log.Warn("vs_name, fs_name, or blendMode missing for MaterialDefinition: " .. args.name)
        return nil
    end

    -- Create newMaterial
    local newMaterial = Material(args.vs_name, args.fs_name, args.blendMode)
    -- Set Textures
    if args.textures then
        newMaterial:addTextures(args.textures)
    end
    -- Set AutoShaderVars
    if args.autoShaderVars then
        newMaterial:addAutoShaderVars(args.autoShaderVars)
    end
    -- Set ConstShaderVars
    if args.constShaderVars then
        newMaterial:addConstShaderVars(args.constShaderVars)
    end

    -- Add New Material to Materials Registery
    Materials:new(args.name, newMaterial)

    -- sets newMaterialDefinition and returns it
    local newMaterialDefinition = setmetatable({
        name = args.name,
        vs_name = args.vs_name,
        fs_name = args.fs_name,
        blendMode = args.blendMode,
        textures = args.textures,
        autoShaderVars = args.autoShaderVars,
        constShaderVars = args.constShaderVars
    }, sharedMeta)

    return newMaterialDefinition
end

setmetatable(MaterialDefinition, classMeta)

return MaterialDefinition
