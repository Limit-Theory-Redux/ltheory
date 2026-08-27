/// Per-instance data for instanced rendering
/// Layout: model matrix (64 bytes) + color (16 bytes) + scale (4 bytes) = 84 bytes per instance
/// The Lua-side cdata typedef is declared manually in ffi_ext/Mesh.lua
/// (luajit_ffi_gen only supports impl/enum blocks, not value structs).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceData {
    /// Model matrix (column-major, 4x4)
    pub model_matrix: [f32; 16],
    /// Per-instance color (RGBA)
    pub color: [f32; 4],
    /// Per-instance uniform scale (asteroid size diversity; also used by
    /// the FDM fragment lookup via a flat varying).
    pub scale: f32,
}

impl InstanceData {
    pub fn new(model_matrix: [f32; 16], color: [f32; 4], scale: f32) -> Self {
        Self {
            model_matrix,
            color,
            scale,
        }
    }

    pub fn from_transform_color(transform: &[f32; 16], r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            model_matrix: *transform,
            color: [r, g, b, a],
            scale: 1.0,
        }
    }
}
