use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type SDL_Thread;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn Fatal(_: cstr, _: ...);
    fn SDL_Delay(ms: Uint32);
    fn SDL_CreateThread(
        fn_0: SDL_ThreadFunction,
        name: *const libc::c_char,
        data: *mut libc::c_void,
    ) -> *mut SDL_Thread;
    fn SDL_WaitThread(thread: *mut SDL_Thread, status: *mut libc::c_int);
    fn SDL_DetachThread(thread: *mut SDL_Thread);
}
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type uint = libc::c_uint;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Thread {
    pub handle: *mut SDL_Thread,
}
pub type Uint32 = uint32_t;
pub type SDL_ThreadFunction = Option::<
    unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int,
>;
pub type ThreadFn = Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>;

#[no_mangle]
pub unsafe extern "C" fn Thread_Create(
    mut name: cstr,
    mut fn_0: ThreadFn,
    mut data: *mut libc::c_void,
) -> *mut Thread {
    let mut self_0: *mut Thread = MemAlloc(
        ::core::mem::size_of::<Thread>() as usize,
    ) as *mut Thread;
    (*self_0).handle = SDL_CreateThread(fn_0, name, data);
    if ((*self_0).handle).is_null() {
        Fatal(
            b"Thread_Create: Failed to start new thread\0" as *const u8
                as *const libc::c_char,
        );
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Thread_Detach(mut self_0: *mut Thread) {
    SDL_DetachThread((*self_0).handle);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Thread_Sleep(mut ms: uint) {
    SDL_Delay(ms);
}
#[no_mangle]
pub unsafe extern "C" fn Thread_Wait(mut self_0: *mut Thread) -> libc::c_int {
    let mut ret: libc::c_int = 0;
    SDL_WaitThread((*self_0).handle, &mut ret);
    MemFree(self_0 as *const libc::c_void);
    return ret;
}
