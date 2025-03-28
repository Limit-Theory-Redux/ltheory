use crate::common::*;

pub type Metric = i32;

const VALUE_CURR_LEN: usize = 8;
// TODO: figure out why we get an integer overflow in Metric_AddDraw for verts
static mut VALUE_CURR: [i64; VALUE_CURR_LEN] = [0, 0, 0, 0, 0, 0, 0, 0];

#[no_mangle]
pub unsafe extern "C" fn Metric_Get(this: Metric) -> i32 {
    VALUE_CURR[this as usize] as i32
}

#[no_mangle]
pub extern "C" fn Metric_GetName(this: Metric) -> *const libc::c_char {
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
    VALUE_CURR[0x1] += 1;
    VALUE_CURR[0x3] += polys as i64;
    VALUE_CURR[0x4] += tris as i64;
    VALUE_CURR[0x5] += verts as i64;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_AddDrawImm(polys: i32, tris: i32, verts: i32) {
    VALUE_CURR[0x2] += 1;
    VALUE_CURR[0x3] += polys as i64;
    VALUE_CURR[0x4] += tris as i64;
    VALUE_CURR[0x5] += verts as i64;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Inc(this: Metric) {
    VALUE_CURR[this as usize] += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Mod(this: Metric, delta: i32) {
    VALUE_CURR[this as usize] += delta as i64;
}

#[no_mangle]
pub unsafe extern "C" fn Metric_Reset() {
    #[allow(clippy::needless_range_loop)]
    for i in 0..VALUE_CURR_LEN {
        VALUE_CURR[i] = 0;
    }
}
