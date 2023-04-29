use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Thread {
    pub handle: *mut SDL_Thread,
}

pub type ThreadFn = Option<unsafe extern "C" fn(*mut libc::c_void) -> i32>;

#[no_mangle]
pub unsafe extern "C" fn Thread_Create(
    name: *const libc::c_char,
    fn_0: ThreadFn,
    data: *mut libc::c_void,
) -> *mut Thread {
    let this = MemNew!(Thread);
    (*this).handle = SDL_CreateThread(fn_0, name, data);
    if ((*this).handle).is_null() {
        CFatal!("Thread_Create: Failed to start new thread");
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Thread_Detach(this: *mut Thread) {
    SDL_DetachThread((*this).handle);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Thread_Sleep(ms: u32) {
    SDL_Delay(ms);
}

#[no_mangle]
pub unsafe extern "C" fn Thread_Wait(this: *mut Thread) -> i32 {
    let mut ret: i32 = 0;
    SDL_WaitThread((*this).handle, &mut ret);
    MemFree(this as *const _);
    ret
}
