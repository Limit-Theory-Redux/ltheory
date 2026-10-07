-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Where a pipeline's vertices come from. The mesh layouts are fixed by
-- `VertexFormat`; the wgpu backend will carry the format here.
---@class VertexLayout
---@field Mesh integer Indexed `Mesh` draws (`pass:drawMesh`).
---@field Fullscreen integer The built-in unit quad of `pass:drawFullscreen`.
VertexLayout = {
    -- Indexed `Mesh` draws (`pass:drawMesh`).
    Mesh = 0,
    -- The built-in unit quad of `pass:drawFullscreen`.
    Fullscreen = 1,
}

