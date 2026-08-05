use super::{CubeFace, Tex2D, Tex3D, TexCube, TexFormat, gl};
use crate::render::{RenderCommand, Renderer, Viewport};
use crate::system::{Metric, Profiler};

pub struct RenderTarget;

#[luajit_ffi_gen::luajit_ffi]
impl RenderTarget {
    pub fn push(r: &mut Renderer, sx: i32, sy: i32) {
        Profiler::begin("RenderTarget_Push");

        r.data.render_target.push();
        Metric::FBOSwap.inc();
        r.submit(RenderCommand::PushFramebuffer {
            id: 0,
            width: sx,
            height: sy,
        });
        Viewport::push(r, 0, 0, sx, sy, false);

        Profiler::end();
    }

    pub fn pop(r: &mut Renderer) {
        Profiler::begin("RenderTarget_Pop");

        r.data.render_target.pop();
        Metric::FBOSwap.inc();
        r.submit(RenderCommand::PopFramebuffer);
        Viewport::pop(r);

        Profiler::end();
    }

    pub fn bind_tex2d(r: &mut Renderer, tex: &Tex2D) {
        Self::bind_tex2d_level(r, tex, 0);
    }

    pub fn bind_tex2d_level(r: &mut Renderer, tex: &Tex2D, level: i32) {
        let attachment = r
            .data
            .render_target
            .attach(TexFormat::is_color(Tex2D::get_format(tex)), "BindTex2D");
        r.submit(RenderCommand::FramebufferAttachTexture2DByResource {
            attachment,
            id: tex.resource_id(),
            level,
        });
    }

    pub fn bind_tex3d(r: &mut Renderer, tex: &Tex3D, layer: i32) {
        Self::bind_tex3d_level(r, tex, layer, 0);
    }

    pub fn bind_tex3d_level(r: &mut Renderer, tex: &Tex3D, layer: i32, level: i32) {
        let attachment = r.data.render_target.attach_color("BindTex3D");
        r.submit(RenderCommand::FramebufferAttachTexture3DByResource {
            attachment,
            id: tex.resource_id(),
            layer,
            level,
        });
    }

    pub fn bind_tex_cube(r: &mut Renderer, tex: &TexCube, face: CubeFace) {
        Self::bind_tex_cube_level(r, tex, face, 0);
    }

    pub fn bind_tex_cube_level(r: &mut Renderer, tex: &TexCube, face: CubeFace, level: i32) {
        let attachment = r.data.render_target.attach_color("BindTexCubeLevel");
        r.submit(RenderCommand::FramebufferAttachTextureCubeByResource {
            attachment,
            id: tex.resource_id(),
            face: face as u32,
            level,
        });
    }

    pub fn push_tex2d(r: &mut Renderer, tex: &Tex2D) {
        Self::push_tex2d_level(r, tex, 0);
    }

    pub fn push_tex2d_level(r: &mut Renderer, tex: &Tex2D, level: i32) {
        let size = tex.get_size_level(level);
        Self::push(r, size.x, size.y);
        Self::bind_tex2d_level(r, tex, level);
    }

    pub fn push_tex3d(r: &mut Renderer, tex: &Tex3D, layer: i32) {
        Self::push_tex3d_level(r, tex, layer, 0);
    }

    pub fn push_tex3d_level(r: &mut Renderer, tex: &Tex3D, layer: i32, level: i32) {
        let size = tex.get_size_level(level);
        Self::push(r, size.x, size.y);
        Self::bind_tex3d_level(r, tex, layer, level);
    }
}

const FBO_STACK_DEPTH: usize = 16;

#[derive(Copy, Clone, Default)]
struct FboAttachState {
    color_index: i32,
    depth: bool,
}

/// Tracks color/depth attachment counts for the currently pushed framebuffers
/// (was part of `thread_local! FBO_STACK`'s `FBO`/`FboStack` in this file).
///
/// This is CPU-only bookkeeping: it decides *which* attachment point
/// (`GL_COLOR_ATTACHMENTn`/`GL_DEPTH_ATTACHMENT`) a bind should use and
/// enforces the same limits the old code did. The actual GL framebuffer
/// object - handle, creation, deletion - now lives entirely in
/// `CommandExecutor`'s own `fbo_stack`, driven by `PushFramebuffer`/
/// `PopFramebuffer`; nothing here needs to mirror that.
pub struct RenderTargetStack {
    stack: [FboAttachState; FBO_STACK_DEPTH],
    stack_size: usize,
}

impl RenderTargetStack {
    pub fn new() -> Self {
        Self {
            stack: [FboAttachState::default(); FBO_STACK_DEPTH],
            stack_size: 0,
        }
    }

    fn push(&mut self) {
        if self.stack_size >= FBO_STACK_DEPTH {
            panic!("RenderTarget_Push: Maximum stack depth {FBO_STACK_DEPTH} exceeded");
        }
        self.stack[self.stack_size] = FboAttachState::default();
        self.stack_size += 1;
    }

    fn pop(&mut self) {
        if self.stack_size == 0 {
            panic!("RenderTarget_Pop: Attempting to pop an empty stack");
        }
        self.stack_size -= 1;
    }

    /// Claim the next color attachment point, or the depth attachment point
    /// if `is_color` is false. `label` is only used to name the panic if the
    /// limit for that kind of attachment is already reached.
    fn attach(&mut self, is_color: bool, label: &str) -> u32 {
        if is_color {
            self.attach_color(label)
        } else {
            let state = &mut self.stack[self.stack_size - 1];
            if state.depth {
                panic!("RenderTarget_{label}: Target already has a depth buffer");
            }
            state.depth = true;
            gl::DEPTH_ATTACHMENT
        }
    }

    fn attach_color(&mut self, label: &str) -> u32 {
        let state = &mut self.stack[self.stack_size - 1];
        if state.color_index >= 4 {
            panic!("RenderTarget_{label}: Max color attachments exceeded");
        }
        let attachment = gl::COLOR_ATTACHMENT0 + state.color_index as u32;
        state.color_index += 1;
        attachment
    }
}

impl Default for RenderTargetStack {
    fn default() -> Self {
        Self::new()
    }
}
