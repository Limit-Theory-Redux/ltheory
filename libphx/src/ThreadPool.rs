use crate::internal::Memory::*;
use crate::Common::*;

use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ThreadPool {
    pub threads: i32,
    pub thread: *mut ThreadData,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ThreadData {
    pub handle: *mut SDL_Thread,
    pub fn_0: ThreadPoolFn,
    pub index: i32,
    pub threads: i32,
    pub data: *mut libc::c_void,
}
pub type ThreadPoolFn = Option<extern "C" fn(i32, i32, *mut libc::c_void) -> i32>;

unsafe extern "C" fn ThreadPool_Dispatch(data: *mut libc::c_void) -> i32 {
    let td: *mut ThreadData = data as *mut ThreadData;
    ((*td).fn_0).expect("non-null function pointer")((*td).index, (*td).threads, (*td).data)
}

#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Create(threads: i32) -> *mut ThreadPool {
    let this = MemNew!(ThreadPool);
    (*this).threads = threads;
    (*this).thread = MemNewArray!(ThreadData, threads);
    let mut i: i32 = 0;
    while i < threads {
        let td: *mut ThreadData = ((*this).thread).offset(i as isize);
        (*td).handle = std::ptr::null_mut();
        (*td).fn_0 = std::option::Option::None;
        (*td).index = i;
        (*td).threads = threads;
        (*td).data = std::ptr::null_mut();
        i += 1;
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Free(this: *mut ThreadPool) {
    let mut i: i32 = 0;
    while i < (*this).threads {
        if !((*((*this).thread).offset(i as isize)).handle).is_null() {
            CFatal!("ThreadPool_Free: Attempting to free pool with active threads");
        }
        i += 1;
    }
    MemFree((*this).thread as *const _);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Launch(
    this: &mut ThreadPool,
    fn_0: ThreadPoolFn,
    data: *mut libc::c_void,
) {
    let mut i: i32 = 0;
    while i < this.threads {
        let td: *mut ThreadData = (this.thread).offset(i as isize);
        (*td).fn_0 = fn_0;
        (*td).data = data;
        (*td).handle = SDL_CreateThread(
            Some(ThreadPool_Dispatch as unsafe extern "C" fn(*mut libc::c_void) -> i32),
            c_str!("PHX_ThreadPool"),
            td as *mut _,
        );
        if ((*td).handle).is_null() {
            CFatal!("ThreadPool_Launch: Failed to start new thread");
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Wait(this: &mut ThreadPool) {
    let mut i: i32 = 0;
    while i < this.threads {
        let td: *mut ThreadData = (this.thread).offset(i as isize);
        if !((*td).handle).is_null() {
            let mut ret: i32 = 0;
            SDL_WaitThread((*td).handle, &mut ret);
            (*td).handle = std::ptr::null_mut();
        }
        i += 1;
    }
}
