use crate::internal::Memory::*;
use crate::Common::*;
use crate::File::*;
use crate::Math::Vec3;
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
    let mut this = MemNew!(Directory);
    (*this).handle = dir;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Directory_Close(mut this: *mut Directory) {
    libc::closedir((*this).handle);
    MemFree(this as *const _);
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
    libc::chdir(cwd) == 0
}

#[no_mangle]
pub unsafe extern "C" fn Directory_Create(mut path: *const libc::c_char) -> bool {
    libc::mkdir(path, 0o775 as libc::mode_t);
    File_IsDir(path)
}

#[no_mangle]
pub unsafe extern "C" fn Directory_GetCurrent() -> *const libc::c_char {
    static mut buffer: [libc::c_char; 1024] = [0; 1024];
    if !(libc::getcwd(
        buffer.as_mut_ptr(),
        std::mem::size_of::<[libc::c_char; 1024]>(),
    ))
    .is_null()
    {
        return std::ptr::null();
    }
    buffer[(std::mem::size_of::<[libc::c_char; 1024]>()).wrapping_sub(1)] =
        0 as libc::c_char;
    buffer.as_mut_ptr() as *const libc::c_char
}

#[no_mangle]
pub unsafe extern "C" fn Directory_Remove(mut path: *const libc::c_char) -> bool {
    libc::rmdir(path) == 0
}
