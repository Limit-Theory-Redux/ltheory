use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::IVec2;
use crate::Math::Vec3;
use crate::OpenGL::*;
use crate::Viewport::*;
use crate::WindowMode::*;
use crate::WindowPos::WindowPos;
use libc;
use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Window {
    pub handle: *mut SDL_Window,
    pub context: SDL_GLContext,
    pub mode: WindowMode,
}

#[no_mangle]
pub unsafe extern "C" fn Window_Create(
    title: *const libc::c_char,
    x: i32,
    y: i32,
    sx: i32,
    sy: i32,
    mode: WindowMode,
) -> Box<Window> {
    let modeWithGL = mode | SDL_WindowFlags::SDL_WINDOW_OPENGL as WindowMode;
    let handle = SDL_CreateWindow(title, x, y, sx, sy, modeWithGL);
    let context = SDL_GL_CreateContext(handle);
    if context.is_null() {
        Fatal(b"Failed to create OpenGL context for window\0" as *const u8 as *const libc::c_char);
    }
    OpenGL_Init();

    Box::new(Window {
        handle: handle,
        context: context,
        mode: modeWithGL,
    })
}

#[no_mangle]
pub unsafe extern "C" fn Window_Free(w: Option<Box<Window>>) {
    let w = unwrap_or_return!(w);
    SDL_GL_DeleteContext(w.context);
    SDL_DestroyWindow(w.handle);
}

#[no_mangle]
pub unsafe extern "C" fn Window_BeginDraw(w: Option<&Window>) {
    let w = unwrap_or_return!(w);
    let mut size: IVec2 = IVec2::new(0, 0);
    SDL_GL_MakeCurrent(w.handle, w.context);
    Window_GetSize(Some(w), &mut size);
    Viewport_Push(0_i32, 0_i32, size.x, size.y, true);
}

#[no_mangle]
pub unsafe extern "C" fn Window_EndDraw(w: Option<&Window>) {
    let w = unwrap_or_return!(w);
    Viewport_Pop();
    SDL_GL_SwapWindow(w.handle);
}

#[no_mangle]
pub unsafe extern "C" fn Window_GetSize(w: Option<&Window>, mut out: *mut IVec2) {
    let w = unwrap_or_return!(w);
    SDL_GetWindowSize(w.handle, &mut (*out).x, &mut (*out).y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_GetPosition(w: Option<&Window>, mut out: *mut IVec2) {
    let w = unwrap_or_return!(w);
    SDL_GetWindowPosition(w.handle, &mut (*out).x, &mut (*out).y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_GetTitle(w: Option<&Window>) -> *const libc::c_char {
    let w = unwrap_or_return!(w, std::ptr::null());
    SDL_GetWindowTitle(w.handle)
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetFullscreen(w: Option<&Window>, mut fs: bool) {
    let w = unwrap_or_return!(w);
    SDL_SetWindowFullscreen(w.handle, if fs { WindowMode_Fullscreen } else { 0_u32 });
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetPosition(
    w: Option<&Window>,
    mut x: WindowPos,
    mut y: WindowPos,
) {
    let w = unwrap_or_return!(w);
    SDL_SetWindowPosition(w.handle, x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetSize(w: Option<&Window>, mut sx: i32, mut sy: i32) {
    let w = unwrap_or_return!(w);
    SDL_SetWindowSize(w.handle, sx, sy);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetTitle(w: Option<&Window>, mut title: *const libc::c_char) {
    let w = unwrap_or_return!(w);
    SDL_SetWindowTitle(w.handle, title);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetVsync(_: Option<&Window>, mut vsync: bool) {
    SDL_GL_SetSwapInterval(if vsync { 1 } else { 0 });
}

#[no_mangle]
pub unsafe extern "C" fn Window_ToggleFullscreen(w: Option<&mut Window>) {
    let w = unwrap_or_return!(w);
    if w.mode & WindowMode_Fullscreen != 0 {
        SDL_SetWindowFullscreen(w.handle, 0);
    } else {
        SDL_SetWindowFullscreen(w.handle, WindowMode_Fullscreen);
    }
    w.mode ^= WindowMode_Fullscreen;
}

#[no_mangle]
pub unsafe extern "C" fn Window_Hide(w: Option<&Window>) {
    let w = unwrap_or_return!(w);
    SDL_HideWindow(w.handle);
}

#[no_mangle]
pub unsafe extern "C" fn Window_Show(w: Option<&Window>) {
    let w = unwrap_or_return!(w);
    SDL_ShowWindow(w.handle);
}
