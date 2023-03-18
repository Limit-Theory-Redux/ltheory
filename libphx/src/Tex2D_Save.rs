use libc;
use stb::image_write::stbi_write_png;
use std::ffi::CStr;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save_Png(
    mut path: *const libc::c_char,
    mut sx: i32,
    mut sy: i32,
    mut components: i32,
    mut data: *mut libc::c_uchar,
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
    result.is_some()
}
