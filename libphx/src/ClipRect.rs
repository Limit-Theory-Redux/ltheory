use crate::internal::Memory::*;
use crate::Viewport::*;
use glam::IVec2;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn glDisable(cap: GLenum);
    fn glEnable(cap: GLenum);
    fn glScissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ClipRect {
    pub x: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
    pub enabled: bool,
}
pub type GLenum = u32;
pub type GLsizei = i32;
pub type GLint = i32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ClipRectTransform {
    pub tx: f32,
    pub ty: f32,
    pub sx: f32,
    pub sy: f32,
}

#[inline]
unsafe extern "C" fn Max(mut a: f64, mut b: f64) -> f64 {
    if a > b { a } else { b }
}

#[inline]
unsafe extern "C" fn Min(mut a: f64, mut b: f64) -> f64 {
    if a < b { a } else { b }
}
static mut transform: [ClipRectTransform; 128] = [ClipRectTransform {
    tx: 0.,
    ty: 0.,
    sx: 0.,
    sy: 0.,
}; 128];
static mut transformIndex: i32 = -1_i32;
static mut rect: [ClipRect; 128] = [ClipRect {
    x: 0.,
    y: 0.,
    sx: 0.,
    sy: 0.,
    enabled: false,
}; 128];
static mut rectIndex: i32 = -1_i32;

#[inline]
unsafe extern "C" fn TransformRect(
    mut x: *mut f32,
    mut y: *mut f32,
    mut sx: *mut f32,
    mut sy: *mut f32,
) {
    if transformIndex >= 0_i32 {
        let mut curr: *mut ClipRectTransform =
            transform.as_mut_ptr().offset(transformIndex as isize);
        *x = (*curr).sx * *x + (*curr).tx;
        *y = (*curr).sy * *y + (*curr).ty;
        *sx = (*curr).sx * *sx;
        *sy = (*curr).sy * *sy;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_Activate(mut this: *mut ClipRect) {
    if !this.is_null() && (*this).enabled as i32 != 0 {
        let mut vpSize: IVec2 = IVec2 { x: 0, y: 0 };
        Viewport_GetSize(&mut vpSize);
        glEnable(0xc11_i32 as GLenum);
        let mut x: f32 = (*this).x;
        let mut y: f32 = (*this).y;
        let mut sx: f32 = (*this).sx;
        let mut sy: f32 = (*this).sy;
        TransformRect(&mut x, &mut y, &mut sx, &mut sy);
        glScissor(x as i32, vpSize.y - (y + sy) as i32, sx as i32, sy as i32);
    } else {
        glDisable(0xc11_i32 as GLenum);
    };
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_Push(mut x: f32, mut y: f32, mut sx: f32, mut sy: f32) {
    if rectIndex + 1_i32 >= 128_i32 {
        Fatal(b"ClipRect_Push: Maximum stack depth exceeded\0" as *const u8 as *const libc::c_char);
    }
    rectIndex += 1;
    let mut curr: *mut ClipRect = rect.as_mut_ptr().offset(rectIndex as isize);
    (*curr).x = x;
    (*curr).y = y;
    (*curr).sx = sx;
    (*curr).sy = sy;
    (*curr).enabled = true;
    ClipRect_Activate(curr);
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PushCombined(mut x: f32, mut y: f32, mut sx: f32, mut sy: f32) {
    let mut curr: *mut ClipRect = rect.as_mut_ptr().offset(rectIndex as isize);
    if rectIndex >= 0_i32 && (*curr).enabled as i32 != 0 {
        let mut maxX: f32 = x + sx;
        let mut maxY: f32 = y + sy;
        x = Max(x as f64, (*curr).x as f64) as f32;
        y = Max(y as f64, (*curr).y as f64) as f32;
        ClipRect_Push(
            x,
            y,
            (Min(maxX as f64, ((*curr).x + (*curr).sx) as f64) - x as f64) as f32,
            (Min(maxY as f64, ((*curr).y + (*curr).sy) as f64) - y as f64) as f32,
        );
    } else {
        ClipRect_Push(x, y, sx, sy);
    };
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PushDisabled() {
    if rectIndex + 1_i32 >= 128_i32 {
        Fatal(b"ClipRect_Push: Maximum stack depth exceeded\0" as *const u8 as *const libc::c_char);
    }
    rectIndex += 1;
    let mut curr: *mut ClipRect = rect.as_mut_ptr().offset(rectIndex as isize);
    (*curr).enabled = false;
    ClipRect_Activate(curr);
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PushTransform(
    mut tx: f32,
    mut ty: f32,
    mut sx: f32,
    mut sy: f32,
) {
    if transformIndex + 1_i32 >= 128_i32 {
        Fatal(
            b"ClipRect_PushTransform: Maximum stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    transformIndex += 1;
    let mut curr: *mut ClipRectTransform = transform.as_mut_ptr().offset(transformIndex as isize);
    (*curr).tx = tx;
    (*curr).ty = ty;
    (*curr).sx = sx;
    (*curr).sy = sy;
    if rectIndex >= 0_i32 {
        ClipRect_Activate(rect.as_mut_ptr().offset(rectIndex as isize));
    }
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_Pop() {
    if rectIndex < 0_i32 {
        Fatal(
            b"ClipRect_Pop: Attempting to pop an empty stack\0" as *const u8 as *const libc::c_char,
        );
    }
    rectIndex -= 1;
    ClipRect_Activate(if rectIndex >= 0_i32 {
        rect.as_mut_ptr().offset(rectIndex as isize)
    } else {
        std::ptr::null_mut()
    });
}

#[no_mangle]
pub unsafe extern "C" fn ClipRect_PopTransform() {
    if transformIndex < 0_i32 {
        Fatal(
            b"ClipRect_PopTransform: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    transformIndex -= 1;
    if rectIndex >= 0_i32 {
        ClipRect_Activate(rect.as_mut_ptr().offset(rectIndex as isize));
    }
}
