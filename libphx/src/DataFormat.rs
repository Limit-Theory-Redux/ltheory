use crate::GL::gl;

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

// Size in bytes of single element
#[no_mangle]
pub extern "C" fn DataFormat_GetSize(this: DataFormat) -> i32 {
    match this {
        df if df == DataFormat_U8 || df == DataFormat_I8 => 1,
        df if df == DataFormat_U16 || df == DataFormat_I16 => 2,
        df if df == DataFormat_U32 || df == DataFormat_I32 || df == DataFormat_Float => 4,
        _ => 0,
    }
}
