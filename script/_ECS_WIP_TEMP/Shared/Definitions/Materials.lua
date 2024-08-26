-- Types --
local MaterialDefinition = require("_ECS_WIP_TEMP.Shared.Types.MaterialDefinition")
local Material = require("_ECS_WIP_TEMP.Shared.Rendering.Material")
local Texture = require("_ECS_WIP_TEMP.Shared.Rendering.Texture")
local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar")
local ConstShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.ConstShaderVar")

-- Definitions --
local ShaderVarFuncs = require("_ECS_WIP_TEMP.Shared.Definitions.ShaderVars")


local Materials = {}

local materialDefinitions = {
    [BlendMode.Disabled] = {},
    [BlendMode.Additive] = {},
    [BlendMode.Alpha] = {},
    [BlendMode.PreMultAlpha] = {}
}

---@param blendMode integer
---@param materialName string
function Materials.getMaterial(blendMode, materialName)
    return Material(materialDefinitions[blendMode][materialName])
end

materialDefinitions[BlendMode.Disabled].Asteroid = MaterialDefinition{
    vertex = "wvp",
    fragment = "material/asteroid",
    blendMode = BlendMode.Disabled,
    textures = {
        { texName = "texDiffuse", tex = Cache.Texture('rock'), texType = Enums.UniformType.Tex2D, texSettings = nil }
    },
    autoShaderVars = {
        { uniformName = "mWorld", uniformType = Enums.UniformType.Matrix, callbackFn = ShaderVarFuncs.mWorldFunc},
        { uniformName = "mWorldIT", uniformType = Enums.UniformType.MatrixT, callbackFn = ShaderVarFuncs.mWorldITFun},
        { uniformName = "scale", uniformType = Enums.UniformType.Float, callbackFn = ShaderVarFuncs.scaleFunc}
    }
}

materialDefinitions[BlendMode.Disabled].Metal = MaterialDefinition{
    vertex = "wvp",
    fragment = "material/metal",
    blendMode = BlendMode.Disabled,
    textures = {
        { texName = "texDiffuse", tex = Cache.Texture('metal/01_d'), texType = Enums.UniformType.Tex2D, texSettings = nil },
        { texName = "texNormal", tex = Cache.Texture('metal/01_n'), texType = Enums.UniformType.Tex2D, texSettings = nil },
        { texName = "texSpec", tex = Cache.Texture('metal/01_s'), texType = Enums.UniformType.Tex2D, texSettings = nil }
    },
    autoShaderVars = {
        { uniformName = "mWorld", uniformType = Enums.UniformType.Matrix, callbackFn = ShaderVarFuncs.mWorldFunc},
        { uniformName = "mWorldIT", uniformType = Enums.UniformType.MatrixT, callbackFn = ShaderVarFuncs.mWorldITFun},
        { uniformName = "scale", uniformType = Enums.UniformType.Float, callbackFn = ShaderVarFuncs.scaleFunc}
    }
}

return Materials()