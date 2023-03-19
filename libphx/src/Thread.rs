use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Thread {
    pub handle: *mut SDL_Thread,
}

pub type ThreadFn = Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>;

#[no_mangle]
pub unsafe extern "C" fn Thread_Create(
    mut name: *const libc::c_char,
    mut fn_0: ThreadFn,
    mut data: *mut libc::c_void,
) -> *mut Thread {
    let mut this: *mut Thread = MemAlloc(std::mem::size_of::<Thread>()) as *mut Thread;
    (*this).handle = SDL_CreateThread(fn_0, name, data);
    if ((*this).handle).is_null() {
        Fatal(b"Thread_Create: Failed to start new thread\0" as *const u8 as *const libc::c_char);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Thread_Detach(mut this: *mut Thread) {
    SDL_DetachThread((*this).handle);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn Thread_Sleep(mut ms: u32) {
    SDL_Delay(ms);
}

#[no_mangle]
pub unsafe extern "C" fn Thread_Wait(mut this: *mut Thread) -> i32 {
    let mut ret: i32 = 0;
    SDL_WaitThread((*this).handle, &mut ret);
    MemFree(this as *const libc::c_void);
    ret
}
