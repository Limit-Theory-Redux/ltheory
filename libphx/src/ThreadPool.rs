use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type SDL_Thread;
    fn Fatal(_: cstr, _: ...);
    fn SDL_CreateThread(
        fn_0: SDL_ThreadFunction,
        name: *const libc::c_char,
        data: *mut libc::c_void,
    ) -> *mut SDL_Thread;
    fn SDL_WaitThread(thread: *mut SDL_Thread, status: *mut i32);
}
pub type cstr = *const libc::c_char;
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
pub type ThreadPoolFn = Option::<
    unsafe extern "C" fn(i32, i32, *mut libc::c_void) -> i32,
>;
pub type SDL_ThreadFunction = Option::<
    unsafe extern "C" fn(*mut libc::c_void) -> i32,
>;

unsafe extern "C" fn ThreadPool_Dispatch(mut data: *mut libc::c_void) -> i32 {
    let mut td: *mut ThreadData = data as *mut ThreadData;
    return ((*td).fn_0)
        .expect("non-null function pointer")((*td).index, (*td).threads, (*td).data);
}
#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Create(mut threads: i32) -> *mut ThreadPool {
    let mut this: *mut ThreadPool = MemAlloc(
        ::core::mem::size_of::<ThreadPool>() as usize,
    ) as *mut ThreadPool;
    (*this).threads = threads;
    (*this)
        .thread = MemAlloc(
        ::core::mem::size_of::<ThreadData>().wrapping_mul(threads as usize),
    ) as *mut ThreadData;
    let mut i: i32 = 0 as i32;
    while i < threads {
        let mut td: *mut ThreadData = ((*this).thread).offset(i as isize);
        (*td).handle = 0 as *mut SDL_Thread;
        (*td).fn_0 = None;
        (*td).index = i;
        (*td).threads = threads;
        (*td).data = 0 as *mut libc::c_void;
        i += 1;
    }
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Free(mut this: *mut ThreadPool) {
    let mut i: i32 = 0 as i32;
    while i < (*this).threads {
        if !((*((*this).thread).offset(i as isize)).handle).is_null() {
            Fatal(
                b"ThreadPool_Free: Attempting to free pool with active threads\0"
                    as *const u8 as *const libc::c_char,
            );
        }
        i += 1;
    }
    MemFree((*this).thread as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Launch(
    mut this: *mut ThreadPool,
    mut fn_0: ThreadPoolFn,
    mut data: *mut libc::c_void,
) {
    let mut i: i32 = 0 as i32;
    while i < (*this).threads {
        let mut td: *mut ThreadData = ((*this).thread).offset(i as isize);
        (*td).fn_0 = fn_0;
        (*td).data = data;
        (*td)
            .handle = SDL_CreateThread(
            Some(
                ThreadPool_Dispatch
                    as unsafe extern "C" fn(*mut libc::c_void) -> i32,
            ),
            b"PHX_ThreadPool\0" as *const u8 as *const libc::c_char,
            td as *mut libc::c_void,
        );
        if ((*td).handle).is_null() {
            Fatal(
                b"ThreadPool_Launch: Failed to start new thread\0" as *const u8
                    as *const libc::c_char,
            );
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ThreadPool_Wait(mut this: *mut ThreadPool) {
    let mut i: i32 = 0 as i32;
    while i < (*this).threads {
        let mut td: *mut ThreadData = ((*this).thread).offset(i as isize);
        if !((*td).handle).is_null() {
            let mut ret: i32 = 0;
            SDL_WaitThread((*td).handle, &mut ret);
            (*td).handle = 0 as *mut SDL_Thread;
        }
        i += 1;
    }
}
