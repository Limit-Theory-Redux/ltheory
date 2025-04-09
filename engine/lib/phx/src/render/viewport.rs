use std::cell::RefCell;

use glam::{vec3, IVec2, Mat4};

use crate::render::{gl, glcheck, ShaderVar};

/* TODO : This is a low-level mechanism and probably not for use outside of
 *        RenderTarget. Should likely be folded into RenderTarget. */

pub struct Viewport;

#[luajit_ffi_gen::luajit_ffi]
impl Viewport {
    pub fn get_aspect() -> f32 {
        VP_STACK.with_borrow(|vs| vs.get_aspect())
    }

    #[bind(out_param = true)]
    pub fn get_size() -> IVec2 {
        VP_STACK.with_borrow(|vs| vs.get_size())
    }

    pub fn push(x: i32, y: i32, sx: i32, sy: i32, is_window: bool) {
        VP_STACK.with_borrow_mut(|vs| vs.push(x, y, sx, sy, is_window))
    }

    pub fn pop() {
        VP_STACK.with_borrow_mut(|vs| vs.pop())
    }
}

thread_local! { static VP_STACK: RefCell<VpStack> = RefCell::new(VpStack::new()); }

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VP {
    pub x: i32,
    pub y: i32,
    pub sx: i32,
    pub sy: i32,
    pub is_window: bool,
}

const VP_STACK_DEPTH: usize = 16;

struct VpStack {
    stack: [VP; VP_STACK_DEPTH],
    stack_size: usize,
}

impl VpStack {
    fn new() -> Self {
        Self {
            stack: [VP {
                x: 0,
                y: 0,
                sx: 0,
                sy: 0,
                is_window: false,
            }; VP_STACK_DEPTH],
            stack_size: 0,
        }
    }

    pub fn get_aspect(&self) -> f32 {
        if self.stack_size == 0 {
            panic!("Viewport_GetAspect: Viewport stack is empty");
        }
        let vp = &self.stack[self.stack_size - 1];
        vp.sx as f32 / vp.sy as f32
    }

    pub fn get_size(&self) -> IVec2 {
        if self.stack_size == 0 {
            panic!("Viewport_GetSize: Viewport stack is empty");
        }
        let vp = &self.stack[self.stack_size - 1];
        IVec2::new(vp.sx, vp.sy)
    }

    pub fn push(&mut self, x: i32, y: i32, sx: i32, sy: i32, is_window: bool) {
        if self.stack_size >= VP_STACK_DEPTH {
            panic!("Viewport_Push: Maximum viewport stack depth exceeded");
        }

        let vp = &mut self.stack[self.stack_size];
        vp.x = x;
        vp.y = y;
        vp.sx = sx;
        vp.sy = sy;
        vp.is_window = is_window;

        self.stack_size += 1;

        // Set up the ortho projection matrix for UI elements.
        let ortho_proj = if vp.is_window {
            Mat4::from_translation(vec3(-1.0, 1.0, 0.0))
                * Mat4::from_scale(vec3(2.0f32 / vp.sx as f32, -2.0f32 / vp.sy as f32, 1.0))
        } else {
            Mat4::from_translation(vec3(-1.0, -1.0, 0.0))
                * Mat4::from_scale(vec3(2.0f32 / vp.sx as f32, 2.0f32 / vp.sy as f32, 1.0))
        };
        ShaderVar::push_matrix("mProjUI", &ortho_proj.into());
        ShaderVar::push_matrix("mWorldViewUI", &Mat4::IDENTITY.into());

        glcheck!(gl::Viewport(vp.x, vp.y, vp.sx, vp.sy));
    }

    pub fn pop(&mut self) {
        if self.stack_size == 0 {
            panic!("Viewport_Pop: Viewport stack is empty");
        }

        ShaderVar::pop("mWorldViewUI");
        ShaderVar::pop("mProjUI");

        self.stack_size -= 1;
        if self.stack_size > 0 {
            let vp = &self.stack[self.stack_size - 1];
            glcheck!(gl::Viewport(vp.x, vp.y, vp.sx, vp.sy));
        }
    }
}
