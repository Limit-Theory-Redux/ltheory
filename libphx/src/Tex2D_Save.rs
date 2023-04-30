use crate::internal::ffi;
use image::{DynamicImage, ImageBuffer, ImageOutputFormat};

#[no_mangle]
pub extern "C" fn Tex2D_Save_Png(
    path: *const libc::c_char,
    sx: i32,
    sy: i32,
    components: i32,
    data: *mut libc::c_uchar,
) -> bool {
    false
    //     // Create the image buffer from the given data
    //     let img_buf = ImageBuffer::from_raw(
    //         sx as u32,
    //         sy as u32,
    //         std::slice::from_raw_parts(data, (sx * sy * components) as usize),
    //     )
    //     .unwrap();

    //     // Convert the buffer to a dynamic image
    //     let img: DynamicImage = DynamicImage::ImageRgba8(img_buf);

    //     // Write the image to the file
    //     let file = File::create(ffi::PtrToSlice(path)).unwrap();
    //     img.write_to(file, ImageOutputFormat::Png).unwrap();
}
