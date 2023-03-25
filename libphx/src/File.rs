use crate::internal::Memory::*;
use crate::Bytes::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
use std::fs;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct File {
    pub handle: *mut libc::FILE,
}

#[no_mangle]
pub unsafe extern "C" fn File_Exists(path: *const libc::c_char) -> bool {
    let f: *mut libc::FILE = libc::fopen(path, c_str!("rb"));
    if !f.is_null() {
        libc::fclose(f);
        return true;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn File_IsDir(path: *const libc::c_char) -> bool {
    let meta = fs::metadata(std::ffi::CStr::from_ptr(path).to_str().unwrap()).unwrap();
    meta.is_dir()
}

unsafe extern "C" fn File_OpenMode(
    path: *const libc::c_char,
    mode: *const libc::c_char,
) -> *mut File {
    let handle: *mut libc::FILE = libc::fopen(path, mode);
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let this = MemNew!(File);
    (*this).handle = handle;
    this
}

#[no_mangle]
pub unsafe extern "C" fn File_Create(path: *const libc::c_char) -> *mut File {
    File_OpenMode(path, c_str!("wb"))
}

#[no_mangle]
pub unsafe extern "C" fn File_Open(path: *const libc::c_char) -> *mut File {
    File_OpenMode(path, c_str!("ab"))
}

#[no_mangle]
pub unsafe extern "C" fn File_Close(this: *mut File) {
    libc::fclose((*this).handle);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadBytes(path: *const libc::c_char) -> *mut Bytes {
    let file: *mut libc::FILE = libc::fopen(path, c_str!("rb"));
    if file.is_null() {
        return std::ptr::null_mut();
    }
    libc::fseek(file, 0 as libc::c_long, libc::SEEK_END);
    let size: i64 = libc::ftell(file);
    if size == 0 {
        return std::ptr::null_mut();
    }
    if size < 0 {
        Fatal(c_str!("File_Read: failed to get size of file '%s'"), path);
    }
    libc::rewind(file);
    if size > sdl2_sys::UINT32_MAX as i64 {
        Fatal(
            c_str!("File_Read: filesize of '%s' exceeds 32-bit capacity limit"),
            path,
        );
    }
    let buffer: *mut Bytes = Bytes_Create(size as u32);
    let result: usize = libc::fread(Bytes_GetData(buffer), size as usize, 1, file);
    if result != 1 {
        Fatal(
            c_str!("File_Read: failed to read correct number of bytes from '%s'"),
            path,
        );
    }
    libc::fclose(file);
    buffer
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadCstr(path: *const libc::c_char) -> *const libc::c_char {
    let file: *mut libc::FILE = libc::fopen(path, c_str!("rb"));
    if file.is_null() {
        return std::ptr::null();
    }
    libc::fseek(file, 0 as libc::c_long, 2);
    let size: i64 = libc::ftell(file);
    if size == 0 {
        return std::ptr::null();
    }
    if size < 0 {
        Fatal(
            c_str!("File_ReadAscii: failed to get size of file '%s'"),
            path,
        );
    }
    libc::rewind(file);
    let buffer: *mut libc::c_char = MemAlloc(
        (std::mem::size_of::<libc::c_char>()).wrapping_mul((size as usize).wrapping_add(1)),
    ) as *mut libc::c_char;
    let result: usize = libc::fread(buffer as *mut _, size as usize, 1, file);
    if result != 1 {
        Fatal(
            c_str!("File_Read: failed to read correct number of bytes from '%s'"),
            path,
        );
    }
    libc::fclose(file);
    *buffer.offset(size as isize) = 0 as libc::c_char;
    buffer as *const libc::c_char
}

#[no_mangle]
pub unsafe extern "C" fn File_Size(path: *const libc::c_char) -> i64 {
    let file: *mut libc::FILE = libc::fopen(path, c_str!("rb"));
    if file.is_null() {
        return 0;
    }
    libc::fseek(file, 0 as libc::c_long, 2);
    let size: i64 = libc::ftell(file);
    libc::fclose(file);
    size
}

#[no_mangle]
pub unsafe extern "C" fn File_Read(this: *mut File, data: *mut libc::c_void, len: u32) {
    libc::fread(data, len as usize, 1, (*this).handle);
}

#[no_mangle]
pub unsafe extern "C" fn File_Write(this: *mut File, data: *const libc::c_void, len: u32) {
    libc::fwrite(data, len as usize, 1, (*this).handle);
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteStr(this: *mut File, data: *const libc::c_char) {
    libc::fwrite(data as *const _, StrLen(data), 1, (*this).handle);
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadI64(this: *mut File) -> i64 {
    let value: i64 = 0;
    libc::fread(
        &value as *const i64 as *mut _,
        std::mem::size_of::<i64>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteU64(this: *mut File, value: u64) {
    libc::fwrite(
        &value as *const u64 as *const _,
        std::mem::size_of::<u64>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteI8(this: *mut File, value: i8) {
    libc::fwrite(
        &value as *const i8 as *const _,
        std::mem::size_of::<i8>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteU32(this: *mut File, value: u32) {
    libc::fwrite(
        &value as *const u32 as *const _,
        std::mem::size_of::<u32>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteI16(this: *mut File, value: i16) {
    libc::fwrite(
        &value as *const i16 as *const _,
        std::mem::size_of::<i16>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadU64(this: *mut File) -> u64 {
    let value: u64 = 0;
    libc::fread(
        &value as *const u64 as *mut _,
        std::mem::size_of::<u64>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadI32(this: *mut File) -> i32 {
    let value: i32 = 0;
    libc::fread(
        &value as *const i32 as *mut _,
        std::mem::size_of::<i32>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadU32(this: *mut File) -> u32 {
    let value: u32 = 0;
    libc::fread(
        &value as *const u32 as *mut _,
        std::mem::size_of::<u32>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteU16(this: *mut File, value: u16) {
    libc::fwrite(
        &value as *const u16 as *const _,
        std::mem::size_of::<u16>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadU8(this: *mut File) -> u8 {
    let value: u8 = 0;
    libc::fread(
        &value as *const u8 as *mut _,
        std::mem::size_of::<u8>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadU16(this: *mut File) -> u16 {
    let value: u16 = 0;
    libc::fread(
        &value as *const u16 as *mut _,
        std::mem::size_of::<u16>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadI16(this: *mut File) -> i16 {
    let value: i16 = 0;
    libc::fread(
        &value as *const i16 as *mut _,
        std::mem::size_of::<i16>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadF32(this: *mut File) -> f32 {
    let value: f32 = 0.;
    libc::fread(
        &value as *const f32 as *mut _,
        std::mem::size_of::<f32>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadF64(this: *mut File) -> f64 {
    let value: f64 = 0.;
    libc::fread(
        &value as *const f64 as *mut _,
        std::mem::size_of::<f64>(),
        1,
        (*this).handle,
    );
    value
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteU8(this: *mut File, value: u8) {
    libc::fwrite(
        &value as *const u8 as *const _,
        std::mem::size_of::<u8>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteI32(this: *mut File, value: i32) {
    libc::fwrite(
        &value as *const i32 as *const _,
        std::mem::size_of::<i32>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteI64(this: *mut File, value: i64) {
    libc::fwrite(
        &value as *const i64 as *const _,
        std::mem::size_of::<i64>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteF32(this: *mut File, value: f32) {
    libc::fwrite(
        &value as *const f32 as *const _,
        std::mem::size_of::<f32>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_WriteF64(this: *mut File, value: f64) {
    libc::fwrite(
        &value as *const f64 as *const _,
        std::mem::size_of::<f64>(),
        1,
        (*this).handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn File_ReadI8(this: *mut File) -> i8 {
    let value: i8 = 0;
    libc::fread(
        &value as *const i8 as *mut _,
        std::mem::size_of::<i8>(),
        1,
        (*this).handle,
    );
    value
}
