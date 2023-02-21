use crate::internal::Memory::*;
use glam::Vec3;
use libc;
extern "C" {
    pub type SDL_Thread;
    fn Fatal(_: cstr, _: ...);
    fn SDL_Delay(ms: u32);
    fn SDL_CreateThread(
        fn_0: SDL_ThreadFunction,
        name: *const libc::c_char,
        data: *mut libc::c_void,
    ) -> *mut SDL_Thread;
    fn SDL_WaitThread(thread: *mut SDL_Thread, status: *mut i32);
    fn SDL_DetachThread(thread: *mut SDL_Thread);
}
pub type uint = u32;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Thread {
    pub handle: *mut SDL_Thread,
}
pub type SDL_ThreadFunction = Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>;
pub type ThreadFn = Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>;

#[no_mangle]
pub unsafe extern "C" fn Thread_Create(
    mut name: cstr,
    mut fn_0: ThreadFn,
    mut data: *mut libc::c_void,
) -> *mut Thread {
    let mut this: *mut Thread = MemAlloc(::core::mem::size_of::<Thread>() as usize) as *mut Thread;
    (*this).handle = SDL_CreateThread(fn_0, name, data);
    if ((*this).handle).is_null() {
        Fatal(b"Thread_Create: Failed to start new thread\0" as *const u8 as *const libc::c_char);
    }
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Thread_Detach(mut this: *mut Thread) {
    SDL_DetachThread((*this).handle);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Thread_Sleep(mut ms: uint) {
    SDL_Delay(ms);
}
#[no_mangle]
pub unsafe extern "C" fn Thread_Wait(mut this: *mut Thread) -> i32 {
    let mut ret: i32 = 0;
    SDL_WaitThread((*this).handle, &mut ret);
    MemFree(this as *const libc::c_void);
    return ret;
}
