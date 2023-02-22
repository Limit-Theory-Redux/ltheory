use crate::internal::Memory::*;
use crate::WindowMode::*;
use crate::WindowPos::WindowPos;
use glam::IVec2;
use glam::Vec3;
use libc;
use sdl2_sys::*;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn OpenGL_Init();
    fn Viewport_Pop();
    fn Viewport_Push(x: i32, y: i32, sx: i32, sy: i32, isWindow: bool);
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Window {
    pub handle: *mut SDL_Window,
    pub context: SDL_GLContext,
    pub mode: WindowMode,
}

#[no_mangle]
pub unsafe extern "C" fn Window_Create(
    mut title: *const libc::c_char,
    mut x: i32,
    mut y: i32,
    mut sx: i32,
    mut sy: i32,
    mut mode: WindowMode,
) -> *mut Window {
    let mut this: *mut Window = MemAlloc(::core::mem::size_of::<Window>() as usize) as *mut Window;
    mode |= SDL_WINDOW_OPENGL as i32 as u32;
    (*this).handle = SDL_CreateWindow(title, x, y, sx, sy, mode);
    (*this).context = SDL_GL_CreateContext((*this).handle);
    (*this).mode = mode;
    if ((*this).context).is_null() {
        Fatal(b"Failed to create OpenGL context for window\0" as *const u8 as *const libc::c_char);
    }
    OpenGL_Init();
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Window_Free(mut this: *mut Window) {
    SDL_GL_DeleteContext((*this).context);
    SDL_DestroyWindow((*this).handle);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Window_BeginDraw(mut this: *mut Window) {
    let mut size: IVec2 = IVec2::new(0, 0);
    SDL_GL_MakeCurrent((*this).handle, (*this).context);
    Window_GetSize(this, &mut size);
    Viewport_Push(0 as i32, 0 as i32, size.x, size.y, 1 as i32 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn Window_EndDraw(mut this: *mut Window) {
    Viewport_Pop();
    SDL_GL_SwapWindow((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetSize(mut this: *mut Window, mut out: *mut IVec2) {
    SDL_GetWindowSize((*this).handle, &mut (*out).x, &mut (*out).y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetPosition(mut this: *mut Window, mut out: *mut IVec2) {
    SDL_GetWindowPosition((*this).handle, &mut (*out).x, &mut (*out).y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetTitle(mut this: *mut Window) -> *const libc::c_char {
    return SDL_GetWindowTitle((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetFullscreen(mut this: *mut Window, mut fs: bool) {
    SDL_SetWindowFullscreen(
        (*this).handle,
        if fs as i32 != 0 {
            WindowMode_Fullscreen
        } else {
            0 as i32 as u32
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetPosition(
    mut this: *mut Window,
    mut x: WindowPos,
    mut y: WindowPos,
) {
    SDL_SetWindowPosition((*this).handle, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetSize(mut this: *mut Window, mut sx: i32, mut sy: i32) {
    SDL_SetWindowSize((*this).handle, sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetTitle(mut this: *mut Window, mut title: *const libc::c_char) {
    SDL_SetWindowTitle((*this).handle, title);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetVsync(mut this: *mut Window, mut vsync: bool) {
    SDL_GL_SetSwapInterval(if vsync {
        1
    } else {
        0
    });
}
#[no_mangle]
pub unsafe extern "C" fn Window_ToggleFullscreen(mut this: *mut Window) {
    if (*this).mode & WindowMode_Fullscreen != 0 {
        SDL_SetWindowFullscreen((*this).handle, 0);
    } else {
        SDL_SetWindowFullscreen((*this).handle, WindowMode_Fullscreen);
    }
    (*this).mode ^= WindowMode_Fullscreen;
}
#[no_mangle]
pub unsafe extern "C" fn Window_Hide(mut this: *mut Window) {
    SDL_HideWindow((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_Show(mut this: *mut Window) {
    SDL_ShowWindow((*this).handle);
}
