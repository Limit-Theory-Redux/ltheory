use stb::image_write::stbi_write_png;
use std::ffi::CStr;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save_Png(
    path: *const libc::c_char,
    sx: i32,
    sy: i32,
    components: i32,
    data: *mut libc::c_uchar,
) -> bool {
    let stride: i32 = components * sx;
    let result = stbi_write_png(
        CStr::from_ptr(path),
        sx,
        sy,
        components,
        std::slice::from_raw_parts(data, (stride * sy) as usize),
        stride,
    );
    result.is_some()
}
