use ::libc;
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type TexFilter = int32;
#[no_mangle]
pub static mut TexFilter_Point: TexFilter = 0x2600 as libc::c_int;
#[no_mangle]
pub static mut TexFilter_PointMipPoint: TexFilter = 0x2700 as libc::c_int;
#[no_mangle]
pub static mut TexFilter_PointMipLinear: TexFilter = 0x2702 as libc::c_int;
#[no_mangle]
pub static mut TexFilter_Linear: TexFilter = 0x2601 as libc::c_int;
#[no_mangle]
pub static mut TexFilter_LinearMipPoint: TexFilter = 0x2701 as libc::c_int;
#[no_mangle]
pub static mut TexFilter_LinearMipLinear: TexFilter = 0x2703 as libc::c_int;
