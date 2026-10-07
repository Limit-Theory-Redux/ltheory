use crate::render::{CubeFace, ResourceId};

/// Which part of a texture a view addresses. `D2Layer` and `CubeFace` are
/// attachment-only on GL 3.3 (sampling binds the whole texture).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewDim {
    D1,
    D2,
    D3,
    Cube,
    /// One z-slice of a 3D texture.
    D2Layer(u16),
    /// One face of a cube texture.
    CubeFace(CubeFace),
}

/// A value-type view of (part of) a texture. No GPU object exists on GL; the
/// executor derives framebuffer attachments (S2) and sampling mip ranges (S3)
/// from it.
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
    /// A one-level view (attachments, `mipView`).
    pub fn new(tex: ResourceId, dim: ViewDim, base_mip: i32, extent: [i32; 2]) -> Self {
        Self {
            tex,
            dim,
            base_mip: base_mip.clamp(0, 255) as u8,
            mip_count: 1,
            extent: [extent[0].max(1) as u32, extent[1].max(1) as u32],
        }
    }

    /// A view of every mip level (what `tex:view()` returns, for sampling).
    pub fn full(tex: ResourceId, dim: ViewDim, extent: [i32; 2]) -> Self {
        Self {
            mip_count: 0,
            ..Self::new(tex, dim, 0, extent)
        }
    }

    /// The `[base, max]` GL mip range this view samples (`max` is the
    /// driver default, 1000, for "all remaining levels").
    pub fn gl_mip_range(&self) -> (i32, i32) {
        let base = self.base_mip as i32;
        let max = if self.mip_count == 0 {
            1000
        } else {
            base + self.mip_count as i32 - 1
        };
        (base, max)
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

    /// 0 = all remaining levels.
    pub fn get_mip_count(&self) -> i32 {
        self.mip_count as i32
    }

    /// The same texture restricted to `mipCount` levels from `baseMip`
    /// (`mipCount` 0 = all remaining), for sampling. The extent follows the
    /// base level.
    pub fn mips(&self, base_mip: i32, mip_count: i32) -> TexView {
        let shift = (base_mip - self.base_mip as i32).clamp(0, 31) as u32;
        TexView {
            base_mip: base_mip.clamp(0, 255) as u8,
            mip_count: mip_count.clamp(0, 255) as u8,
            extent: [
                (self.extent[0] >> shift).max(1),
                (self.extent[1] >> shift).max(1),
            ],
            ..*self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mip_ranges() {
        let v = TexView::full(ResourceId(1), ViewDim::D2, [64, 32]);
        assert_eq!(v.gl_mip_range(), (0, 1000));
        let m = v.mips(2, 3);
        assert_eq!(m.gl_mip_range(), (2, 4));
        assert_eq!(m.extent, [16, 8]);
        assert_eq!(
            TexView::new(ResourceId(1), ViewDim::D2, 1, [8, 8]).gl_mip_range(),
            (1, 1)
        );
    }
}
