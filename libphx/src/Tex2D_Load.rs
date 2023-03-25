use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
use stb::image::{stbi_load_from_reader, Channels};
use std::ffi::CStr;
use std::fs::File;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_LoadRaw(
    path: *const libc::c_char,
    sx: *mut i32,
    sy: *mut i32,
    components: *mut i32,
) -> *mut libc::c_uchar {
    let path_as_str = CStr::from_ptr(path).to_str().unwrap();
    match File::open(path_as_str) {
        Ok(mut reader) => {
            let result = stbi_load_from_reader(&mut reader, Channels::Default);
            match result {
                Some((info, data)) => {
                    *sx = info.width;
                    *sy = info.height;
                    *components = info.components;

                    let memory: *mut libc::c_uchar = MemAlloc(data.size()) as *mut libc::c_uchar;
                    MemCpy(
                        memory as *mut _,
                        data.as_slice().as_ptr() as *mut _,
                        data.size(),
                    );
                    memory
                }
                None => Fatal(c_str!("Failed to load image from '%s'"), path),
            }
        }
        Err(_) => Fatal(c_str!("Failed to load image from '%s'"), path),
    }
}
