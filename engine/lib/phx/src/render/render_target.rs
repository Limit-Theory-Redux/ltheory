use std::cell::RefCell;

use super::{gl, CubeFace, Tex2D, Tex3D, TexCube, TexFormat};
use crate::render::{glcheck, Viewport};
use crate::system::{Metric, Profiler};

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
        fbo.handle = 0;
        fbo.color_index = 0;
        fbo.sx = sx;
        fbo.sy = sy;
        fbo.depth = false;

        Metric::FBOSwap.inc();

        glcheck!(gl::GenFramebuffers(1, &mut fbo.handle));
        glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo.handle));

        Viewport::push(0, 0, sx, sy, false);
        Profiler::end();
    }

    fn pop(&mut self) {
        Profiler::begin("RenderTarget_Pop");

        if self.stack_size == 0 {
            panic!("RenderTarget_Pop: Attempting to pop an empty stack");
        }

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

        self.stack_size -= 1;

        Metric::FBOSwap.inc();

        if self.stack_size > 0 {
            glcheck!(gl::BindFramebuffer(
                gl::FRAMEBUFFER,
                self.get_active().handle
            ));
        } else {
            glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        }

        Viewport::pop();
        Profiler::end();
    }

    fn bind_tex2d(&mut self, tex: &Tex2D) {
        self.bind_tex2d_level(tex, 0);
    }

    fn bind_tex2d_level(&mut self, tex: &Tex2D, level: i32) {
        let fbo = self.get_active();
        let handle = Tex2D::get_handle(tex);

        if TexFormat::is_color(Tex2D::get_format(tex)) {
            if fbo.color_index >= 4 {
                panic!("RenderTarget_BindTex2D: Max color attachments exceeded");
            }

            glcheck!(gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0 + fbo.color_index as u32,
                gl::TEXTURE_2D,
                handle,
                level,
            ));
            fbo.color_index += 1;
            glcheck!(gl::DrawBuffers(fbo.color_index, self.bufs.as_ptr()));
        } else {
            if fbo.depth {
                panic!("RenderTarget_BindTex2D: Target already has a depth buffer");
            }

            glcheck!(gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                handle,
                level,
            ));
            fbo.depth = true;
        }
    }

    fn bind_tex3d(&mut self, tex: &Tex3D, layer: i32) {
        self.bind_tex3d_level(tex, layer, 0);
    }

    fn bind_tex3d_level(&mut self, tex: &Tex3D, layer: i32, level: i32) {
        let fbo = self.get_active();
        if fbo.color_index >= 4 {
            panic!("RenderTarget_BindTex3D: Max color attachments exceeded");
        }

        let handle = Tex3D::get_handle(tex);
        glcheck!(gl::FramebufferTexture3D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0 + fbo.color_index as u32,
            gl::TEXTURE_3D,
            handle,
            level,
            layer,
        ));
        fbo.color_index += 1;
        glcheck!(gl::DrawBuffers(fbo.color_index, self.bufs.as_ptr()));
    }

    fn bind_tex_cube(&mut self, tex: &TexCube, face: CubeFace) {
        self.bind_tex_cube_level(tex, face, 0);
    }

    fn bind_tex_cube_level(&mut self, tex: &TexCube, face: CubeFace, level: i32) {
        let fbo = self.get_active();
        if fbo.color_index >= 4 {
            panic!("RenderTarget_BindTexCubeLevel: Max color attachments exceeded");
        }
        let handle: u32 = TexCube::get_handle(tex);

        glcheck!(gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0 + fbo.color_index as u32,
            face as u32,
            handle,
            level,
        ));
        fbo.color_index += 1;
        glcheck!(gl::DrawBuffers(fbo.color_index, self.bufs.as_ptr()));
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
