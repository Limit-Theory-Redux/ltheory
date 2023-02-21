use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type Metric = int32;
static mut valueCurr: [int32; 8] = [0 as libc::c_int, 0, 0, 0, 0, 0, 0, 0];
#[no_mangle]
pub unsafe extern "C" fn Metric_Get(mut this: Metric) -> int32 {
    return valueCurr[this as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Metric_GetName(mut this: Metric) -> cstr {
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
    return 0 as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_AddDraw(
    mut polys: int32,
    mut tris: int32,
    mut verts: int32,
) {
    valueCurr[0x1] += 1 as libc::c_int;
    valueCurr[0x3] += polys;
    valueCurr[0x4] += tris;
    valueCurr[0x5] += verts;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_AddDrawImm(
    mut polys: int32,
    mut tris: int32,
    mut verts: int32,
) {
    valueCurr[0x2] += 1 as libc::c_int;
    valueCurr[0x3] += polys;
    valueCurr[0x4] += tris;
    valueCurr[0x5] += verts;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_Inc(mut this: Metric) {
    valueCurr[this as usize] += 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_Mod(mut this: Metric, mut delta: int32) {
    valueCurr[this as usize] += delta;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_Reset() {
    MemZero(
        valueCurr.as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[int32; 8]>(),
    );
}
