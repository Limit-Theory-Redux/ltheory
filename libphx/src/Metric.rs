use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {}
pub type Metric = i32;
static mut valueCurr: [i32; 8] = [0_i32, 0, 0, 0, 0, 0, 0, 0];

#[no_mangle]
pub unsafe extern "C" fn Metric_Get(mut this: Metric) -> i32 {
    return valueCurr[this as usize];
}

#[no_mangle]
pub unsafe extern "C" fn Metric_GetName(mut this: Metric) -> *const libc::c_char {
    match this {
        1 => return b"Draw Calls\0" as *const u8 as *const libc::c_char,
        2 => return b"Draw Calls (Immediate)\0" as *const u8 as *const libc::c_char,
        3 => return b"Polys\0" as *const u8 as *const libc::c_char,
        4 => return b"Tris\0" as *const u8 as *const libc::c_char,
        5 => return b"Vertices\0" as *const u8 as *const libc::c_char,
        6 => return b"Pipeline Flushes\0" as *const u8 as *const libc::c_char,
        7 => return b"Framebuffer Swaps\0" as *const u8 as *const libc::c_char,
        _ => {}
    }
    return std::ptr::null();
}

#[no_mangle]
pub unsafe extern "C" fn Metric_AddDraw(mut polys: i32, mut tris: i32, mut verts: i32) {
    valueCurr[0x1] += 1_i32;
    valueCurr[0x3] += polys;
    valueCurr[0x4] += tris;
    valueCurr[0x5] += verts;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_AddDrawImm(mut polys: i32, mut tris: i32, mut verts: i32) {
    valueCurr[0x2] += 1_i32;
    valueCurr[0x3] += polys;
    valueCurr[0x4] += tris;
    valueCurr[0x5] += verts;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Inc(mut this: Metric) {
    valueCurr[this as usize] += 1_i32;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Mod(mut this: Metric, mut delta: i32) {
    valueCurr[this as usize] += delta;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Reset() {
    MemZero(
        valueCurr.as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[i32; 8]>(),
    );
}
