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
---@field vertex string
---@field fragment string
---@field blendMode BlendMode
---@field textures table<Texture>
---@field autoShaderVars table<AutoShaderVar>
---@field constShaderVars table<ConstShaderVar>

---@class MaterialDefinitionConstructor
---@field vertex string
---@field fragment string
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
    if not args.vertex or not args.fragment or not args.blendMode then
        return nil
    end
    
    -- Not required arguments
    if not args.textures then args.textures = {} end
    if not args.autoShaderVars then args.autoShaderVars = {} end
    if not args.constShaderVars then args.constShaderVars = {} end

    local newMaterialDefinition = setmetatable({
        vertex = args.vertex,
        fragment = args.fragment,
        blendMode = args.blendMode,
        textures = args.textures,
        autoShaderVars = args.autoShaderVars,
        constShaderVars = args.constShaderVars
    }, sharedMeta)

    return newMaterialDefinition
end

setmetatable(MaterialDefinition, classMeta)

return MaterialDefinition
