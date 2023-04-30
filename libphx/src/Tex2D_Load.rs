use crate::internal::ffi;
use crate::internal::Memory::*;
use image::{GenericImageView, ImageBuffer, Rgba};
use std::ffi::CStr;
use std::fs::File;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_LoadRaw(
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
    let img = image::open(ffi::PtrToSlice(path)).unwrap();
    let (width, height) = img.dimensions();

    *sx = width as i32;
    *sy = height as i32;

    let data = match img {
        ImageBuffer::<Rgba<u8>, _>(buf) => {
            let mut raw_data: Vec<u8> = vec![0; (width * height * 4) as usize];
            for (i, rgba) in buf.into_raw().iter().enumerate() {
                raw_data[i] = *rgba;
            }
            *components = 4;
            raw_data
        }
        _ => panic!("Unsupported color type"),
    };

    let memory: *mut libc::c_uchar = MemAlloc(data.len()) as *mut libc::c_uchar;
    MemCpy(
        memory as *mut _,
        data.as_slice().as_ptr() as *mut _,
        data.len(),
    );
    memory
}
