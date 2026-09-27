const DEFAULT_STRIDE_SIZE: u32 = 32; // 3 floats pos + 3 floats normal + 2 floats uv = 32 bytes

/// Vertex format description for mesh creation
#[derive(Debug, Clone)]
pub struct VertexFormat {
    pub has_position: bool,
    pub has_normal: bool,
    pub has_uv: bool,
    pub has_color: bool,
    pub stride: u32,
}

impl Default for VertexFormat {
    fn default() -> Self {
        Self {
            has_position: true,
            has_normal: true,
            has_uv: true,
            has_color: false,
            stride: DEFAULT_STRIDE_SIZE,
        }
    }
}
