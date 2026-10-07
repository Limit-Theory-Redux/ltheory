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
    buffer0 = 0,   -- Src Buffer
    buffer1 = 1,   -- Alpha/Normals/UI Buffer
    buffer2 = 2,   -- Lighting Buffer
    -- Depth Buffers
    zBuffer = 3,   -- Primary Depth Buffer
    zBufferL = 4,  -- Lighting Depth Buffer
    -- Downsample Buffers
    dsBuffer0 = 5, -- Primary Downsample Buffer
    dsBuffer1 = 6  -- Secondary Downsample Buffer
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
