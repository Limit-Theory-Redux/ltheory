use ::libc;
use glam::Vec3;
use glam::IVec2;
use crate::internal::Memory::*;
use crate::WindowMode::*;
use sdl2_sys::*;

extern "C" {
    pub type SDL_Window;
    fn Fatal(_: cstr, _: ...);
    fn OpenGL_Init();
    fn SDL_CreateWindow(
        title: *const libc::c_char,
        x: libc::c_int,
        y: libc::c_int,
        w: libc::c_int,
        h: libc::c_int,
        flags: u32,
    ) -> *mut SDL_Window;
    fn SDL_SetWindowTitle(window: *mut SDL_Window, title: *const libc::c_char);
    fn SDL_GetWindowTitle(window: *mut SDL_Window) -> *const libc::c_char;
    fn SDL_SetWindowPosition(window: *mut SDL_Window, x: libc::c_int, y: libc::c_int);
    fn SDL_GetWindowPosition(
        window: *mut SDL_Window,
        x: *mut libc::c_int,
        y: *mut libc::c_int,
    );
    fn SDL_SetWindowSize(window: *mut SDL_Window, w: libc::c_int, h: libc::c_int);
    fn SDL_GetWindowSize(
        window: *mut SDL_Window,
        w: *mut libc::c_int,
        h: *mut libc::c_int,
    );
    fn SDL_ShowWindow(window: *mut SDL_Window);
    fn SDL_HideWindow(window: *mut SDL_Window);
    fn SDL_SetWindowFullscreen(window: *mut SDL_Window, flags: u32) -> libc::c_int;
    fn SDL_DestroyWindow(window: *mut SDL_Window);
    fn SDL_GL_CreateContext(window: *mut SDL_Window) -> SDL_GLContext;
    fn SDL_GL_MakeCurrent(
        window: *mut SDL_Window,
        context: SDL_GLContext,
    ) -> libc::c_int;
    fn SDL_GL_SetSwapInterval(interval: libc::c_int) -> libc::c_int;
    fn SDL_GL_SwapWindow(window: *mut SDL_Window);
    fn SDL_GL_DeleteContext(context: SDL_GLContext);
    fn Viewport_Pop();
    fn Viewport_Push(
        x: libc::c_int,
        y: libc::c_int,
        sx: libc::c_int,
        sy: libc::c_int,
        isWindow: bool,
    );
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Window {
    pub handle: *mut SDL_Window,
    pub context: SDL_GLContext,
    pub mode: WindowMode,
}
pub type WindowMode = u32;
pub type SDL_GLContext = *mut libc::c_void;

pub type WindowPos = libc::c_int;
pub type C2RustUnnamed = libc::c_uint;
pub const SDL_WINDOW_INPUT_GRABBED: C2RustUnnamed = 256;
pub const SDL_WINDOW_METAL: C2RustUnnamed = 536870912;
pub const SDL_WINDOW_VULKAN: C2RustUnnamed = 268435456;
pub const SDL_WINDOW_KEYBOARD_GRABBED: C2RustUnnamed = 1048576;
pub const SDL_WINDOW_POPUP_MENU: C2RustUnnamed = 524288;
pub const SDL_WINDOW_TOOLTIP: C2RustUnnamed = 262144;
pub const SDL_WINDOW_UTILITY: C2RustUnnamed = 131072;
pub const SDL_WINDOW_SKIP_TASKBAR: C2RustUnnamed = 65536;
pub const SDL_WINDOW_ALWAYS_ON_TOP: C2RustUnnamed = 32768;
pub const SDL_WINDOW_MOUSE_CAPTURE: C2RustUnnamed = 16384;
pub const SDL_WINDOW_ALLOW_HIGHDPI: C2RustUnnamed = 8192;
pub const SDL_WINDOW_FOREIGN: C2RustUnnamed = 2048;
pub const SDL_WINDOW_FULLSCREEN_DESKTOP: C2RustUnnamed = 4097;
pub const SDL_WINDOW_MOUSE_FOCUS: C2RustUnnamed = 1024;
pub const SDL_WINDOW_INPUT_FOCUS: C2RustUnnamed = 512;
pub const SDL_WINDOW_MOUSE_GRABBED: C2RustUnnamed = 256;
pub const SDL_WINDOW_MAXIMIZED: C2RustUnnamed = 128;
pub const SDL_WINDOW_MINIMIZED: C2RustUnnamed = 64;
pub const SDL_WINDOW_RESIZABLE: C2RustUnnamed = 32;
pub const SDL_WINDOW_BORDERLESS: C2RustUnnamed = 16;
pub const SDL_WINDOW_HIDDEN: C2RustUnnamed = 8;
pub const SDL_WINDOW_SHOWN: C2RustUnnamed = 4;
pub const SDL_WINDOW_OPENGL: C2RustUnnamed = 2;
pub const SDL_WINDOW_FULLSCREEN: C2RustUnnamed = 1;

#[no_mangle]
pub unsafe extern "C" fn Window_Create(
    mut title: cstr,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut mode: WindowMode,
) -> *mut Window {
    let mut this: *mut Window = MemAlloc(
        ::core::mem::size_of::<Window>() as usize,
    ) as *mut Window;
    mode |= SDL_WINDOW_OPENGL as libc::c_int as libc::c_uint;
    (*this).handle = SDL_CreateWindow(title, x, y, sx, sy, mode);
    (*this).context = SDL_GL_CreateContext((*this).handle);
    (*this).mode = mode;
    if ((*this).context).is_null() {
        Fatal(
            b"Failed to create OpenGL context for window\0" as *const u8
                as *const libc::c_char,
        );
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
    Viewport_Push(
        0 as libc::c_int,
        0 as libc::c_int,
        size.x,
        size.y,
        1 as libc::c_int != 0,
    );
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
pub unsafe extern "C" fn Window_GetPosition(
    mut this: *mut Window,
    mut out: *mut IVec2,
) {
    SDL_GetWindowPosition((*this).handle, &mut (*out).x, &mut (*out).y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetTitle(mut this: *mut Window) -> cstr {
    return SDL_GetWindowTitle((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetFullscreen(mut this: *mut Window, mut fs: bool) {
    SDL_SetWindowFullscreen(
        (*this).handle,
        if fs as libc::c_int != 0 {
            WindowMode_Fullscreen
        } else {
            0 as libc::c_int as libc::c_uint
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
pub unsafe extern "C" fn Window_SetSize(
    mut this: *mut Window,
    mut sx: libc::c_int,
    mut sy: libc::c_int,
) {
    SDL_SetWindowSize((*this).handle, sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetTitle(mut this: *mut Window, mut title: cstr) {
    SDL_SetWindowTitle((*this).handle, title);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetVsync(mut this: *mut Window, mut vsync: bool) {
    SDL_GL_SetSwapInterval(
        if vsync as libc::c_int != 0 { 1 as libc::c_int } else { 0 as libc::c_int },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Window_ToggleFullscreen(mut this: *mut Window) {
    if (*this).mode & WindowMode_Fullscreen != 0 {
        SDL_SetWindowFullscreen((*this).handle, 0 as libc::c_int as u32);
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
