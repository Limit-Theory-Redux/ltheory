use crate::internal::Memory::*;
use glam::Vec3;
use libc;
use stb::image::{stbi_load_from_reader, Channels};
use std::ffi::CStr;
use std::fs::File;

extern "C" {
    pub type __sFILEX;
    fn Fatal(_: *const libc::c_char, _: ...);
}

pub type uchar = libc::c_uchar;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_LoadRaw(
    mut path: *const libc::c_char,
    mut sx: *mut i32,
    mut sy: *mut i32,
    mut components: *mut i32,
) -> *mut uchar {
    let path_as_str = CStr::from_ptr(path).to_str().unwrap();
    match File::open(path_as_str) {
        Ok(mut reader) => {
            let result = stbi_load_from_reader(&mut reader, Channels::Default);
            match result {
                Some((info, data)) => {
                    *sx = info.width as i32;
                    *sy = info.height as i32;
                    *components = info.components as i32;

                    let mut memory: *mut uchar = MemAlloc(data.size()) as *mut uchar;
                    MemCpy(
                        memory as *mut libc::c_void,
                        data.as_slice().as_ptr() as *mut libc::c_void,
                        data.size(),
                    );
                    memory
                }
                None => {
                    Fatal(
                        b"Failed to load image from '%s'\0" as *const u8 as *const libc::c_char,
                        path,
                    );
                    0 as *mut uchar
                }
            }
        }
        Err(_) => {
            Fatal(
                b"Failed to load image from '%s'\0" as *const u8 as *const libc::c_char,
                path,
            );
            0 as *mut uchar
        }
    }
}
