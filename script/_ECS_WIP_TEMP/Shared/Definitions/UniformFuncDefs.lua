-- Types --
local UniformFunc = require('_ECS_WIP_TEMP.Shared.Types.UniformFunc')

UniformFunc {
    funcType = Enums.UniformType.Float,
    func = function(shader, uniformInt, ...) shader:iSetFloat(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Float2,
    func = function(shader, uniformInt, ...) shader:iSetFloat2(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Float3,
    func = function(shader, uniformInt, ...) shader:iSetFloat3(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Float4,
    func = function(shader, uniformInt, ...) shader:iSetFloat4(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Int,
    func = function(shader, uniformInt, ...) shader:iSetInt(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Int2,
    func = function(shader, uniformInt, ...) shader:iSetInt2(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Int3,
    func = function(shader, uniformInt, ...) shader:iSetInt3(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Int4,
    func = function(shader, uniformInt, ...) shader:iSetInt4(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Matrix,
    func = function(shader, uniformInt, ...) shader:iSetMatrix(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.MatrixT,
    func = function(shader, uniformInt, ...) shader:iSetMatrixT(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Tex1D,
    func = function(shader, uniformInt, ...) shader:iSetTex1D(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Tex2D,
    func = function(shader, uniformInt, ...) shader:iSetTex2D(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.Tex3D,
    func = function(shader, uniformInt, ...) shader:iSetTex3D(uniformInt, ...) end,
}
UniformFunc {
    funcType = Enums.UniformType.TexCube,
    func = function(shader, uniformInt, ...) shader:iSetTexCube(uniformInt, ...) end,
}
