use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type CubeFace = i32;

#[no_mangle]
pub static CubeFace_PX: CubeFace = 0x8515 as i32;

#[no_mangle]
pub static CubeFace_NX: CubeFace = 0x8516 as i32;

#[no_mangle]
pub static CubeFace_PY: CubeFace = 0x8517 as i32;

#[no_mangle]
pub static CubeFace_NY: CubeFace = 0x8518 as i32;

#[no_mangle]
pub static CubeFace_PZ: CubeFace = 0x8519 as i32;

#[no_mangle]
pub static CubeFace_NZ: CubeFace = 0x851a as i32;

static mut kFaces: [CubeFace; 6] = unsafe {
    [
        CubeFace_PX,
        CubeFace_NX,
        CubeFace_PY,
        CubeFace_NY,
        CubeFace_PZ,
        CubeFace_NZ,
    ]
};

#[no_mangle]
pub unsafe extern "C" fn CubeFace_Get(mut index: i32) -> CubeFace {
    return kFaces[index as usize];
}
