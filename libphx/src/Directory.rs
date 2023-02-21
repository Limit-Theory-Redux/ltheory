use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {
    fn File_IsDir(path: cstr) -> bool;
}

pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Directory {
    pub handle: *mut libc::DIR,
}

#[no_mangle]
pub unsafe extern "C" fn Directory_Open(mut path: cstr) -> *mut Directory {
    let mut dir: *mut libc::DIR = libc::opendir(path);
    if dir.is_null() {
        return 0 as *mut Directory;
    }
    let mut this: *mut Directory =
        MemAlloc(::core::mem::size_of::<Directory>() as usize) as *mut Directory;
    (*this).handle = dir;
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Close(mut this: *mut Directory) {
    libc::closedir((*this).handle);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Directory_GetNext(mut this: *mut Directory) -> cstr {
    loop {
        let mut ent: *mut libc::dirent = libc::readdir((*this).handle);
        if ent.is_null() {
            return 0 as cstr;
        }
        if StrEqual(
            ((*ent).d_name).as_mut_ptr() as cstr,
            b".\0" as *const u8 as *const libc::c_char,
        ) as i32
            != 0
            || StrEqual(
                ((*ent).d_name).as_mut_ptr() as cstr,
                b"..\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
        {
            continue;
        }
        return ((*ent).d_name).as_mut_ptr() as cstr;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Change(mut cwd: cstr) -> bool {
    return libc::chdir(cwd) == 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Create(mut path: cstr) -> bool {
    libc::mkdir(path, 0o775 as libc::mode_t);
    return File_IsDir(path);
}
#[no_mangle]
pub unsafe extern "C" fn Directory_GetCurrent() -> cstr {
    static mut buffer: [libc::c_char; 1024] = [0; 1024];
    if !(libc::getcwd(
        buffer.as_mut_ptr(),
        ::core::mem::size_of::<[libc::c_char; 1024]>() as usize,
    ))
    .is_null()
    {
        return 0 as cstr;
    }
    buffer[(::core::mem::size_of::<[libc::c_char; 1024]>()).wrapping_sub(1 as usize)] =
        0 as i32 as libc::c_char;
    return buffer.as_mut_ptr() as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Remove(mut path: cstr) -> bool {
    return libc::rmdir(path) == 0 as i32;
}
