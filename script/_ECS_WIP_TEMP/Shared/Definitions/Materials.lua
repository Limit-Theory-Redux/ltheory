-- Types --
local MaterialDefinition = require("_ECS_WIP_TEMP.Shared.Types.MaterialDefinition")

-- Definitions --
local ShaderVarFuncs = require("_ECS_WIP_TEMP.Shared.Definitions.ShaderVars")

MaterialDefinition{
    name = "Asteroid",
    vs_name = "wvp",
    fs_name = "material/asteroid",
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

MaterialDefinition{
    name = "Metal",
    vs_name = "wvp",
    fs_name = "material/metal",
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