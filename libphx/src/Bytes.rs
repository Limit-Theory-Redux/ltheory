use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type File;
    fn Fatal(_: cstr, _: ...);
    fn File_Create(path: cstr) -> *mut File;
    fn File_Close(_: *mut File);
    fn File_ReadBytes(path: cstr) -> *mut Bytes;
    fn File_Write(_: *mut File, data: *const libc::c_void, len: u32);
    fn putchar(_: i32) -> i32;
    fn printf(_: *const libc::c_char, _: ...) -> i32;
    fn LZ4_versionNumber() -> i32;
    fn LZ4_compress_default(
        src: *const libc::c_char,
        dst: *mut libc::c_char,
        srcSize: i32,
        dstCapacity: i32,
    ) -> i32;
    fn LZ4_decompress_fast(
        src: *const libc::c_char,
        dst: *mut libc::c_char,
        originalSize: i32,
    ) -> i32;
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Bytes {
    pub size: u32,
    pub cursor: u32,
    pub data: libc::c_char,
}


unsafe extern "C" fn Bytes_CheckLZ4Version() {
    let mut vLink: i32 = LZ4_versionNumber()
        / (100 as i32 * 100 as i32);
    let mut vCompile: i32 = (1 as i32 * 100 as i32
        * 100 as i32 + 9 as i32 * 100 as i32 + 4 as i32)
        / (100 as i32 * 100 as i32);
    if vLink != vCompile {
        Fatal(
            b"Bytes_CheckLZ4Version: Linked against incompatible major version of liblz4: Compiled (Major): %d, Linked (Major): %d\0"
                as *const u8 as *const libc::c_char,
            vCompile,
            vLink,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Create(mut size: u32) -> *mut Bytes {
    let mut this: *mut Bytes = MemAlloc(
        (2 as usize)
            .wrapping_mul(::core::mem::size_of::<u32>())
            .wrapping_add(size as usize),
    ) as *mut Bytes;
    (*this).size = size;
    (*this).cursor = 0 as i32 as u32;
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_FromData(
    mut data: *const libc::c_void,
    mut len: u32,
) -> *mut Bytes {
    let mut this: *mut Bytes = Bytes_Create(len);
    Bytes_Write(this, data, len);
    Bytes_Rewind(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Load(mut path: cstr) -> *mut Bytes {
    let mut this: *mut Bytes = File_ReadBytes(path);
    if this.is_null() {
        Fatal(
            b"Bytes_Load: Failed to read file '%s'\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Free(mut this: *mut Bytes) {
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_GetData(mut this: *mut Bytes) -> *mut libc::c_void {
    return &mut (*this).data as *mut libc::c_char as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_GetSize(mut this: *mut Bytes) -> u32 {
    return (*this).size;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Compress(mut bytes: *mut Bytes) -> *mut Bytes {
    Bytes_CheckLZ4Version();
    let mut input: *mut libc::c_char = Bytes_GetData(bytes) as *mut libc::c_char;
    let mut inputLen: u32 = Bytes_GetSize(bytes);
    let mut header: u32 = inputLen;
    let mut headerLen: u32 = ::core::mem::size_of::<u32>() as libc::c_ulong
        as u32;
    if inputLen > 0x7e000000 as i32 as u32
        || inputLen > (4294967295 as u32).wrapping_sub(headerLen)
    {
        Fatal(
            b"Bytes_Compress: Input is too large to compress.\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut bufferLen: u32 = inputLen.wrapping_add(headerLen);
    let mut buffer: *mut libc::c_char = MemAlloc(
        (::core::mem::size_of::<libc::c_char>())
            .wrapping_mul(bufferLen as usize),
    ) as *mut libc::c_char;
    *(buffer as *mut u32) = header;
    let mut resultLen: u32 = LZ4_compress_default(
        input,
        buffer.offset(headerLen as isize),
        inputLen as i32,
        bufferLen.wrapping_sub(headerLen) as i32,
    ) as u32;
    if resultLen == 0 as i32 as u32 {
        Fatal(
            b"Bytes_Compress: LZ4 failed to compress.\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut result: *mut Bytes = Bytes_FromData(
        buffer as *const libc::c_void,
        resultLen.wrapping_add(headerLen),
    );
    MemFree(buffer as *const libc::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Decompress(mut bytes: *mut Bytes) -> *mut Bytes {
    Bytes_CheckLZ4Version();
    let mut input: *mut libc::c_char = Bytes_GetData(bytes) as *mut libc::c_char;
    let mut inputLen: u32 = Bytes_GetSize(bytes);
    let mut header: u32 = *(input as *mut u32);
    let mut headerLen: u32 = ::core::mem::size_of::<u32>() as libc::c_ulong
        as u32;
    let mut bufferLen: u32 = header;
    let mut buffer: *mut libc::c_char = MemAlloc(
        (::core::mem::size_of::<libc::c_char>())
            .wrapping_mul(bufferLen as usize),
    ) as *mut libc::c_char;
    if inputLen < headerLen {
        Fatal(
            b"Bytes_Decompress: Input is smaller than the header size. Data is likely corrupted.\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut resultLen: i32 = LZ4_decompress_fast(
        input.offset(headerLen as isize),
        buffer,
        bufferLen as i32,
    );
    if resultLen < 0 as i32 {
        Fatal(
            b"Bytes_Decompress: LZ4 failed with return value: %i\0" as *const u8
                as *const libc::c_char,
            resultLen,
        );
    }
    if (resultLen as u32).wrapping_add(headerLen) != inputLen {
        Fatal(
            b"Bytes_Decompress: Decompressed length does not match expected result.Expected: %u, Actual: %u\0"
                as *const u8 as *const libc::c_char,
            inputLen.wrapping_sub(headerLen),
            resultLen,
        );
    }
    let mut result: *mut Bytes = Bytes_FromData(
        buffer as *const libc::c_void,
        bufferLen,
    );
    MemFree(buffer as *const libc::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_GetCursor(mut this: *mut Bytes) -> u32 {
    return (*this).cursor;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Rewind(mut this: *mut Bytes) {
    (*this).cursor = 0 as i32 as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_SetCursor(mut this: *mut Bytes, mut cursor: u32) {
    (*this).cursor = cursor;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Read(
    mut this: *mut Bytes,
    mut data: *mut libc::c_void,
    mut len: u32,
) {
    MemCpy(
        data,
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
            as *const libc::c_void,
        len as usize,
    );
    (*this)
        .cursor = ((*this).cursor as u32).wrapping_add(len) as u32
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Write(
    mut this: *mut Bytes,
    mut data: *const libc::c_void,
    mut len: u32,
) {
    MemCpy(
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
            as *mut libc::c_void,
        data,
        len as usize,
    );
    (*this)
        .cursor = ((*this).cursor as u32).wrapping_add(len) as u32
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteStr(mut this: *mut Bytes, mut data: cstr) {
    let mut len: usize = StrLen(data);
    MemCpy(
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
            as *mut libc::c_void,
        data as *const libc::c_void,
        len as usize,
    );
    (*this)
        .cursor = ((*this).cursor as u32).wrapping_add(len as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU64(mut this: *mut Bytes) -> u64 {
    let mut value: u64 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut u64);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u64>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI8(mut this: *mut Bytes) -> i8 {
    let mut value: i8 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut i8);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i8>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI8(mut this: *mut Bytes, mut value: i8) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut i8) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i8>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI16(mut this: *mut Bytes, mut value: i16) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut i16) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i16>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU8(mut this: *mut Bytes) -> u8 {
    let mut value: u8 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut u8);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u8>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI32(mut this: *mut Bytes, mut value: i32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut i32) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i32>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI64(mut this: *mut Bytes, mut value: i64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut i64) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i64>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteF32(
    mut this: *mut Bytes,
    mut value: f32,
) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut f32) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<f32>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU16(mut this: *mut Bytes, mut value: u16) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut u16) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u16>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU32(mut this: *mut Bytes, mut value: u32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut u32) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u32>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU8(mut this: *mut Bytes, mut value: u8) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut u8) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u8>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadF32(mut this: *mut Bytes) -> f32 {
    let mut value: f32 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut f32);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<f32>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU16(mut this: *mut Bytes) -> u16 {
    let mut value: u16 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut u16);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u16>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU32(mut this: *mut Bytes) -> u32 {
    let mut value: u32 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut u32);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u32>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI64(mut this: *mut Bytes) -> i64 {
    let mut value: i64 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut i64);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i64>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadF64(mut this: *mut Bytes) -> f64 {
    let mut value: f64 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut f64);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(
            ::core::mem::size_of::<f64>() as libc::c_ulong as u32,
        ) as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU64(mut this: *mut Bytes, mut value: u64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut u64) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<u64>() as libc::c_ulong as u32)
        as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI16(mut this: *mut Bytes) -> i16 {
    let mut value: i16 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut i16);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i16>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI32(mut this: *mut Bytes) -> i32 {
    let mut value: i32 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut i32);
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(::core::mem::size_of::<i32>() as libc::c_ulong as u32)
        as u32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteF64(
    mut this: *mut Bytes,
    mut value: f64,
) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut f64) = value;
    (*this)
        .cursor = ((*this).cursor as u32)
        .wrapping_add(
            ::core::mem::size_of::<f64>() as libc::c_ulong as u32,
        ) as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Print(mut this: *mut Bytes) {
    printf(b"%d bytes:\n\0" as *const u8 as *const libc::c_char, (*this).size);
    let mut i: u32 = 0 as i32 as u32;
    while i < (*this).size {
        putchar(
            *(&mut (*this).data as *mut libc::c_char).offset(i as isize) as i32,
        );
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Save(mut this: *mut Bytes, mut path: cstr) {
    let mut file: *mut File = File_Create(path);
    if file.is_null() {
        Fatal(
            b"Bytes_Save: Failed to open file '%s' for writing\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    File_Write(
        file,
        &mut (*this).data as *mut libc::c_char as *const libc::c_void,
        (*this).size,
    );
    File_Close(file);
}
