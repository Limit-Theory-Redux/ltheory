//! Group 0: the per-pass `ViewBlock` (doc/engine/render-api-v2.md, section 1.2).

use glam::{Mat4, Vec3, vec3};

use super::{BlockLayout, GlslType};

/// Camera state kept on the `Renderer` by `Renderer:setCamera`. Every pass
/// that begins afterwards snapshots it into its `ViewBlock`.
#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    pub view: Mat4,
    pub proj: Mat4,
    pub star_dir: Vec3,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            star_dir: Vec3::Y,
        }
    }
}

/// Group 0, one per pass (ring-allocated). A superset of the old camera UBO:
/// the first six members are unchanged, so shaders keep their `mView`,
/// `mProj`, `eye`, ... accessors.
///
/// std140: every member is a mat4 or vec4, so the Rust layout is the GLSL
/// layout with no padding.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct ViewBlock {
    pub m_view: [f32; 16],
    pub m_proj: [f32; 16],
    pub m_view_inv: [f32; 16],
    pub m_proj_inv: [f32; 16],
    /// xyz = eye position (camera-relative rendering keeps it at the
    /// origin), w = 1.
    pub eye: [f32; 4],
    /// xyz = direction towards the primary light.
    pub star_dir: [f32; 4],
    /// Orthographic projection for the pass (was ShaderVar `mProjUI`).
    pub m_proj_ui: [f32; 16],
    /// Was ShaderVar `mWorldViewUI`; `pass:setUiTransform` changes it.
    pub m_world_view_ui: [f32; 16],
    /// x, y, w, h in pixels.
    pub viewport: [f32; 4],
    /// Reserved.
    pub time: [f32; 4],
}

impl ViewBlock {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NAME: &'static str = "ViewBlock";

    /// GLSL member name and byte offset of each member, in declaration order
    /// (`res/shader/include/view_block.glsl`).
    pub const MEMBERS: [(&'static str, u32); 10] = [
        ("ubo_mView", std::mem::offset_of!(ViewBlock, m_view) as u32),
        ("ubo_mProj", std::mem::offset_of!(ViewBlock, m_proj) as u32),
        (
            "ubo_mViewInv",
            std::mem::offset_of!(ViewBlock, m_view_inv) as u32,
        ),
        (
            "ubo_mProjInv",
            std::mem::offset_of!(ViewBlock, m_proj_inv) as u32,
        ),
        ("ubo_eye", std::mem::offset_of!(ViewBlock, eye) as u32),
        (
            "ubo_starDir",
            std::mem::offset_of!(ViewBlock, star_dir) as u32,
        ),
        (
            "ubo_mProjUI",
            std::mem::offset_of!(ViewBlock, m_proj_ui) as u32,
        ),
        (
            "ubo_mWorldViewUI",
            std::mem::offset_of!(ViewBlock, m_world_view_ui) as u32,
        ),
        (
            "ubo_viewport",
            std::mem::offset_of!(ViewBlock, viewport) as u32,
        ),
        ("ubo_time", std::mem::offset_of!(ViewBlock, time) as u32),
    ];

    /// Build the block for a pass. `viewport` is `[x, y, w, h]`; `is_window`
    /// selects the y-down UI projection of the backbuffer.
    pub fn new(camera: &CameraState, viewport: [i32; 4], is_window: bool) -> Self {
        let mut block = Self {
            m_view: camera.view.to_cols_array(),
            m_proj: camera.proj.to_cols_array(),
            // Derived rather than passed in: both Lua camera paths agree on
            // the rotation of mViewInv, which is all `worldray.glsl` needs.
            m_view_inv: camera.view.inverse().to_cols_array(),
            m_proj_inv: camera.proj.inverse().to_cols_array(),
            eye: [0.0, 0.0, 0.0, 1.0],
            star_dir: [camera.star_dir.x, camera.star_dir.y, camera.star_dir.z, 0.0],
            m_proj_ui: [0.0; 16],
            m_world_view_ui: Mat4::IDENTITY.to_cols_array(),
            viewport: [0.0; 4],
            time: [0.0; 4],
        };
        block.set_viewport(viewport, is_window);
        block
    }

    /// Update the viewport-dependent members (`m_proj_ui`, `viewport`).
    pub fn set_viewport(&mut self, viewport: [i32; 4], is_window: bool) {
        let [x, y, sx, sy] = viewport;
        self.viewport = [x as f32, y as f32, sx as f32, sy as f32];
        self.m_proj_ui = ui_projection(sx, sy, is_window).to_cols_array();
    }

    #[allow(unsafe_code)]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: repr(C) POD, no padding (all members are multiples of 16
        // bytes and the struct is 16-aligned).
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, Self::SIZE as usize) }
    }

    /// Compare a reflected block against this struct. Used at shader link
    /// (startup assert) and by the layout test.
    pub fn check_layout(block: &BlockLayout) -> Result<(), String> {
        if block.size != Self::SIZE {
            return Err(format!(
                "ViewBlock is {} bytes in the shader but {} in Rust",
                block.size,
                Self::SIZE
            ));
        }
        for (name, offset) in Self::MEMBERS {
            let Some(member) = block.member(name) else {
                return Err(format!("ViewBlock is missing member '{name}'"));
            };
            if member.offset != offset {
                return Err(format!(
                    "ViewBlock member '{name}' is at offset {} in the shader but {offset} in Rust",
                    member.offset
                ));
            }
            let expected = if name.starts_with("ubo_m") {
                GlslType::Mat4
            } else {
                GlslType::Vec4
            };
            if member.ty != expected {
                return Err(format!(
                    "ViewBlock member '{name}' is {:?}, expected {expected:?}",
                    member.ty
                ));
            }
        }
        if block.members.len() != Self::MEMBERS.len() {
            return Err(format!(
                "ViewBlock has {} members in the shader, {} in Rust",
                block.members.len(),
                Self::MEMBERS.len()
            ));
        }
        Ok(())
    }
}

/// The orthographic UI projection of a `sx` x `sy` pass (was
/// `VpStack::push`'s return value). The backbuffer is y-down, textures y-up.
pub fn ui_projection(sx: i32, sy: i32, is_window: bool) -> Mat4 {
    if is_window {
        Mat4::from_translation(vec3(-1.0, 1.0, 0.0))
            * Mat4::from_scale(vec3(2.0f32 / sx as f32, -2.0f32 / sy as f32, 1.0))
    } else {
        Mat4::from_translation(vec3(-1.0, -1.0, 0.0))
            * Mat4::from_scale(vec3(2.0f32 / sx as f32, 2.0f32 / sy as f32, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_matches_std140() {
        // 6 camera members (4 mat4 + 2 vec4) + 2 mat4 + 2 vec4.
        assert_eq!(ViewBlock::SIZE, 448);
        assert_eq!(std::mem::align_of::<ViewBlock>(), 16);
        assert_eq!(ViewBlock::SIZE % 16, 0);
    }

    #[test]
    fn members_are_contiguous() {
        let mut expected = 0;
        for (name, offset) in ViewBlock::MEMBERS {
            assert_eq!(offset, expected, "{name}");
            expected += if name.starts_with("ubo_m") { 64 } else { 16 };
        }
        assert_eq!(expected, ViewBlock::SIZE);
    }

    #[test]
    fn ui_projection_maps_corners() {
        let p = ui_projection(100, 50, false);
        let bl = p * glam::vec4(0.0, 0.0, 0.0, 1.0);
        let tr = p * glam::vec4(100.0, 50.0, 0.0, 1.0);
        assert_eq!((bl.x, bl.y), (-1.0, -1.0));
        assert_eq!((tr.x, tr.y), (1.0, 1.0));
        let w = ui_projection(100, 50, true);
        let tl = w * glam::vec4(0.0, 0.0, 0.0, 1.0);
        assert_eq!((tl.x, tl.y), (-1.0, 1.0));
    }
}
