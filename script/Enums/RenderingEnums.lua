---@enum MeshType
Enums.MeshType = {
    -- Basic Shapes
    Box = 1,
    Prism = 2,
    Pyramid = 3,
    Icosahedron = 4,
    Ellipsoid = 5,
    Torus = 6,
    IrregularPrism = 7,
    -- Celestial Objects
    Asteroid = 8,
}

--[[
---@enum MaterialType 
Enums.MaterialType = {
    Debug = 1,
    Asteroid = 2,
    -- ...
}


---@enum TextureType
Enums.TextureType = {
    Tex1D = 1,
    Tex2D = 2,
    Tex3D = 3,
    TexCube = 4,
}
--]]

---@enum UniformType
Enums.UniformType = {
    Float = 1,
    Float2 = 2,
    Float3 = 3,
    Float4 = 4,
    Int = 5,
    Int2 = 6,
    Int3 = 7,
    Int4 = 8,
    Matrix = 9,
    MatrixT = 10,
    Tex1D = 11,
    Tex2D = 12,
    Tex3D = 13,
    TexCube = 14
}