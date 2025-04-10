use std::cell::RefCell;

use glam::IVec2;

use crate::render::{gl, glcheck, Viewport_GetSize};

const MAX_STACK_DEPTH: usize = 128;

#[derive(Copy, Clone)]
struct ClipRectTransform {
    tx: f32,
    ty: f32,
    sx: f32,
    sy: f32,
}

#[derive(Copy, Clone)]
pub struct ClipRect {
    x: f32,
    y: f32,
    sx: f32,
    sy: f32,
    enabled: bool,
}

#[luajit_ffi_gen::luajit_ffi]
impl ClipRect {
    pub fn push(x: f32, y: f32, sx: f32, sy: f32) {
        CLIP_MANAGER.with_borrow_mut(|cm| cm.push(x, y, sx, sy));
    }

    pub fn push_combined(x: f32, y: f32, sx: f32, sy: f32) {
        CLIP_MANAGER.with_borrow_mut(|cm| cm.push_combined(x, y, sx, sy));
    }

    pub fn push_disabled() {
        CLIP_MANAGER.with_borrow_mut(|cm| cm.push_disabled());
    }

    pub fn push_transform(tx: f32, ty: f32, sx: f32, sy: f32) {
        CLIP_MANAGER.with_borrow_mut(|cm| cm.push_transform(tx, ty, sx, sy));
    }

    pub fn pop() {
        CLIP_MANAGER.with_borrow_mut(|cm| cm.pop());
    }

    pub fn pop_transform() {
        CLIP_MANAGER.with_borrow_mut(|cm| cm.pop_transform());
    }
}

thread_local! { static CLIP_MANAGER: RefCell<ClipManager> = RefCell::new(ClipManager::new()); }

struct ClipManager {
    transforms: [ClipRectTransform; MAX_STACK_DEPTH],
    transforms_count: usize,
    rects: [ClipRect; MAX_STACK_DEPTH],
    rects_count: usize,
}

impl ClipManager {
    fn new() -> Self {
        Self {
            transforms: [ClipRectTransform {
                tx: 0.,
                ty: 0.,
                sx: 0.,
                sy: 0.,
            }; MAX_STACK_DEPTH],
            transforms_count: 0,
            rects: [ClipRect {
                x: 0.,
                y: 0.,
                sx: 0.,
                sy: 0.,
                enabled: false,
            }; MAX_STACK_DEPTH],
            rects_count: 0,
        }
    }

    #[inline]
    fn transform_rect(&self, x: &mut f32, y: &mut f32, sx: &mut f32, sy: &mut f32) {
        if self.transforms_count > 0 {
            let t = &self.transforms[self.transforms_count - 1];
            *x = t.sx * *x + t.tx;
            *y = t.sy * *y + t.ty;
            *sx *= t.sx;
            *sy *= t.sy;
        }
    }

    fn activate(&mut self) {
        let rect = &mut self.rects[self.rects_count - 1];
        if !rect.enabled {
            glcheck!(gl::Disable(gl::SCISSOR_TEST));
            return;
        }
        
        let mut vp_size = IVec2::ZERO;

        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            Viewport_GetSize(&mut vp_size);
        }
        glcheck!(gl::Enable(gl::SCISSOR_TEST));

        let mut x = rect.x;
        let mut y = rect.y;
        let mut sx = rect.sx;
        let mut sy = rect.sy;

        self.transform_rect(&mut x, &mut y, &mut sx, &mut sy);

        glcheck!(gl::Scissor(
            x as i32,
            vp_size.y - (y + sy) as i32,
            sx as i32,
            sy as i32
        ));
    }

    fn push_rect_intern(&mut self, x: f32, y: f32, sx: f32, sy: f32) {
        if self.rects_count >= MAX_STACK_DEPTH {
            panic!("ClipRect.Push: Maximum stack depth {MAX_STACK_DEPTH} exceeded");
        }

        let rect = &mut self.rects[self.rects_count];
        rect.x = x;
        rect.y = y;
        rect.sx = sx;
        rect.sy = sy;
        rect.enabled = true;

        self.rects_count += 1;
        self.activate();
    }

    fn push_transform_intern(&mut self, tx: f32, ty: f32, sx: f32, sy: f32) {
        if self.transforms_count >= MAX_STACK_DEPTH {
            panic!("ClipRect.PushTransform: Maximum stack depth {MAX_STACK_DEPTH} exceeded");
        }

        let transform = &mut self.transforms[self.transforms_count];
        transform.tx = tx;
        transform.ty = ty;
        transform.sx = sx;
        transform.sy = sy;

        self.transforms_count += 1;
    }

    fn push(&mut self, x: f32, y: f32, sx: f32, sy: f32) {
        self.push_rect_intern(x, y, sx, sy);
    }

    fn push_combined(&mut self, x: f32, y: f32, sx: f32, sy: f32) {
        if self.rects_count > 0 {
            let curr = self.rects[self.rects_count - 1];
            if curr.enabled {
                let max_x = x + sx;
                let max_y = y + sy;
                let x = f32::max(x, curr.x);
                let y = f32::max(y, curr.y);
                let sx = f32::min(max_x, curr.x + curr.sx) - x;
                let sy = f32::min(max_y, curr.y + curr.sy) - y;

                self.push_rect_intern(x, y, sx, sy);
                return;
            }
        }
        self.push_rect_intern(x, y, sx, sy);
    }

    fn push_disabled(&mut self) {
        if self.rects_count >= MAX_STACK_DEPTH {
            panic!("ClipRect_PushDisabled: Maximum stack depth exceeded");
        }

        let rect = &mut self.rects[self.rects_count];
        rect.enabled = false;

        self.rects_count += 1;
        self.activate();
    }

    fn push_transform(&mut self, tx: f32, ty: f32, sx: f32, sy: f32) {
        self.push_transform_intern(tx, ty, sx, sy);

        if self.rects_count > 0 {
            self.activate();
        }
    }

    fn pop(&mut self) {
        if self.rects_count == 0 {
            panic!("ClipRect_Pop: Attempting to pop an empty stack");
        }
        self.rects_count -= 1;

        if self.rects_count > 0 {
            self.activate();
        } else {
            glcheck!(gl::Disable(gl::SCISSOR_TEST));
        }
    }

    fn pop_transform(&mut self) {
        if self.transforms_count == 0 {
            panic!("ClipRect_PopTransform: Attempting to pop an empty stack");
        }
        self.transforms_count -= 1;

        if self.rects_count > 0 {
            self.activate();
        }
    }
}
