use ::libc;
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type CubeFace = int32;
#[no_mangle]
pub static mut CubeFace_PX: CubeFace = 0x8515 as libc::c_int;
#[no_mangle]
pub static mut CubeFace_NX: CubeFace = 0x8516 as libc::c_int;
#[no_mangle]
pub static mut CubeFace_PY: CubeFace = 0x8517 as libc::c_int;
#[no_mangle]
pub static mut CubeFace_NY: CubeFace = 0x8518 as libc::c_int;
#[no_mangle]
pub static mut CubeFace_PZ: CubeFace = 0x8519 as libc::c_int;
#[no_mangle]
pub static mut CubeFace_NZ: CubeFace = 0x851a as libc::c_int;
static mut kFaces: [CubeFace; 6] = unsafe {
    [CubeFace_PX, CubeFace_NX, CubeFace_PY, CubeFace_NY, CubeFace_PZ, CubeFace_NZ]
};
#[no_mangle]
pub unsafe extern "C" fn CubeFace_Get(mut index: libc::c_int) -> CubeFace {
    return kFaces[index as usize];
}
