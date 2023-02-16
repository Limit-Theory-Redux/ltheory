use libc;
use std::ffi::CStr;
use stb::image_write::stbi_write_png;

pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save_Png(
    mut path: cstr,
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut components: libc::c_int,
    mut data: *mut uchar,
) -> bool {
    let mut stride: i32 = components * sx;
    let mut result = stbi_write_png(
        CStr::from_ptr(path),
        sx,
        sy,
        components,
        std::slice::from_raw_parts(data, (stride * sy) as usize),
        stride,
    );
    return result.is_some();
}
