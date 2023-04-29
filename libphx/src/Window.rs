use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::IVec2;
use crate::Math::Vec3;
use crate::OpenGL::*;
use crate::Viewport::*;
use crate::WindowMode::*;
use crate::WindowPos::WindowPos;
use sdl2_sys::*;

#[repr(C)]
pub struct Window {
    pub handle: *mut SDL_Window,
    pub context: SDL_GLContext,
    pub mode: WindowMode,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            SDL_GL_DeleteContext(self.context);
            SDL_DestroyWindow(self.handle);
        }
    }
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
        CFatal!("Failed to create OpenGL context for window");
    }
    OpenGL_Init();

    Box::new(Window {
        handle: handle,
        context: context,
        mode: modeWithGL,
    })
}

#[no_mangle]
pub extern "C" fn Window_Free(_: Option<Box<Window>>) {}

#[no_mangle]
pub unsafe extern "C" fn Window_BeginDraw(w: &Window) {
    let mut size: IVec2 = IVec2::ZERO;
    SDL_GL_MakeCurrent(w.handle, w.context);
    Window_GetSize(w, &mut size);
    Viewport_Push(0, 0, size.x, size.y, true);
}

#[no_mangle]
pub unsafe extern "C" fn Window_EndDraw(w: &Window) {
    Viewport_Pop();
    SDL_GL_SwapWindow(w.handle);
}

#[no_mangle]
pub unsafe extern "C" fn Window_GetSize(w: &Window, out: &mut IVec2) {
    SDL_GetWindowSize(w.handle, &mut out.x, &mut out.y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_GetPosition(w: &Window, out: &mut IVec2) {
    SDL_GetWindowPosition(w.handle, &mut out.x, &mut out.y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_GetTitle(w: &Window) -> *const libc::c_char {
    SDL_GetWindowTitle(w.handle)
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetFullscreen(w: &Window, fs: bool) {
    SDL_SetWindowFullscreen(w.handle, if fs { WindowMode_Fullscreen } else { 0 });
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetPosition(w: &Window, x: WindowPos, y: WindowPos) {
    SDL_SetWindowPosition(w.handle, x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetSize(w: &Window, sx: i32, sy: i32) {
    SDL_SetWindowSize(w.handle, sx, sy);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetTitle(w: &Window, title: *const libc::c_char) {
    SDL_SetWindowTitle(w.handle, title);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetVsync(_: Option<&Window>, vsync: bool) {
    SDL_GL_SetSwapInterval(if vsync { 1 } else { 0 });
}

#[no_mangle]
pub unsafe extern "C" fn Window_ToggleFullscreen(w: &mut Window) {
    if w.mode & WindowMode_Fullscreen != 0 {
        SDL_SetWindowFullscreen(w.handle, 0);
    } else {
        SDL_SetWindowFullscreen(w.handle, WindowMode_Fullscreen);
    }
    w.mode ^= WindowMode_Fullscreen;
}

#[no_mangle]
pub unsafe extern "C" fn Window_Hide(w: &Window) {
    SDL_HideWindow(w.handle);
}

#[no_mangle]
pub unsafe extern "C" fn Window_Show(w: &Window) {
    SDL_ShowWindow(w.handle);
}
