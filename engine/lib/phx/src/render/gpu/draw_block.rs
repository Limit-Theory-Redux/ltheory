//! Group 2: the fixed per-draw block of scene meshes
//! (doc/engine/render-api-v2.md, section 1.2). One `DrawBlock` per drawn
//! mesh is written straight into the uniform ring by `SceneList::submit`.

use super::{BlockLayout, GlslType, UNIFORM_ALIGN};

/// Number of material-defined `vec4` slots of a draw block.
pub const DRAW_USER_VECS: usize = 7;
/// `f32`s in the material-defined part of a draw block.
pub const DRAW_USER_FLOATS: usize = DRAW_USER_VECS * 4;

/// std140: two `mat4`, a `vec4` and `vec4[7]`, so the Rust layout is the GLSL
/// layout with no padding, and exactly one `UNIFORM_ALIGN` stride.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct DrawBlock {
    /// Local to camera-relative world (includes the body's scale).
    pub m_world: [f32; 16],
    /// Inverse transpose of `m_world`, for normals.
    pub m_world_it: [f32; 16],
    /// x = the body's uniform scale (was the `scale` auto-var).
    pub draw_scale: [f32; 4],
    /// Material-defined per-draw values (`MaterialType.perDraw`).
    pub draw_user: [[f32; 4]; DRAW_USER_VECS],
}

impl DrawBlock {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NAME: &'static str = "DrawBlock";

    /// GLSL member name, byte offset and type of each member, in declaration
    /// order (`res/shader/include/draw_block.glsl`).
    pub const MEMBERS: [(&'static str, u32, GlslType); 4] = [
        (
            "mWorld",
            std::mem::offset_of!(DrawBlock, m_world) as u32,
            GlslType::Mat4,
        ),
        (
            "mWorldIT",
            std::mem::offset_of!(DrawBlock, m_world_it) as u32,
            GlslType::Mat4,
        ),
        (
            "drawScale",
            std::mem::offset_of!(DrawBlock, draw_scale) as u32,
            GlslType::Vec4,
        ),
        (
            "drawUser",
            std::mem::offset_of!(DrawBlock, draw_user) as u32,
            GlslType::Vec4,
        ),
    ];

    pub fn new(m_world: &[f32; 16], m_world_it: &[f32; 16], scale: f32, user: &[f32; 28]) -> Self {
        let mut draw_user = [[0.0; 4]; DRAW_USER_VECS];
        for (i, v) in draw_user.iter_mut().enumerate() {
            v.copy_from_slice(&user[i * 4..i * 4 + 4]);
        }
        Self {
            m_world: *m_world,
            m_world_it: *m_world_it,
            draw_scale: [scale, 0.0, 0.0, 0.0],
            draw_user,
        }
    }

    #[allow(unsafe_code)]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: repr(C) POD without padding (every member is a multiple of
        // 16 bytes and the struct is 16-aligned).
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, Self::SIZE as usize) }
    }

    /// Compare a reflected block against this struct. Used at shader link and
    /// by the layout test.
    pub fn check_layout(block: &BlockLayout) -> Result<(), String> {
        if block.size != Self::SIZE {
            return Err(format!(
                "DrawBlock is {} bytes in the shader but {} in Rust",
                block.size,
                Self::SIZE
            ));
        }
        for (name, offset, ty) in Self::MEMBERS {
            let Some(member) = block.member(name) else {
                return Err(format!("DrawBlock is missing member '{name}'"));
            };
            if member.offset != offset {
                return Err(format!(
                    "DrawBlock member '{name}' is at offset {} in the shader but {offset} in Rust",
                    member.offset
                ));
            }
            if member.ty != ty {
                return Err(format!(
                    "DrawBlock member '{name}' is {:?}, expected {ty:?}",
                    member.ty
                ));
            }
        }
        let user = block.member("drawUser").expect("checked above");
        if user.count != DRAW_USER_VECS as u32 {
            return Err(format!(
                "DrawBlock.drawUser has {} elements in the shader, {DRAW_USER_VECS} in Rust",
                user.count
            ));
        }
        if block.members.len() != Self::MEMBERS.len() {
            return Err(format!(
                "DrawBlock has {} members in the shader, {} in Rust",
                block.members.len(),
                Self::MEMBERS.len()
            ));
        }
        Ok(())
    }
}

const _: () = assert!(DrawBlock::SIZE == UNIFORM_ALIGN);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_is_one_ring_stride() {
        assert_eq!(DrawBlock::SIZE, 256);
        assert_eq!(std::mem::align_of::<DrawBlock>(), 16);
    }

    #[test]
    fn members_are_contiguous() {
        let mut expected = 0;
        for (name, offset, ty) in DrawBlock::MEMBERS {
            assert_eq!(offset, expected, "{name}");
            expected += match name {
                "drawUser" => 16 * DRAW_USER_VECS as u32,
                _ => ty.packed_size(),
            };
        }
        assert_eq!(expected, DrawBlock::SIZE);
    }

    #[test]
    fn user_values_land_in_their_slots() {
        let mut user = [0.0f32; DRAW_USER_FLOATS];
        user[0] = 1.0;
        user[5] = 2.0;
        user[27] = 3.0;
        let block = DrawBlock::new(&[0.0; 16], &[0.0; 16], 4.0, &user);
        assert_eq!(block.draw_scale[0], 4.0);
        assert_eq!(block.draw_user[0][0], 1.0);
        assert_eq!(block.draw_user[1][1], 2.0);
        assert_eq!(block.draw_user[6][3], 3.0);
    }
}
