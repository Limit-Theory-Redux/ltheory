use crate::render::{CubeFace, ResourceId};

/// Which part of a texture a view addresses. S2 only uses the dimensions that
/// can be a render attachment (`D2`, `D2Layer`, `CubeFace`); the sampling
/// dimensions arrive with S3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewDim {
    D2,
    D3,
    Cube,
    /// One z-slice of a 3D texture.
    D2Layer(u16),
    /// One face of a cube texture.
    CubeFace(CubeFace),
}

/// A value-type view of (part of) a texture. No GPU object exists on GL; the
/// executor derives framebuffer attachments from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TexView {
    pub tex: ResourceId,
    pub dim: ViewDim,
    pub base_mip: u8,
    /// 0 = all remaining levels
    pub mip_count: u8,
    /// Width and height of `base_mip`, so a pass can size its viewport
    /// without asking the executor.
    pub extent: [u32; 2],
}

impl TexView {
    pub fn new(tex: ResourceId, dim: ViewDim, base_mip: i32, extent: [i32; 2]) -> Self {
        Self {
            tex,
            dim,
            base_mip: base_mip.clamp(0, 255) as u8,
            mip_count: 1,
            extent: [extent[0].max(1) as u32, extent[1].max(1) as u32],
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TexView {
    pub fn get_width(&self) -> i32 {
        self.extent[0] as i32
    }

    pub fn get_height(&self) -> i32 {
        self.extent[1] as i32
    }

    pub fn get_base_mip(&self) -> i32 {
        self.base_mip as i32
    }
}
