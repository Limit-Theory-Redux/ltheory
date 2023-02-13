use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type _telldir;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn File_IsDir(path: cstr) -> bool;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn closedir(_: *mut DIR) -> libc::c_int;
    fn opendir(_: *const libc::c_char) -> *mut DIR;
    fn readdir(_: *mut DIR) -> *mut dirent;
    fn chdir(_: *const libc::c_char) -> libc::c_int;
    fn getcwd(_: *mut libc::c_char, _: size_t) -> *mut libc::c_char;
    fn rmdir(_: *const libc::c_char) -> libc::c_int;
    fn mkdir(_: *const libc::c_char, _: mode_t) -> libc::c_int;
}
pub type __uint8_t = libc::c_uchar;
pub type __uint16_t = libc::c_ushort;
pub type __uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type __darwin_mode_t = __uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _opaque_pthread_mutex_t {
    pub __sig: libc::c_long,
    pub __opaque: [libc::c_char; 56],
}
pub type __darwin_pthread_mutex_t = _opaque_pthread_mutex_t;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Directory {
    pub handle: *mut DIR,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DIR {
    pub __dd_fd: libc::c_int,
    pub __dd_loc: libc::c_long,
    pub __dd_size: libc::c_long,
    pub __dd_buf: *mut libc::c_char,
    pub __dd_len: libc::c_int,
    pub __dd_seek: libc::c_long,
    pub __padding: libc::c_long,
    pub __dd_flags: libc::c_int,
    pub __dd_lock: __darwin_pthread_mutex_t,
    pub __dd_td: *mut _telldir,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dirent {
    pub d_ino: __uint64_t,
    pub d_seekoff: __uint64_t,
    pub d_reclen: __uint16_t,
    pub d_namlen: __uint16_t,
    pub d_type: __uint8_t,
    pub d_name: [libc::c_char; 1024],
}
pub type mode_t = __darwin_mode_t;

#[inline]
unsafe extern "C" fn StrEqual(mut a: cstr, mut b: cstr) -> bool {
    return strcmp(a, b) == 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Open(mut path: cstr) -> *mut Directory {
    let mut dir: *mut DIR = opendir(path);
    if dir.is_null() {
        return 0 as *mut Directory;
    }
    let mut self_0: *mut Directory = MemAlloc(
        ::core::mem::size_of::<Directory>() as libc::c_ulong,
    ) as *mut Directory;
    (*self_0).handle = dir;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Close(mut self_0: *mut Directory) {
    closedir((*self_0).handle);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Directory_GetNext(mut self_0: *mut Directory) -> cstr {
    loop {
        let mut ent: *mut dirent = readdir((*self_0).handle);
        if ent.is_null() {
            return 0 as cstr;
        }
        if StrEqual(
            ((*ent).d_name).as_mut_ptr() as cstr,
            b".\0" as *const u8 as *const libc::c_char,
        ) as libc::c_int != 0
            || StrEqual(
                ((*ent).d_name).as_mut_ptr() as cstr,
                b"..\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
        {
            continue;
        }
        return ((*ent).d_name).as_mut_ptr() as cstr;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Change(mut cwd: cstr) -> bool {
    return chdir(cwd) == 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Create(mut path: cstr) -> bool {
    mkdir(path, 0o775 as libc::c_int as mode_t);
    return File_IsDir(path);
}
#[no_mangle]
pub unsafe extern "C" fn Directory_GetCurrent() -> cstr {
    static mut buffer: [libc::c_char; 1024] = [0; 1024];
    if !(getcwd(
        buffer.as_mut_ptr(),
        ::core::mem::size_of::<[libc::c_char; 1024]>() as libc::c_ulong,
    ))
        .is_null()
    {
        return 0 as cstr;
    }
    buffer[(::core::mem::size_of::<[libc::c_char; 1024]>())
        .wrapping_sub(1 as usize)] = 0 as libc::c_int as libc::c_char;
    return buffer.as_mut_ptr() as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Remove(mut path: cstr) -> bool {
    return rmdir(path) == 0 as libc::c_int;
}
