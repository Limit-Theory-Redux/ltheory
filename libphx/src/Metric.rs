use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

pub type Metric = i32;

static mut valueCurr: [i32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

#[no_mangle]
pub unsafe extern "C" fn Metric_Get(this: Metric) -> i32 {
    valueCurr[this as usize]
}

#[no_mangle]
pub unsafe extern "C" fn Metric_GetName(this: Metric) -> *const libc::c_char {
    match this {
        1 => return c_str!("Draw Calls"),
        2 => return c_str!("Draw Calls (Immediate)"),
        3 => return c_str!("Polys"),
        4 => return c_str!("Tris"),
        5 => return c_str!("Vertices"),
        6 => return c_str!("Pipeline Flushes"),
        7 => return c_str!("Framebuffer Swaps"),
        _ => {}
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn Metric_AddDraw(polys: i32, tris: i32, verts: i32) {
    valueCurr[0x1] += 1;
    valueCurr[0x3] += polys;
    valueCurr[0x4] += tris;
    valueCurr[0x5] += verts;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_AddDrawImm(polys: i32, tris: i32, verts: i32) {
    valueCurr[0x2] += 1;
    valueCurr[0x3] += polys;
    valueCurr[0x4] += tris;
    valueCurr[0x5] += verts;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Inc(this: Metric) {
    valueCurr[this as usize] += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Mod(this: Metric, delta: i32) {
    valueCurr[this as usize] += delta;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Reset() {
    MemZero(
        valueCurr.as_mut_ptr() as *mut _,
        std::mem::size_of::<[i32; 8]>(),
    );
}
