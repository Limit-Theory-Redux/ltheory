use crate::internal::ffi;
use crate::internal::Memory::*;
use crate::Common::*;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView};
use std::ffi::CStr;
use std::fs::File;

#[no_mangle]
pub extern "C" fn Tex2D_LoadRaw(
    path: *const libc::c_char,
    sx: &mut i32,
    sy: &mut i32,
    components: &mut i32,
) -> *mut libc::c_uchar {
    // let path_as_str = CStr::from_ptr(path).to_str().unwrap();
    // match File::open(path_as_str) {
    //     Ok(mut reader) => match stbi_load_from_reader(&mut reader, Channels::Default) {
    //         Some((info, data)) => {
    //             *sx = info.width;
    //             *sy = info.height;
    //             *components = info.components;

    //             let memory: *mut libc::c_uchar = MemAlloc(data.size()) as *mut libc::c_uchar;
    //             MemCpy(
    //                 memory as *mut _,
    //                 data.as_slice().as_ptr() as *mut _,
    //                 data.size(),
    //             );
    //             memory
    //         }
    //         None => CFatal!("Failed to load image from '%s'", path),
    //     },
    //     Err(_) => CFatal!("Failed to load image from '%s'", path),
    // }
    let reader = ImageReader::open(ffi::PtrAsSlice(path))
        .unwrap_or_else(|_| CFatal!("Failed to load image from '%s', unable to open file", path));
    let img = reader
        .decode()
        .unwrap_or_else(|_| CFatal!("Failed to load image from '%s', decode failed", path));
    let (width, height) = img.dimensions();

    *sx = width as i32;
    *sy = height as i32;

    let data = match img {
        DynamicImage::ImageRgba8(buf) => {
            *components = 4;
            buf.into_raw()
        }
        DynamicImage::ImageRgb8(buf) => {
            *components = 3;
            buf.into_raw()
        }
        _ => CFatal!(
            "Failed to load image from '%s', unsupported image format",
            path
        ),
    };

    // Copy the data to a malloc allocated buffer.
    unsafe {
        let memory: *mut libc::c_uchar = MemAlloc(data.len()) as *mut libc::c_uchar;
        MemCpy(
            memory as *mut _,
            data.as_slice().as_ptr() as *mut _,
            data.len(),
        );
        memory
    }
}
