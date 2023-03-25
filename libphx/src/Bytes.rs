use crate::internal::Memory::*;
use crate::Common::*;
use crate::Common::*;
use crate::File::*;
use crate::Math::Vec3;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use libc;
use std::ffi::CString;
use std::io::Write;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Bytes {
    pub size: u32,
    pub cursor: u32,
    pub data: libc::c_char,
}

impl Bytes {
    fn to_slice(&self) -> &[u8] {
        return unsafe {
            std::slice::from_raw_parts(&self.data as *const i8 as *const u8, self.size as usize)
        };
    }

    fn to_slice_mut(&mut self) -> &mut [u8] {
        return unsafe {
            std::slice::from_raw_parts_mut(&mut self.data as *mut i8 as *mut u8, self.size as usize)
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Create(size: u32) -> *mut Bytes {
    let this: *mut Bytes = MemAlloc(
        2_usize
            .wrapping_mul(std::mem::size_of::<u32>())
            .wrapping_add(size as usize),
    ) as *mut Bytes;
    (*this).size = size;
    (*this).cursor = 0;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_FromData(data: *const libc::c_void, len: u32) -> *mut Bytes {
    let this: *mut Bytes = Bytes_Create(len);
    Bytes_Write(this, data, len);
    Bytes_Rewind(this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Load(path: *const libc::c_char) -> *mut Bytes {
    let this: *mut Bytes = File_ReadBytes(path);
    if this.is_null() {
        Fatal(c_str!("Bytes_Load: Failed to read file '%s'"), path);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Free(this: *mut Bytes) {
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_GetData(this: *mut Bytes) -> *mut libc::c_void {
    &mut (*this).data as *mut libc::c_char as *mut _
}

#[no_mangle]
pub extern "C" fn Bytes_GetSize(this: *mut Bytes) -> u32 {
    unsafe { (*this).size }
}

#[no_mangle]
pub extern "C" fn Bytes_Compress(bytes: *mut Bytes) -> *mut Bytes {
    let input = unsafe { (*bytes).to_slice() };

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    if let Err(e) = encoder.write_all(input) {
        unsafe {
            let str = CString::new(e.to_string()).unwrap();
            Fatal(
                c_str!("Bytes_Compress: Encoding failed: %s"),
                str.as_ptr() as *const libc::c_char,
            );
        }
    }

    let result = encoder.finish().unwrap();
    unsafe { Bytes_FromData(result.as_ptr() as *const _, result.len() as u32) }
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Decompress(bytes: *mut Bytes) -> *mut Bytes {
    let input = unsafe { (*bytes).to_slice() };

    let mut decoder = ZlibDecoder::new(Vec::new());
    if let Err(e) = decoder.write_all(input) {
        unsafe {
            let str = CString::new(e.to_string()).unwrap();
            Fatal(
                c_str!("Bytes_Decompress: Decoding failed: %s"),
                str.as_ptr() as *const libc::c_char,
            );
        }
    }

    let result = decoder.finish().unwrap();
    unsafe { Bytes_FromData(result.as_ptr() as *const _, result.len() as u32) }
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_GetCursor(this: *mut Bytes) -> u32 {
    (*this).cursor
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Rewind(this: *mut Bytes) {
    (*this).cursor = 0;
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_SetCursor(this: *mut Bytes, cursor: u32) {
    (*this).cursor = cursor;
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Read(this: *mut Bytes, data: *mut libc::c_void, len: u32) {
    MemCpy(
        data,
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *const _,
        len as usize,
    );
    (*this).cursor = (*this).cursor.wrapping_add(len);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Write(this: *mut Bytes, data: *const libc::c_void, len: u32) {
    MemCpy(
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut _,
        data,
        len as usize,
    );
    (*this).cursor = (*this).cursor.wrapping_add(len);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteStr(this: *mut Bytes, data: *const libc::c_char) {
    let len: usize = StrLen(data);
    MemCpy(
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut _,
        data as *const _,
        len,
    );
    (*this).cursor = (*this).cursor.wrapping_add(len as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU64(this: *mut Bytes) -> u64 {
    let value: u64 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u64);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u64>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI8(this: *mut Bytes) -> i8 {
    let value: i8 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i8);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i8>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI8(this: *mut Bytes, value: i8) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i8) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i8>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI16(this: *mut Bytes, value: i16) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i16) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i16>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU8(this: *mut Bytes) -> u8 {
    let value: u8 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u8);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u8>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI32(this: *mut Bytes, value: i32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i32) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i32>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI64(this: *mut Bytes, value: i64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i64) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i64>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteF32(this: *mut Bytes, value: f32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut f32) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<f32>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU16(this: *mut Bytes, value: u16) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u16) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u16>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU32(this: *mut Bytes, value: u32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u32) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u32>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU8(this: *mut Bytes, value: u8) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u8) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u8>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadF32(this: *mut Bytes) -> f32 {
    let value: f32 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut f32);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<f32>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU16(this: *mut Bytes) -> u16 {
    let value: u16 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u16);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u16>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU32(this: *mut Bytes) -> u32 {
    let value: u32 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u32);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u32>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI64(this: *mut Bytes) -> i64 {
    let value: i64 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i64);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i64>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadF64(this: *mut Bytes) -> f64 {
    let value: f64 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut f64);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<f64>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU64(this: *mut Bytes, value: u64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut u64) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<u64>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI16(this: *mut Bytes) -> i16 {
    let value: i16 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i16);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i16>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI32(this: *mut Bytes) -> i32 {
    let value: i32 =
        *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut i32);
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<i32>() as libc::c_ulong as u32);
    value
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteF64(this: *mut Bytes, value: f64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize) as *mut f64) = value;
    (*this).cursor = (*this)
        .cursor
        .wrapping_add(std::mem::size_of::<f64>() as libc::c_ulong as u32);
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Print(this: *mut Bytes) {
    libc::printf(c_str!("%d bytes:\n"), (*this).size);
    let mut i: u32 = 0;
    while i < (*this).size {
        libc::putchar(*(&mut (*this).data as *mut libc::c_char).offset(i as isize) as i32);
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Bytes_Save(this: *mut Bytes, path: *const libc::c_char) {
    let file: *mut File = File_Create(path);
    if file.is_null() {
        Fatal(
            c_str!("Bytes_Save: Failed to open file '%s' for writing"),
            path,
        );
    }
    File_Write(
        file,
        &mut (*this).data as *mut libc::c_char as *const _,
        (*this).size,
    );
    File_Close(file);
}
