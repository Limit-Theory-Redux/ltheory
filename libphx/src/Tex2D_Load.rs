use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use std::fs::File;
use std::ffi::CStr;
use ::stb::image::{stbi_load_from_reader, Channels};

extern "C" {
    pub type __sFILEX;
    fn Fatal(_: cstr, _: ...);
}

pub type cstr = *const libc::c_char;
pub type uchar = libc::c_uchar;

#[no_mangle]
pub unsafe extern "C" fn Tex2D_LoadRaw(
    mut path: cstr,
    mut sx: *mut libc::c_int,
    mut sy: *mut libc::c_int,
    mut components: *mut libc::c_int,
) -> *mut uchar {
    let path_as_str = CStr::from_ptr(path).to_str().unwrap();
    match File::open(path_as_str) {
        Ok(mut reader) => {
            let result = stbi_load_from_reader(&mut reader, Channels::Default);
            match result {
                Some((info, data)) => {
                    *sx = info.width as libc::c_int;
                    *sy = info.height as libc::c_int;
                    *components = info.components as libc::c_int;

                    let mut memory: *mut uchar = MemAlloc(data.size()) as *mut uchar;
                    MemCpy(memory as *mut libc::c_void, data.as_slice().as_ptr() as *mut libc::c_void, data.size());
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
        },
        Err(_) => {
            Fatal(
                b"Failed to load image from '%s'\0" as *const u8 as *const libc::c_char,
                path,
            );
            0 as *mut uchar
        }
    }
}
