use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
}
pub type int32_t = libc::c_int;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type Metric = int32;
#[inline]
unsafe extern "C" fn MemZero(mut dst: *mut libc::c_void, mut size: size_t) {
    memset(dst, 0 as libc::c_int, size);
}
static mut valueCurr: [int32; 8] = [0 as libc::c_int, 0, 0, 0, 0, 0, 0, 0];
#[no_mangle]
pub unsafe extern "C" fn Metric_Get(mut self_0: Metric) -> int32 {
    return valueCurr[self_0 as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Metric_GetName(mut self_0: Metric) -> cstr {
    match self_0 {
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
    valueCurr[0x1 as libc::c_int as usize] += 1 as libc::c_int;
    valueCurr[0x3 as libc::c_int as usize] += polys;
    valueCurr[0x4 as libc::c_int as usize] += tris;
    valueCurr[0x5 as libc::c_int as usize] += verts;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_AddDrawImm(
    mut polys: int32,
    mut tris: int32,
    mut verts: int32,
) {
    valueCurr[0x2 as libc::c_int as usize] += 1 as libc::c_int;
    valueCurr[0x3 as libc::c_int as usize] += polys;
    valueCurr[0x4 as libc::c_int as usize] += tris;
    valueCurr[0x5 as libc::c_int as usize] += verts;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_Inc(mut self_0: Metric) {
    valueCurr[self_0 as usize] += 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_Mod(mut self_0: Metric, mut delta: int32) {
    valueCurr[self_0 as usize] += delta;
}
#[no_mangle]
pub unsafe extern "C" fn Metric_Reset() {
    MemZero(
        valueCurr.as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[int32; 8]>() as libc::c_ulong,
    );
}
