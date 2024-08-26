local UniformFunc = require('_ECS_WIP_TEMP.Shared.Types.UniformFunc')

---@type table<UniformType, UniformFunc>
local UniformFuncs = {}

insert(UniformFuncs, Enums.UniformType.Float, UniformFunc {
    type = Enums.UniformType.Float,
    func = Shader.iSetFloat,
    paramLen = 1,
    paramTypes = {float}
})
insert(UniformFuncs, Enums.UniformType.Float2, UniformFunc {
    type = Enums.UniformType.Float2,
    func = Shader.iSetFloat2,
    paramLen = 2,
    paramTypes = {float, float}
})
insert(UniformFuncs, Enums.UniformType.Float3, UniformFunc {
    type = Enums.UniformType.Float3,
    func = Shader.iSetFloat3,
    paramLen = 3,
    paramTypes = {float, float, float}
})
insert(UniformFuncs, Enums.UniformType.Float4, UniformFunc {
    type = Enums.UniformType.Float4,
    func = Shader.iSetFloat4,
    paramLen = 4,
    paramTypes = {float, float, float, float}
})
insert(UniformFuncs, Enums.UniformType.Int, UniformFunc {
    type = Enums.UniformType.Int,
    func = Shader.iSetInt,
    paramLen = 1,
    paramTypes = {int}
})
insert(UniformFuncs, Enums.UniformType.Int2, UniformFunc {
    type = Enums.UniformType.Int2,
    func = Shader.iSetInt2,
    paramLen = 2,
    paramTypes = {int, int}
})
insert(UniformFuncs, Enums.UniformType.Int3, UniformFunc {
    type = Enums.UniformType.Int3,
    func = Shader.iSetInt3,
    paramLen = 3,
    paramTypes = {int, int, int}
})
insert(UniformFuncs, Enums.UniformType.Int4, UniformFunc {
    type = Enums.UniformType.Int4,
    func = Shader.iSetInt4,
    paramLen = 4,
    paramTypes = {int, int, int, int}
})
insert(UniformFuncs, Enums.UniformType.Matrix, UniformFunc {
    type = Enums.UniformType.Matrix,
    func = Shader.iSetMatrix,
    paramLen = 1,
    paramTypes = {Matrix}
})
insert(UniformFuncs, Enums.UniformType.MatrixT, UniformFunc {
    type = Enums.UniformType.MatrixT,
    func = Shader.iSetMatrixT,
    paramLen = 1,
    paramTypes = {Matrix}
})
insert(UniformFuncs, Enums.UniformType.Tex1D, UniformFunc {
    type = Enums.UniformType.Tex1D,
    func = Shader.iSetTex1D,
    paramLen = 1,
    paramTypes = {Tex1D}
})
insert(UniformFuncs, Enums.UniformType.Tex2D, UniformFunc {
    type = Enums.UniformType.Tex2D,
    func = Shader.iSetTex2D,
    paramLen = 1,
    paramTypes = {Tex2D}
})
insert(UniformFuncs, Enums.UniformType.Tex3D, UniformFunc {
    type = Enums.UniformType.Tex3D,
    func = Shader.iSetTex3D,
    paramLen = 1,
    paramTypes = {Tex3D}
})
insert(UniformFuncs, Enums.UniformType.TexCube, UniformFunc {
    type = Enums.UniformType.TexCube,
    func = Shader.iSetTexCube,
    paramLen = 1,
    paramTypes = {TexCube}
})

return UniformFuncs