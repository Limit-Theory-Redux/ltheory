use ::libc;
use crate::internal::Memory::*;
extern "C" {
    pub type Bytes;
    pub type __sFILEX;
    fn Fatal(_: cstr, _: ...);
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn fclose(_: *mut FILE) -> libc::c_int;
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
    fn ftell(_: *mut FILE) -> libc::c_long;
    fn fread(
        _: *mut libc::c_void,
        _: libc::size_t,
        _: libc::size_t,
        _: *mut FILE,
    ) -> libc::size_t;
    fn rewind(_: *mut FILE);
    fn fseek(_: *mut FILE, _: libc::c_long, _: libc::c_int) -> libc::c_int;
    fn fwrite(
        _: *const libc::c_void,
        _: libc::size_t,
        _: libc::size_t,
        _: *mut FILE,
    ) -> libc::c_ulong;
    fn stat(_: *const libc::c_char, _: *mut stat) -> libc::c_int;
}
pub type int8_t = libc::c_schar;
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type int64_t = libc::c_longlong;
pub type uint8_t = libc::c_uchar;
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __uint16_t = libc::c_ushort;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type __int64_t = libc::c_longlong;
pub type __uint64_t = libc::c_ulonglong;
pub type __darwin_time_t = libc::c_long;
pub type __darwin_blkcnt_t = __int64_t;
pub type __darwin_blksize_t = __int32_t;
pub type __darwin_dev_t = __int32_t;
pub type __darwin_gid_t = __uint32_t;
pub type __darwin_ino64_t = __uint64_t;
pub type __darwin_mode_t = __uint16_t;
pub type __darwin_off_t = __int64_t;
pub type __darwin_uid_t = __uint32_t;
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
pub struct File {
    pub handle: *mut FILE,
}
pub type FILE = __sFILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
    pub _p: *mut libc::c_uchar,
    pub _r: libc::c_int,
    pub _w: libc::c_int,
    pub _flags: libc::c_short,
    pub _file: libc::c_short,
    pub _bf: __sbuf,
    pub _lbfsize: libc::c_int,
    pub _cookie: *mut libc::c_void,
    pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
    pub _read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _seek: Option::<
        unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
    >,
    pub _write: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *const libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _ub: __sbuf,
    pub _extra: *mut __sFILEX,
    pub _ur: libc::c_int,
    pub _ubuf: [libc::c_uchar; 3],
    pub _nbuf: [libc::c_uchar; 1],
    pub _lb: __sbuf,
    pub _blksize: libc::c_int,
    pub _offset: fpos_t,
}
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}
pub type mode_t = __darwin_mode_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stat {
    pub st_dev: dev_t,
    pub st_mode: mode_t,
    pub st_nlink: nlink_t,
    pub st_ino: __darwin_ino64_t,
    pub st_uid: uid_t,
    pub st_gid: gid_t,
    pub st_rdev: dev_t,
    pub st_atimespec: timespec,
    pub st_mtimespec: timespec,
    pub st_ctimespec: timespec,
    pub st_birthtimespec: timespec,
    pub st_size: off_t,
    pub st_blocks: blkcnt_t,
    pub st_blksize: blksize_t,
    pub st_flags: __uint32_t,
    pub st_gen: __uint32_t,
    pub st_lspare: __int32_t,
    pub st_qspare: [__int64_t; 2],
}
pub type blksize_t = __darwin_blksize_t;
pub type blkcnt_t = __darwin_blkcnt_t;
pub type off_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __darwin_time_t,
    pub tv_nsec: libc::c_long,
}
pub type dev_t = __darwin_dev_t;
pub type gid_t = __darwin_gid_t;
pub type uid_t = __darwin_uid_t;
pub type nlink_t = __uint16_t;

#[no_mangle]
pub unsafe extern "C" fn File_Exists(mut path: cstr) -> bool {
    let mut f: *mut FILE = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);
    if !f.is_null() {
        fclose(f);
        return 1 as libc::c_int != 0;
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn File_IsDir(mut path: cstr) -> bool {
    let mut s: stat = stat {
        st_dev: 0,
        st_mode: 0,
        st_nlink: 0,
        st_ino: 0,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        st_atimespec: timespec { tv_sec: 0, tv_nsec: 0 },
        st_mtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
        st_ctimespec: timespec { tv_sec: 0, tv_nsec: 0 },
        st_birthtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
        st_size: 0,
        st_blocks: 0,
        st_blksize: 0,
        st_flags: 0,
        st_gen: 0,
        st_lspare: 0,
        st_qspare: [0; 2],
    };
    return stat(path, &mut s) == 0 as libc::c_int
        && s.st_mode as libc::c_int & 0o40000 as libc::c_int != 0;
}
unsafe extern "C" fn File_OpenMode(mut path: cstr, mut mode: cstr) -> *mut File {
    let mut handle: *mut FILE = fopen(path, mode);
    if handle.is_null() {
        return 0 as *mut File;
    }
    let mut self_0: *mut File = MemAlloc(::core::mem::size_of::<File>())
        as *mut File;
    (*self_0).handle = handle;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn File_Create(mut path: cstr) -> *mut File {
    return File_OpenMode(path, b"wb\0" as *const u8 as *const libc::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn File_Open(mut path: cstr) -> *mut File {
    return File_OpenMode(path, b"ab\0" as *const u8 as *const libc::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn File_Close(mut self_0: *mut File) {
    fclose((*self_0).handle);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadBytes(mut path: cstr) -> *mut Bytes {
    let mut file: *mut FILE = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);
    if file.is_null() {
        return 0 as *mut Bytes;
    }
    fseek(file, 0 as libc::c_int as libc::c_long, 2 as libc::c_int);
    let mut size: int64 = ftell(file) as int64;
    if size == 0 as libc::c_int as libc::c_longlong {
        return 0 as *mut Bytes;
    }
    if size < 0 as libc::c_int as libc::c_longlong {
        Fatal(
            b"File_Read: failed to get size of file '%s'\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    rewind(file);
    if size > 4294967295 as libc::c_uint as libc::c_longlong {
        Fatal(
            b"File_Read: filesize of '%s' exceeds 32-bit capacity limit\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    let mut buffer: *mut Bytes = Bytes_Create(size as uint32);
    let mut result: libc::size_t = fread(
        Bytes_GetData(buffer),
        size as libc::size_t,
        1 as libc::size_t,
        file,
    );
    if result != 1 as libc::size_t {
        Fatal(
            b"File_Read: failed to read correct number of bytes from '%s'\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    fclose(file);
    return buffer;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadCstr(mut path: cstr) -> cstr {
    let mut file: *mut FILE = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);
    if file.is_null() {
        return 0 as cstr;
    }
    fseek(file, 0 as libc::c_int as libc::c_long, 2 as libc::c_int);
    let mut size: int64 = ftell(file) as int64;
    if size == 0 as libc::c_int as libc::c_longlong {
        return 0 as cstr;
    }
    if size < 0 as libc::c_int as libc::c_longlong {
        Fatal(
            b"File_ReadAscii: failed to get size of file '%s'\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    rewind(file);
    let mut buffer: *mut libc::c_char = MemAlloc(
        (::core::mem::size_of::<libc::c_char>())
            .wrapping_mul(
                (size as usize).wrapping_add(1 as usize),
            ),
    ) as *mut libc::c_char;
    let mut result: libc::size_t = fread(
        buffer as *mut libc::c_void,
        size as libc::size_t,
        1 as libc::size_t,
        file,
    );
    if result != 1 as libc::size_t {
        Fatal(
            b"File_Read: failed to read correct number of bytes from '%s'\0" as *const u8
                as *const libc::c_char,
            path,
        );
    }
    fclose(file);
    *buffer.offset(size as isize) = 0 as libc::c_int as libc::c_char;
    return buffer as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn File_Size(mut path: cstr) -> int64 {
    let mut file: *mut FILE = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);
    if file.is_null() {
        return 0 as libc::c_int as int64;
    }
    fseek(file, 0 as libc::c_int as libc::c_long, 2 as libc::c_int);
    let mut size: int64 = ftell(file) as int64;
    fclose(file);
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn File_Read(
    mut self_0: *mut File,
    mut data: *mut libc::c_void,
    mut len: uint32,
) {
    fread(
        data,
        len as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_Write(
    mut self_0: *mut File,
    mut data: *const libc::c_void,
    mut len: uint32,
) {
    fwrite(
        data,
        len as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteStr(mut self_0: *mut File, mut data: cstr) {
    fwrite(
        data as *const libc::c_void,
        StrLen(data),
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadI64(mut self_0: *mut File) -> int64 {
    let mut value: int64 = 0;
    fread(
        &mut value as *mut int64 as *mut libc::c_void,
        ::core::mem::size_of::<int64>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteU64(mut self_0: *mut File, mut value: uint64) {
    fwrite(
        &mut value as *mut uint64 as *const libc::c_void,
        ::core::mem::size_of::<uint64>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteI8(mut self_0: *mut File, mut value: int8) {
    fwrite(
        &mut value as *mut int8 as *const libc::c_void,
        ::core::mem::size_of::<int8>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteU32(mut self_0: *mut File, mut value: uint32) {
    fwrite(
        &mut value as *mut uint32 as *const libc::c_void,
        ::core::mem::size_of::<uint32>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteI16(mut self_0: *mut File, mut value: int16) {
    fwrite(
        &mut value as *mut int16 as *const libc::c_void,
        ::core::mem::size_of::<int16>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadU64(mut self_0: *mut File) -> uint64 {
    let mut value: uint64 = 0;
    fread(
        &mut value as *mut uint64 as *mut libc::c_void,
        ::core::mem::size_of::<uint64>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadI32(mut self_0: *mut File) -> int32 {
    let mut value: int32 = 0;
    fread(
        &mut value as *mut int32 as *mut libc::c_void,
        ::core::mem::size_of::<int32>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadU32(mut self_0: *mut File) -> uint32 {
    let mut value: uint32 = 0;
    fread(
        &mut value as *mut uint32 as *mut libc::c_void,
        ::core::mem::size_of::<uint32>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteU16(mut self_0: *mut File, mut value: uint16) {
    fwrite(
        &mut value as *mut uint16 as *const libc::c_void,
        ::core::mem::size_of::<uint16>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadU8(mut self_0: *mut File) -> uint8 {
    let mut value: uint8 = 0;
    fread(
        &mut value as *mut uint8 as *mut libc::c_void,
        ::core::mem::size_of::<uint8>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadU16(mut self_0: *mut File) -> uint16 {
    let mut value: uint16 = 0;
    fread(
        &mut value as *mut uint16 as *mut libc::c_void,
        ::core::mem::size_of::<uint16>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadI16(mut self_0: *mut File) -> int16 {
    let mut value: int16 = 0;
    fread(
        &mut value as *mut int16 as *mut libc::c_void,
        ::core::mem::size_of::<int16>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadF32(mut self_0: *mut File) -> libc::c_float {
    let mut value: libc::c_float = 0.;
    fread(
        &mut value as *mut libc::c_float as *mut libc::c_void,
        ::core::mem::size_of::<libc::c_float>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadF64(mut self_0: *mut File) -> libc::c_double {
    let mut value: libc::c_double = 0.;
    fread(
        &mut value as *mut libc::c_double as *mut libc::c_void,
        ::core::mem::size_of::<libc::c_double>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteU8(mut self_0: *mut File, mut value: uint8) {
    fwrite(
        &mut value as *mut uint8 as *const libc::c_void,
        ::core::mem::size_of::<uint8>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteI32(mut self_0: *mut File, mut value: int32) {
    fwrite(
        &mut value as *mut int32 as *const libc::c_void,
        ::core::mem::size_of::<int32>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteI64(mut self_0: *mut File, mut value: int64) {
    fwrite(
        &mut value as *mut int64 as *const libc::c_void,
        ::core::mem::size_of::<int64>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteF32(mut self_0: *mut File, mut value: libc::c_float) {
    fwrite(
        &mut value as *mut libc::c_float as *const libc::c_void,
        ::core::mem::size_of::<libc::c_float>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_WriteF64(
    mut self_0: *mut File,
    mut value: libc::c_double,
) {
    fwrite(
        &mut value as *mut libc::c_double as *const libc::c_void,
        ::core::mem::size_of::<libc::c_double>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn File_ReadI8(mut self_0: *mut File) -> int8 {
    let mut value: int8 = 0;
    fread(
        &mut value as *mut int8 as *mut libc::c_void,
        ::core::mem::size_of::<int8>() as libc::size_t,
        1 as libc::size_t,
        (*self_0).handle,
    );
    return value;
}
