-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Where a pipeline's vertices come from. The mesh layouts are fixed by
-- `VertexFormat`; the wgpu backend will carry the format here.
---@class VertexLayout
---@field Mesh integer Indexed `Mesh` draws (`pass:drawMesh`).
---@field Fullscreen integer The built-in unit quad of `pass:drawFullscreen`.
---@field Imm2D integer UI vertices of the immediate batcher (`Imm2DVertex`).
---@field Imm3D integer Position, uv and color vertices of the immediate batcher (`Imm3DVertex`): debug geometry and the backdrop box.
VertexLayout = {
    -- Indexed `Mesh` draws (`pass:drawMesh`).
    Mesh = 0,
    -- The built-in unit quad of `pass:drawFullscreen`.
    Fullscreen = 1,
    -- UI vertices of the immediate batcher (`Imm2DVertex`).
    Imm2D = 2,
    -- Position, uv and color vertices of the immediate batcher
    -- (`Imm3DVertex`): debug geometry and the backdrop box.
    Imm3D = 3,
}

