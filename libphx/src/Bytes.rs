use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type File;
    fn Fatal(_: cstr, _: ...);
    fn File_Create(path: cstr) -> *mut File;
    fn File_Close(_: *mut File);
    fn File_ReadBytes(path: cstr) -> *mut Bytes;
    fn File_Write(_: *mut File, data: *const libc::c_void, len: uint32);
    fn putchar(_: libc::c_int) -> libc::c_int;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn LZ4_versionNumber() -> libc::c_int;
    fn LZ4_compress_default(
        src: *const libc::c_char,
        dst: *mut libc::c_char,
        srcSize: libc::c_int,
        dstCapacity: libc::c_int,
    ) -> libc::c_int;
    fn LZ4_decompress_fast(
        src: *const libc::c_char,
        dst: *mut libc::c_char,
        originalSize: libc::c_int,
    ) -> libc::c_int;
}
pub type int8_t = libc::c_schar;
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type int64_t = libc::c_longlong;
pub type uint8_t = libc::c_uchar;
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type cstr = *const libc::c_char;
pub type int8 = int8_t;
pub type int16 = int16_t;
pub type int32 = int32_t;
pub type int64 = int64_t;
pub type uint8 = uint8_t;
pub type uint16 = uint16_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Bytes {
    pub size: uint32,
    pub cursor: uint32,
    pub data: libc::c_char,
}


unsafe extern "C" fn Bytes_CheckLZ4Version() {
    let mut vLink: libc::c_int = LZ4_versionNumber()
        / (100 as libc::c_int * 100 as libc::c_int);
    let mut vCompile: libc::c_int = (1 as libc::c_int * 100 as libc::c_int
        * 100 as libc::c_int + 9 as libc::c_int * 100 as libc::c_int + 4 as libc::c_int)
        / (100 as libc::c_int * 100 as libc::c_int);
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
pub unsafe extern "C" fn Bytes_Create(mut size: uint32) -> *mut Bytes {
    let mut this: *mut Bytes = MemAlloc(
        (2 as usize)
            .wrapping_mul(::core::mem::size_of::<uint32>())
            .wrapping_add(size as usize),
    ) as *mut Bytes;
    (*this).size = size;
    (*this).cursor = 0 as libc::c_int as uint32;
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_FromData(
    mut data: *const libc::c_void,
    mut len: uint32,
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
pub unsafe extern "C" fn Bytes_GetSize(mut this: *mut Bytes) -> uint32 {
    return (*this).size;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Compress(mut bytes: *mut Bytes) -> *mut Bytes {
    Bytes_CheckLZ4Version();
    let mut input: *mut libc::c_char = Bytes_GetData(bytes) as *mut libc::c_char;
    let mut inputLen: uint32 = Bytes_GetSize(bytes);
    let mut header: uint32 = inputLen;
    let mut headerLen: uint32 = ::core::mem::size_of::<uint32>() as libc::c_ulong
        as uint32;
    if inputLen > 0x7e000000 as libc::c_int as libc::c_uint
        || inputLen > (4294967295 as libc::c_uint).wrapping_sub(headerLen)
    {
        Fatal(
            b"Bytes_Compress: Input is too large to compress.\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut bufferLen: uint32 = inputLen.wrapping_add(headerLen);
    let mut buffer: *mut libc::c_char = MemAlloc(
        (::core::mem::size_of::<libc::c_char>())
            .wrapping_mul(bufferLen as usize),
    ) as *mut libc::c_char;
    *(buffer as *mut uint32) = header;
    let mut resultLen: uint32 = LZ4_compress_default(
        input,
        buffer.offset(headerLen as isize),
        inputLen as libc::c_int,
        bufferLen.wrapping_sub(headerLen) as libc::c_int,
    ) as uint32;
    if resultLen == 0 as libc::c_int as libc::c_uint {
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
    let mut inputLen: uint32 = Bytes_GetSize(bytes);
    let mut header: uint32 = *(input as *mut uint32);
    let mut headerLen: uint32 = ::core::mem::size_of::<uint32>() as libc::c_ulong
        as uint32;
    let mut bufferLen: uint32 = header;
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
    let mut resultLen: int32 = LZ4_decompress_fast(
        input.offset(headerLen as isize),
        buffer,
        bufferLen as libc::c_int,
    );
    if resultLen < 0 as libc::c_int {
        Fatal(
            b"Bytes_Decompress: LZ4 failed with return value: %i\0" as *const u8
                as *const libc::c_char,
            resultLen,
        );
    }
    if (resultLen as libc::c_uint).wrapping_add(headerLen) != inputLen {
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
pub unsafe extern "C" fn Bytes_GetCursor(mut this: *mut Bytes) -> uint32 {
    return (*this).cursor;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Rewind(mut this: *mut Bytes) {
    (*this).cursor = 0 as libc::c_int as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_SetCursor(mut this: *mut Bytes, mut cursor: uint32) {
    (*this).cursor = cursor;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Read(
    mut this: *mut Bytes,
    mut data: *mut libc::c_void,
    mut len: uint32,
) {
    MemCpy(
        data,
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
            as *const libc::c_void,
        len as usize,
    );
    (*this)
        .cursor = ((*this).cursor as libc::c_uint).wrapping_add(len) as uint32
        as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Write(
    mut this: *mut Bytes,
    mut data: *const libc::c_void,
    mut len: uint32,
) {
    MemCpy(
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
            as *mut libc::c_void,
        data,
        len as usize,
    );
    (*this)
        .cursor = ((*this).cursor as libc::c_uint).wrapping_add(len) as uint32
        as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteStr(mut this: *mut Bytes, mut data: cstr) {
    let mut len: libc::size_t = StrLen(data);
    MemCpy(
        (&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
            as *mut libc::c_void,
        data as *const libc::c_void,
        len as usize,
    );
    (*this)
        .cursor = ((*this).cursor as libc::c_uint).wrapping_add(len as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU64(mut this: *mut Bytes) -> uint64 {
    let mut value: uint64 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut uint64);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint64>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI8(mut this: *mut Bytes) -> int8 {
    let mut value: int8 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut int8);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int8>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI8(mut this: *mut Bytes, mut value: int8) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut int8) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int8>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI16(mut this: *mut Bytes, mut value: int16) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut int16) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int16>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU8(mut this: *mut Bytes) -> uint8 {
    let mut value: uint8 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut uint8);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint8>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI32(mut this: *mut Bytes, mut value: int32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut int32) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int32>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteI64(mut this: *mut Bytes, mut value: int64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut int64) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int64>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteF32(
    mut this: *mut Bytes,
    mut value: f32,
) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut f32) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<f32>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU16(mut this: *mut Bytes, mut value: uint16) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut uint16) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint16>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU32(mut this: *mut Bytes, mut value: uint32) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut uint32) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint32>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU8(mut this: *mut Bytes, mut value: uint8) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut uint8) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint8>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadF32(mut this: *mut Bytes) -> f32 {
    let mut value: f32 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut f32);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<f32>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU16(mut this: *mut Bytes) -> uint16 {
    let mut value: uint16 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut uint16);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint16>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadU32(mut this: *mut Bytes) -> uint32 {
    let mut value: uint32 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut uint32);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint32>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI64(mut this: *mut Bytes) -> int64 {
    let mut value: int64 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut int64);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int64>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadF64(mut this: *mut Bytes) -> f64 {
    let mut value: f64 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut f64);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(
            ::core::mem::size_of::<f64>() as libc::c_ulong as uint32,
        ) as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_WriteU64(mut this: *mut Bytes, mut value: uint64) {
    *((&mut (*this).data as *mut libc::c_char).offset((*this).cursor as isize)
        as *mut uint64) = value;
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<uint64>() as libc::c_ulong as uint32)
        as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI16(mut this: *mut Bytes) -> int16 {
    let mut value: int16 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut int16);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int16>() as libc::c_ulong as uint32)
        as uint32 as uint32;
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_ReadI32(mut this: *mut Bytes) -> int32 {
    let mut value: int32 = *((&mut (*this).data as *mut libc::c_char)
        .offset((*this).cursor as isize) as *mut int32);
    (*this)
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(::core::mem::size_of::<int32>() as libc::c_ulong as uint32)
        as uint32 as uint32;
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
        .cursor = ((*this).cursor as libc::c_uint)
        .wrapping_add(
            ::core::mem::size_of::<f64>() as libc::c_ulong as uint32,
        ) as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn Bytes_Print(mut this: *mut Bytes) {
    printf(b"%d bytes:\n\0" as *const u8 as *const libc::c_char, (*this).size);
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*this).size {
        putchar(
            *(&mut (*this).data as *mut libc::c_char).offset(i as isize) as libc::c_int,
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
