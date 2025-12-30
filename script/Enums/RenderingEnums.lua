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

---@enum BufferName
Enums.BufferName = {
    -- Primary Buffers
    buffer0 = 0,   -- Src/Albedo Buffer
    buffer1 = 1,   -- G-Buffer: Normal/Material (xy=normal, z=roughness, w=material)
    buffer2 = 2,   -- Scratch/Composite Buffer
    -- Depth Buffers
    zBuffer = 3,   -- Primary Depth Buffer
    zBufferL = 4,  -- Linear Depth Buffer (for deferred)
    -- Downsample Buffers
    dsBuffer0 = 5, -- Primary Downsample Buffer
    dsBuffer1 = 6, -- Secondary Downsample Buffer
    -- Deferred Lighting
    lightAccum = 7 -- Light Accumulation Buffer
}

---@enum RenderingPassName
Enums.RenderingPasses = {
    Skybox = 0,
    Opaque = 1,
    Lighting = 2,
    Additive = 3,
    Alpha = 4,
    Debug = 5,
    UI = 6,
}

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
