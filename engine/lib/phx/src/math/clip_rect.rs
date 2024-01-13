use super::*;
use crate::common::*;
use crate::render::*;

const MAX_STACK_DEPTH: i32 = 128;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ClipRect {
    pub x: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
    pub enabled: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ClipRectTransform {
    pub tx: f32,
    pub ty: f32,
    pub sx: f32,
    pub sy: f32,
}
static mut transform: [ClipRectTransform; 128] = [ClipRectTransform {
    tx: 0.,
    ty: 0.,
    sx: 0.,
    sy: 0.,
}; 128];

static mut transformIndex: i32 = -1;

static mut rect: [ClipRect; 128] = [ClipRect {
    x: 0.,
    y: 0.,
    sx: 0.,
    sy: 0.,
    enabled: false,
}; 128];

static mut rectIndex: i32 = -1;

#[inline]
unsafe extern "C" fn TransformRect(x: &mut f32, y: &mut f32, sx: &mut f32, sy: &mut f32) {
    if transformIndex >= 0 {
        let curr: *mut ClipRectTransform = transform.as_mut_ptr().offset(transformIndex as isize);
        *x = (*curr).sx * *x + (*curr).tx;
        *y = (*curr).sy * *y + (*curr).ty;
        *sx *= (*curr).sx;
        *sy *= (*curr).sy;
    }
}

#[no_mangle]
pub extern "C" fn ClipRect_Activate(this: Option<&mut ClipRect>) {
    match this {
        Some(this) => {
            let mut vpSize: IVec2 = IVec2::ZERO;

            unsafe { Viewport_GetSize(&mut vpSize) };
            gl_enable(gl::SCISSOR_TEST);

            let mut x: f32 = this.x;
            let mut y: f32 = this.y;
            let mut sx: f32 = this.sx;
            let mut sy: f32 = this.sy;

            unsafe { TransformRect(&mut x, &mut y, &mut sx, &mut sy) };
            gl_scissor(x as i32, vpSize.y - (y + sy) as i32, sx as i32, sy as i32);
        }
        None => gl_disable(gl::SCISSOR_TEST),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_Push(x: f32, y: f32, sx: f32, sy: f32) {
    if rectIndex + 1 >= MAX_STACK_DEPTH {
        panic!("ClipRect_Push: Maximum stack depth exceeded");
    }
    rectIndex += 1;
    let curr: *mut ClipRect = rect.as_mut_ptr().offset(rectIndex as isize);
    (*curr).x = x;
    (*curr).y = y;
    (*curr).sx = sx;
    (*curr).sy = sy;
    (*curr).enabled = true;
    ClipRect_Activate(Some(&mut *curr));
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PushCombined(x: f32, y: f32, sx: f32, sy: f32) {
    let curr: *mut ClipRect = rect.as_mut_ptr().offset(rectIndex as isize);
    if rectIndex >= 0 && (*curr).enabled as i32 != 0 {
        let maxX: f32 = x + sx;
        let maxY: f32 = y + sy;
        let x = f32::max(x, (*curr).x);
        let y = f32::max(y, (*curr).y);

        ClipRect_Push(
            x,
            y,
            f32::min(maxX, (*curr).x + (*curr).sx) - x,
            f32::min(maxY, (*curr).y + (*curr).sy) - y,
        );
    } else {
        ClipRect_Push(x, y, sx, sy);
    };
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PushDisabled() {
    if rectIndex + 1 >= MAX_STACK_DEPTH {
        panic!("ClipRect_Push: Maximum stack depth exceeded");
    }

    rectIndex += 1;
    let curr: *mut ClipRect = rect.as_mut_ptr().offset(rectIndex as isize);
    (*curr).enabled = false;
    ClipRect_Activate(Some(&mut *curr));
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PushTransform(tx: f32, ty: f32, sx: f32, sy: f32) {
    if transformIndex + 1 >= MAX_STACK_DEPTH {
        panic!("ClipRect_PushTransform: Maximum stack depth exceeded");
    }
    transformIndex += 1;
    let curr: *mut ClipRectTransform = transform.as_mut_ptr().offset(transformIndex as isize);
    (*curr).tx = tx;
    (*curr).ty = ty;
    (*curr).sx = sx;
    (*curr).sy = sy;
    if rectIndex >= 0 {
        ClipRect_Activate(Some(&mut *rect.as_mut_ptr().offset(rectIndex as isize)));
    }
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_Pop() {
    if rectIndex < 0 {
        panic!("ClipRect_Pop: Attempting to pop an empty stack");
    }
    rectIndex -= 1;
    ClipRect_Activate(if rectIndex >= 0 {
        Some(&mut *rect.as_mut_ptr().offset(rectIndex as isize))
    } else {
        None
    });
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PopTransform() {
    if transformIndex < 0 {
        panic!("ClipRect_PopTransform: Attempting to pop an empty stack");
    }
    transformIndex -= 1;
    if rectIndex >= 0 {
        ClipRect_Activate(Some(&mut *rect.as_mut_ptr().offset(rectIndex as isize)));
    }
}
