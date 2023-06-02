use crate::common::*;
use crate::internal::*;
use crate::*;

use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView};

#[no_mangle]
pub extern "C" fn Tex2D_LoadRaw(
    path: *const libc::c_char,
    sx: &mut i32,
    sy: &mut i32,
    components: &mut i32,
) -> *mut libc::c_uchar {
    tex2d_load_raw(&path.convert(), sx, sy, components)
}

pub fn tex2d_load_raw(
    path: &str,
    sx: &mut i32,
    sy: &mut i32,
    components: &mut i32,
) -> *mut libc::c_uchar {
    let reader = ImageReader::open(path)
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
