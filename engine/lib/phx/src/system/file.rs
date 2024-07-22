use std::fs;
use std::io::{Read, Write};

use internal::{static_string, ConvertIntoString};

use super::*;

#[repr(C)]
pub struct File {
    pub file: fs::File,
}

#[no_mangle]
pub extern "C" fn File_Exists(path: *const libc::c_char) -> bool {
    file_exists(path.as_str())
}

pub fn file_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn File_IsDir(path: *const libc::c_char) -> bool {
    match fs::metadata(path.as_str()) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn File_Create(path: *const libc::c_char) -> Option<Box<File>> {
    let file = fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path.as_str())
        .ok()?;
    Some(Box::new(File { file }))
}

#[no_mangle]
pub extern "C" fn File_Open(path: *const libc::c_char) -> Option<Box<File>> {
    let file = fs::File::options()
        .create(true)
        .append(true)
        .open(path.as_str())
        .ok()?;
    Some(Box::new(File { file }))
}

#[no_mangle]
pub extern "C" fn File_Close(_: Option<Box<File>>) {
    // 'this' will get dropped here, as we're moving "Box<File>" into this function and it's falling out of scope.
}

#[no_mangle]
pub extern "C" fn File_ReadBytes(path: *const libc::c_char) -> *mut Bytes {
    match fs::read(path.as_str()) {
        Ok(bytes) => Bytes_FromVec(bytes),
        _ => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn File_ReadCstr(path: *const libc::c_char) -> *const libc::c_char {
    file_read_cstr(path.as_str())
        .map(|val| static_string!(val))
        .unwrap_or(std::ptr::null())
}

pub fn file_read_cstr(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

#[no_mangle]
pub extern "C" fn File_Size(path: *const libc::c_char) -> i64 {
    if let Ok(file) = fs::File::open(path.as_str()) {
        if let Ok(metadata) = file.metadata() {
            return metadata.len() as i64;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn File_Read(this: &mut File, data: *mut libc::c_void, len: u32) {
    let buffer = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, len as usize) };
    let _ = this.file.read(buffer);
}

#[no_mangle]
pub extern "C" fn File_Write(this: &mut File, data: *const libc::c_void, len: u32) {
    let buffer = unsafe { std::slice::from_raw_parts(data as *mut u8, len as usize) };
    let _ = this.file.write(buffer);
}

#[no_mangle]
pub extern "C" fn File_WriteStr(this: &mut File, data: *const libc::c_char) {
    let data_str = data.as_str();
    let buffer = data_str.as_bytes();

    let _ = this.file.write(buffer);
}

#[no_mangle]
pub extern "C" fn File_ReadU8(this: &mut File) -> u8 {
    let mut buf = [0u8; std::mem::size_of::<u8>()];
    if this.file.read_exact(&mut buf).is_ok() {
        u8::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadU16(this: &mut File) -> u16 {
    let mut buf = [0u8; std::mem::size_of::<u16>()];
    if this.file.read_exact(&mut buf).is_ok() {
        u16::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadU32(this: &mut File) -> u32 {
    let mut buf = [0u8; std::mem::size_of::<u32>()];
    if this.file.read_exact(&mut buf).is_ok() {
        u32::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadU64(this: &mut File) -> u64 {
    let mut buf = [0u8; std::mem::size_of::<u64>()];
    if this.file.read_exact(&mut buf).is_ok() {
        u64::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadI8(this: &mut File) -> i8 {
    let mut buf = [0u8; std::mem::size_of::<i8>()];
    if this.file.read_exact(&mut buf).is_ok() {
        i8::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadI16(this: &mut File) -> i16 {
    let mut buf = [0u8; std::mem::size_of::<i16>()];
    if this.file.read_exact(&mut buf).is_ok() {
        i16::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadI32(this: &mut File) -> i32 {
    let mut buf = [0u8; std::mem::size_of::<i32>()];
    if this.file.read_exact(&mut buf).is_ok() {
        i32::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadI64(this: &mut File) -> i64 {
    let mut buf = [0u8; std::mem::size_of::<i64>()];
    if this.file.read_exact(&mut buf).is_ok() {
        i64::from_le_bytes(buf)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadF32(this: &mut File) -> f32 {
    let mut buf = [0u8; std::mem::size_of::<f32>()];
    if this.file.read_exact(&mut buf).is_ok() {
        f32::from_le_bytes(buf)
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn File_ReadF64(this: &mut File) -> f64 {
    let mut buf = [0u8; std::mem::size_of::<f64>()];
    if this.file.read_exact(&mut buf).is_ok() {
        f64::from_le_bytes(buf)
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn File_WriteU8(this: &mut File, value: u8) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteU16(this: &mut File, value: u16) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteU32(this: &mut File, value: u32) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteU64(this: &mut File, value: u64) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteI8(this: &mut File, value: i8) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteI16(this: &mut File, value: i16) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteI32(this: &mut File, value: i32) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteI64(this: &mut File, value: i64) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteF32(this: &mut File, value: f32) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}

#[no_mangle]
pub extern "C" fn File_WriteF64(this: &mut File, value: f64) {
    let _ = this.file.write(value.to_le_bytes().as_slice());
}
