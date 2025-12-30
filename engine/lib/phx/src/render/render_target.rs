use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::warn;

use super::{CubeFace, Tex2D, Tex3D, TexCube, TexFormat, gl};
use super::{GpuHandle, RenderCommand, is_command_mode, submit_command};
use crate::render::{Viewport, glcheck};
use crate::system::{Metric, Profiler};

/// Counter for generating unique FBO IDs in command mode
static NEXT_FBO_ID: AtomicU64 = AtomicU64::new(1);

pub struct RenderTarget;

#[luajit_ffi_gen::luajit_ffi]
impl RenderTarget {
    pub fn push(sx: i32, sy: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.push(sx, sy));
    }

    pub fn pop() {
        FBO_STACK.with_borrow_mut(|fs| fs.pop());
    }

    pub fn bind_tex2d(tex: &Tex2D) {
        FBO_STACK.with_borrow_mut(|fs| fs.bind_tex2d(tex));
    }

    pub fn bind_tex2d_level(tex: &Tex2D, level: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.bind_tex2d_level(tex, level));
    }

    pub fn bind_tex3d(tex: &Tex3D, layer: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.bind_tex3d(tex, layer));
    }

    pub fn bind_tex3d_level(tex: &Tex3D, layer: i32, level: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.bind_tex3d_level(tex, layer, level));
    }

    pub fn bind_tex_cube(tex: &TexCube, face: CubeFace) {
        FBO_STACK.with_borrow_mut(|fs| fs.bind_tex_cube(tex, face));
    }

    pub fn bind_tex_cube_level(tex: &TexCube, face: CubeFace, level: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.bind_tex_cube_level(tex, face, level));
    }

    pub fn push_tex2d(tex: &Tex2D) {
        FBO_STACK.with_borrow_mut(|fs| fs.push_tex2d(tex));
    }

    pub fn push_tex2d_level(tex: &Tex2D, level: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.push_tex2d_level(tex, level));
    }

    pub fn push_tex3d(tex: &Tex3D, layer: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.push_tex3d(tex, layer));
    }

    pub fn push_tex3d_level(tex: &Tex3D, layer: i32, level: i32) {
        FBO_STACK.with_borrow_mut(|fs| fs.push_tex3d_level(tex, layer, level));
    }
}

thread_local! { static FBO_STACK: RefCell<FboStack> = RefCell::new(FboStack::new()); }

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FBO {
    pub handle: u32,
    pub color_index: i32,
    pub sx: i32,
    pub sy: i32,
    pub depth: bool,
}

const FBO_STACK_DEPTH: usize = 16;
const BUFS_COUNT: usize = 4;

struct FboStack {
    stack: [FBO; FBO_STACK_DEPTH],
    stack_size: usize,
    bufs: [gl::types::GLenum; BUFS_COUNT],
}

impl FboStack {
    fn new() -> Self {
        Self {
            stack: [FBO {
                handle: 0,
                color_index: 0,
                sx: 0,
                sy: 0,
                depth: false,
            }; FBO_STACK_DEPTH],
            stack_size: 0,
            bufs: [
                gl::COLOR_ATTACHMENT0 as _,
                gl::COLOR_ATTACHMENT1 as _,
                gl::COLOR_ATTACHMENT2 as _,
                gl::COLOR_ATTACHMENT3 as _,
            ],
        }
    }

    fn get_active(&mut self) -> &mut FBO {
        &mut self.stack[self.stack_size - 1]
    }

    fn push(&mut self, sx: i32, sy: i32) {
        Profiler::begin("RenderTarget_Push");

        if self.stack_size >= FBO_STACK_DEPTH {
            panic!("RenderTarget_Push: Maximum stack depth {FBO_STACK_DEPTH} exceeded");
        }

        self.stack_size += 1;

        let fbo = self.get_active();
        fbo.color_index = 0;
        fbo.sx = sx;
        fbo.sy = sy;
        fbo.depth = false;

        Metric::FBOSwap.inc();

        if is_command_mode() {
            // In command mode, the render thread manages the actual FBO
            // We use an ID to track it locally for state management
            let id = NEXT_FBO_ID.fetch_add(1, Ordering::Relaxed);
            fbo.handle = id as u32; // Store ID locally for tracking
            submit_command(RenderCommand::PushFramebuffer {
                id,
                width: sx,
                height: sy,
            });
        } else {
            fbo.handle = 0;
            glcheck!(gl::GenFramebuffers(1, &mut fbo.handle));
            glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo.handle));
        }

        Viewport::push(0, 0, sx, sy, false);
        Profiler::end();
    }

    fn pop(&mut self) {
        Profiler::begin("RenderTarget_Pop");

        if self.stack_size == 0 {
            panic!("RenderTarget_Pop: Attempting to pop an empty stack");
        }

        Metric::FBOSwap.inc();

        if is_command_mode() {
            // In command mode, the render thread handles all the cleanup
            submit_command(RenderCommand::PopFramebuffer);
        } else {
            let mut i = 0;
            while i < BUFS_COUNT {
                glcheck!(gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + i as u32,
                    gl::TEXTURE_2D,
                    0,
                    0,
                ));
                i += 1;
            }

            glcheck!(gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                0,
                0
            ));

            let fbo = &self.stack[self.stack_size - 1];
            glcheck!(gl::DeleteFramebuffers(1, &fbo.handle,));

            if self.stack_size > 1 {
                glcheck!(gl::BindFramebuffer(
                    gl::FRAMEBUFFER,
                    self.stack[self.stack_size - 2].handle
                ));
            } else {
                glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
            }
        }

        self.stack_size -= 1;

        Viewport::pop();
        Profiler::end();
    }

    fn bind_tex2d(&mut self, tex: &Tex2D) {
        self.bind_tex2d_level(tex, 0);
    }

    fn bind_tex2d_level(&mut self, tex: &Tex2D, level: i32) {
        // Get bufs_ptr before mutable borrow of fbo
        let bufs_ptr = self.bufs.as_ptr();
        let fbo = self.get_active();
        let handle = Tex2D::get_handle(tex);
        let resource_id = tex.get_resource_id();

        if TexFormat::is_color(Tex2D::get_format(tex)) {
            if fbo.color_index >= 4 {
                panic!("RenderTarget_BindTex2D: Max color attachments exceeded");
            }

            let attachment = gl::COLOR_ATTACHMENT0 + fbo.color_index as u32;

            if is_command_mode() {
                // Use ResourceId-based command if texture was created on render thread
                // If no resource_id, lazily migrate texture to render thread
                if let Some(id) = resource_id {
                    submit_command(RenderCommand::FramebufferAttachTexture2DByResource {
                        attachment,
                        id,
                        level,
                    });
                } else {
                    // Texture was created before command mode - lazily migrate
                    let mut tex_clone = tex.clone();
                    if let Some(id) = tex_clone.ensure_resource_id() {
                        submit_command(RenderCommand::FramebufferAttachTexture2DByResource {
                            attachment,
                            id,
                            level,
                        });
                    } else {
                        // Fallback - will likely fail on render thread
                        warn!("RenderTarget: texture has no resource_id and cannot migrate");
                        submit_command(RenderCommand::FramebufferAttachTexture2D {
                            attachment,
                            texture: GpuHandle(handle),
                            level,
                        });
                    }
                }
            } else {
                glcheck!(gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    attachment,
                    gl::TEXTURE_2D,
                    handle,
                    level,
                ));
                glcheck!(gl::DrawBuffers(fbo.color_index + 1, bufs_ptr));
            }
            fbo.color_index += 1;
        } else {
            if fbo.depth {
                panic!("RenderTarget_BindTex2D: Target already has a depth buffer");
            }

            if is_command_mode() {
                // Use ResourceId-based command if texture was created on render thread
                // If no resource_id, lazily migrate texture to render thread
                if let Some(id) = resource_id {
                    submit_command(RenderCommand::FramebufferAttachTexture2DByResource {
                        attachment: gl::DEPTH_ATTACHMENT,
                        id,
                        level,
                    });
                } else {
                    // Texture was created before command mode - lazily migrate
                    let mut tex_clone = tex.clone();
                    if let Some(id) = tex_clone.ensure_resource_id() {
                        submit_command(RenderCommand::FramebufferAttachTexture2DByResource {
                            attachment: gl::DEPTH_ATTACHMENT,
                            id,
                            level,
                        });
                    } else {
                        // Fallback - will likely fail on render thread
                        warn!("RenderTarget: depth texture has no resource_id and cannot migrate");
                        submit_command(RenderCommand::FramebufferAttachTexture2D {
                            attachment: gl::DEPTH_ATTACHMENT,
                            texture: GpuHandle(handle),
                            level,
                        });
                    }
                }
            } else {
                glcheck!(gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::TEXTURE_2D,
                    handle,
                    level,
                ));
            }
            fbo.depth = true;
        }
    }

    fn bind_tex3d(&mut self, tex: &Tex3D, layer: i32) {
        self.bind_tex3d_level(tex, layer, 0);
    }

    fn bind_tex3d_level(&mut self, tex: &Tex3D, layer: i32, level: i32) {
        // Get bufs pointer before mutable borrow of stack entry
        let bufs_ptr = self.bufs.as_ptr();

        let fbo = self.get_active();
        if fbo.color_index >= 4 {
            panic!("RenderTarget_BindTex3D: Max color attachments exceeded");
        }

        let handle = Tex3D::get_handle(tex);
        let attachment = gl::COLOR_ATTACHMENT0 + fbo.color_index as u32;

        if is_command_mode() {
            submit_command(RenderCommand::FramebufferAttachTexture3D {
                attachment,
                texture: GpuHandle(handle),
                layer,
                level,
            });
        } else {
            glcheck!(gl::FramebufferTexture3D(
                gl::FRAMEBUFFER,
                attachment,
                gl::TEXTURE_3D,
                handle,
                level,
                layer,
            ));
            glcheck!(gl::DrawBuffers(fbo.color_index + 1, bufs_ptr));
        }
        fbo.color_index += 1;
    }

    fn bind_tex_cube(&mut self, tex: &TexCube, face: CubeFace) {
        self.bind_tex_cube_level(tex, face, 0);
    }

    fn bind_tex_cube_level(&mut self, tex: &TexCube, face: CubeFace, level: i32) {
        // Get bufs pointer before mutable borrow of stack entry
        let bufs_ptr = self.bufs.as_ptr();

        let fbo = self.get_active();
        if fbo.color_index >= 4 {
            panic!("RenderTarget_BindTexCubeLevel: Max color attachments exceeded");
        }
        let handle: u32 = TexCube::get_handle(tex);
        let attachment = gl::COLOR_ATTACHMENT0 + fbo.color_index as u32;

        if is_command_mode() {
            submit_command(RenderCommand::FramebufferAttachTextureCube {
                attachment,
                texture: GpuHandle(handle),
                face: face as u32,
                level,
            });
        } else {
            glcheck!(gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachment,
                face as u32,
                handle,
                level,
            ));
            glcheck!(gl::DrawBuffers(fbo.color_index + 1, bufs_ptr));
        }
        fbo.color_index += 1;
    }

    fn push_tex2d(&mut self, tex: &Tex2D) {
        self.push_tex2d_level(tex, 0);
    }

    fn push_tex2d_level(&mut self, tex: &Tex2D, level: i32) {
        let size = tex.get_size_level(level);
        self.push(size.x, size.y);
        self.bind_tex2d_level(tex, level);
    }

    fn push_tex3d(&mut self, tex: &Tex3D, layer: i32) {
        self.push_tex3d_level(tex, layer, 0);
    }

    fn push_tex3d_level(&mut self, tex: &Tex3D, layer: i32, level: i32) {
        let size = tex.get_size_level(level);
        self.push(size.x, size.y);
        self.bind_tex3d_level(tex, layer, level);
    }
}
