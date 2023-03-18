use crate::GL::gl;
use libc;

pub type DataFormat = i32;

#[no_mangle]
pub static DataFormat_U8: DataFormat = gl::UNSIGNED_BYTE as DataFormat;

#[no_mangle]
pub static DataFormat_I8: DataFormat = gl::BYTE as DataFormat;

#[no_mangle]
pub static DataFormat_U16: DataFormat = gl::UNSIGNED_SHORT as DataFormat;

#[no_mangle]
pub static DataFormat_I16: DataFormat = gl::SHORT as DataFormat;

#[no_mangle]
pub static DataFormat_U32: DataFormat = gl::UNSIGNED_INT as DataFormat;

#[no_mangle]
pub static DataFormat_I32: DataFormat = gl::INT as DataFormat;

#[no_mangle]
pub static DataFormat_Float: DataFormat = gl::FLOAT as DataFormat;

#[no_mangle]
pub extern "C" fn DataFormat_GetSize(this: DataFormat) -> i32 {
    if this == DataFormat_U8 || this == DataFormat_I8 {
        1
    } else if this == DataFormat_U16 || this == DataFormat_I16 {
        2
    } else if this == DataFormat_U32 || this == DataFormat_I32 || this == DataFormat_Float {
        4
    } else {
        0
    }
}
