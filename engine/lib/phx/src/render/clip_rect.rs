use glam::IVec2;

use crate::render::{Renderer, Viewport};

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

/// What `ClipManager`'s CPU-side bookkeeping decided the GL scissor state
/// should become. The caller (which holds `&mut Renderer`) turns this into
/// the matching `submit()` calls.
enum ScissorUpdate {
    Disable,
    Set {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
}

fn apply(r: &mut Renderer, update: Option<ScissorUpdate>) {
    match update {
        None => {}
        Some(ScissorUpdate::Disable) => r.enable_scissor(false),
        Some(ScissorUpdate::Set {
            x,
            y,
            width,
            height,
        }) => {
            r.enable_scissor(true);
            r.set_scissor(x, y, width, height);
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl ClipRect {
    pub fn push(r: &mut Renderer, x: f32, y: f32, sx: f32, sy: f32) {
        let vp_size = Viewport::get_size(r);
        let update = r.data.clip_rect.push(vp_size, x, y, sx, sy);
        apply(r, update);
    }

    pub fn push_combined(r: &mut Renderer, x: f32, y: f32, sx: f32, sy: f32) {
        let vp_size = Viewport::get_size(r);
        let update = r.data.clip_rect.push_combined(vp_size, x, y, sx, sy);
        apply(r, update);
    }

    pub fn push_disabled(r: &mut Renderer) {
        let vp_size = Viewport::get_size(r);
        let update = r.data.clip_rect.push_disabled(vp_size);
        apply(r, update);
    }

    pub fn push_transform(r: &mut Renderer, tx: f32, ty: f32, sx: f32, sy: f32) {
        let vp_size = Viewport::get_size(r);
        let update = r.data.clip_rect.push_transform(vp_size, tx, ty, sx, sy);
        apply(r, update);
    }

    pub fn pop(r: &mut Renderer) {
        let vp_size = Viewport::get_size(r);
        let update = r.data.clip_rect.pop(vp_size);
        apply(r, update);
    }

    pub fn pop_transform(r: &mut Renderer) {
        let vp_size = Viewport::get_size(r);
        let update = r.data.clip_rect.pop_transform(vp_size);
        apply(r, update);
    }
}

/// Clip-rect stack, owned by `Renderer` (was `thread_local! CLIP_MANAGER`) -
/// GPU scissor state has to live with whatever owns the GL context. Its
/// methods are pure CPU bookkeeping: they return the scissor update to apply
/// rather than touching GL/`Renderer` themselves, since they're called
/// through `renderer.clip_rect`, which already holds `Renderer` borrowed.
pub struct ClipManager {
    transforms: [ClipRectTransform; MAX_STACK_DEPTH],
    transforms_count: usize,
    rects: [ClipRect; MAX_STACK_DEPTH],
    rects_count: usize,
}

impl ClipManager {
    pub fn new() -> Self {
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

    fn activate(&mut self, vp_size: IVec2) -> ScissorUpdate {
        let rect = &mut self.rects[self.rects_count - 1];
        if !rect.enabled {
            return ScissorUpdate::Disable;
        }

        let mut x = rect.x;
        let mut y = rect.y;
        let mut sx = rect.sx;
        let mut sy = rect.sy;

        self.transform_rect(&mut x, &mut y, &mut sx, &mut sy);

        ScissorUpdate::Set {
            x: x as i32,
            y: vp_size.y - (y + sy) as i32,
            width: sx as i32,
            height: sy as i32,
        }
    }

    fn push_rect_intern(
        &mut self,
        vp_size: IVec2,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    ) -> ScissorUpdate {
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
        self.activate(vp_size)
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

    fn push(&mut self, vp_size: IVec2, x: f32, y: f32, sx: f32, sy: f32) -> Option<ScissorUpdate> {
        Some(self.push_rect_intern(vp_size, x, y, sx, sy))
    }

    fn push_combined(
        &mut self,
        vp_size: IVec2,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    ) -> Option<ScissorUpdate> {
        if self.rects_count > 0 {
            let curr = self.rects[self.rects_count - 1];
            if curr.enabled {
                let max_x = x + sx;
                let max_y = y + sy;
                let x = f32::max(x, curr.x);
                let y = f32::max(y, curr.y);
                let sx = f32::min(max_x, curr.x + curr.sx) - x;
                let sy = f32::min(max_y, curr.y + curr.sy) - y;

                return Some(self.push_rect_intern(vp_size, x, y, sx, sy));
            }
        }
        Some(self.push_rect_intern(vp_size, x, y, sx, sy))
    }

    fn push_disabled(&mut self, vp_size: IVec2) -> Option<ScissorUpdate> {
        if self.rects_count >= MAX_STACK_DEPTH {
            panic!("ClipRect_PushDisabled: Maximum stack depth exceeded");
        }

        let rect = &mut self.rects[self.rects_count];
        rect.enabled = false;

        self.rects_count += 1;
        Some(self.activate(vp_size))
    }

    fn push_transform(
        &mut self,
        vp_size: IVec2,
        tx: f32,
        ty: f32,
        sx: f32,
        sy: f32,
    ) -> Option<ScissorUpdate> {
        self.push_transform_intern(tx, ty, sx, sy);

        (self.rects_count > 0).then(|| self.activate(vp_size))
    }

    fn pop(&mut self, vp_size: IVec2) -> Option<ScissorUpdate> {
        if self.rects_count == 0 {
            panic!("ClipRect_Pop: Attempting to pop an empty stack");
        }
        self.rects_count -= 1;

        if self.rects_count > 0 {
            Some(self.activate(vp_size))
        } else {
            Some(ScissorUpdate::Disable)
        }
    }

    fn pop_transform(&mut self, vp_size: IVec2) -> Option<ScissorUpdate> {
        if self.transforms_count == 0 {
            panic!("ClipRect_PopTransform: Attempting to pop an empty stack");
        }
        self.transforms_count -= 1;

        (self.rects_count > 0).then(|| self.activate(vp_size))
    }
}

impl Default for ClipManager {
    fn default() -> Self {
        Self::new()
    }
}
