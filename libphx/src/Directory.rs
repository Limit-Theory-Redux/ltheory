use crate::internal::Memory::*;
use crate::File::*;
use glam::Vec3;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Directory {
    pub handle: *mut libc::DIR,
}

#[no_mangle]
pub unsafe extern "C" fn Directory_Open(mut path: *const libc::c_char) -> *mut Directory {
    let mut dir: *mut libc::DIR = libc::opendir(path);
    if dir.is_null() {
        return std::ptr::null_mut();
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
pub unsafe extern "C" fn Directory_GetNext(mut this: *mut Directory) -> *const libc::c_char {
    loop {
        let mut ent: *mut libc::dirent = libc::readdir((*this).handle);
        if ent.is_null() {
            return std::ptr::null();
        }
        if StrEqual(
            ((*ent).d_name).as_mut_ptr() as *const libc::c_char,
            b".\0" as *const u8 as *const libc::c_char,
        ) as i32
            != 0
            || StrEqual(
                ((*ent).d_name).as_mut_ptr() as *const libc::c_char,
                b"..\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
        {
            continue;
        }
        return ((*ent).d_name).as_mut_ptr() as *const libc::c_char;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Change(mut cwd: *const libc::c_char) -> bool {
    return libc::chdir(cwd) == 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Create(mut path: *const libc::c_char) -> bool {
    libc::mkdir(path, 0o775 as libc::mode_t);
    return File_IsDir(path);
}
#[no_mangle]
pub unsafe extern "C" fn Directory_GetCurrent() -> *const libc::c_char {
    static mut buffer: [libc::c_char; 1024] = [0; 1024];
    if !(libc::getcwd(
        buffer.as_mut_ptr(),
        ::core::mem::size_of::<[libc::c_char; 1024]>() as usize,
    ))
    .is_null()
    {
        return std::ptr::null();
    }
    buffer[(::core::mem::size_of::<[libc::c_char; 1024]>()).wrapping_sub(1 as usize)] =
        0 as i32 as libc::c_char;
    return buffer.as_mut_ptr() as *const libc::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn Directory_Remove(mut path: *const libc::c_char) -> bool {
    return libc::rmdir(path) == 0 as i32;
}
