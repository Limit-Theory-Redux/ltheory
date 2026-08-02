use glam::{IVec2, Mat4, vec3};

use crate::render::{RenderCommand, Renderer, ShaderVar};

/* TODO : This is a low-level mechanism and probably not for use outside of
 *        RenderTarget. Should likely be folded into RenderTarget. */

pub struct Viewport;

#[luajit_ffi_gen::luajit_ffi]
impl Viewport {
    pub fn get_aspect(r: &Renderer) -> f32 {
        r.viewport.get_aspect()
    }

    #[bind(out_param = true)]
    pub fn get_size(r: &Renderer) -> IVec2 {
        r.viewport.get_size()
    }

    pub fn push(r: &mut Renderer, x: i32, y: i32, sx: i32, sy: i32, is_window: bool) {
        let ortho_proj = r.viewport.push(x, y, sx, sy, is_window);

        ShaderVar::push_matrix("mProjUI", &ortho_proj.into());
        ShaderVar::push_matrix("mWorldViewUI", &Mat4::IDENTITY.into());

        r.submit(RenderCommand::SetViewport {
            x,
            y,
            width: sx,
            height: sy,
        });
    }

    pub fn pop(r: &mut Renderer) {
        ShaderVar::pop("mWorldViewUI");
        ShaderVar::pop("mProjUI");

        if let Some(vp) = r.viewport.pop() {
            r.submit(RenderCommand::SetViewport {
                x: vp.x,
                y: vp.y,
                width: vp.sx,
                height: vp.sy,
            });
        }
    }
}

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

/// Viewport stack, owned by `Renderer` (was `thread_local! VP_STACK`) - GPU
/// viewport state has to live with whatever owns the GL context.
pub struct VpStack {
    stack: [VP; VP_STACK_DEPTH],
    stack_size: usize,
}

impl VpStack {
    pub fn new() -> Self {
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

    /// Push a viewport onto the stack and return the UI ortho-projection
    /// matrix for it. Pure CPU bookkeeping - the caller submits the matching
    /// `SetViewport` command and pushes the matrix onto `ShaderVar`.
    pub fn push(&mut self, x: i32, y: i32, sx: i32, sy: i32, is_window: bool) -> Mat4 {
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
        if vp.is_window {
            Mat4::from_translation(vec3(-1.0, 1.0, 0.0))
                * Mat4::from_scale(vec3(2.0f32 / vp.sx as f32, -2.0f32 / vp.sy as f32, 1.0))
        } else {
            Mat4::from_translation(vec3(-1.0, -1.0, 0.0))
                * Mat4::from_scale(vec3(2.0f32 / vp.sx as f32, 2.0f32 / vp.sy as f32, 1.0))
        }
    }

    /// Pop a viewport off the stack. Returns the viewport to restore as the
    /// active GL viewport, or `None` if the stack is now empty.
    pub fn pop(&mut self) -> Option<VP> {
        if self.stack_size == 0 {
            panic!("Viewport_Pop: Viewport stack is empty");
        }

        self.stack_size -= 1;
        (self.stack_size > 0).then(|| self.stack[self.stack_size - 1])
    }
}

impl Default for VpStack {
    fn default() -> Self {
        Self::new()
    }
}
