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
        flags: Uint32,
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
    fn SDL_SetWindowFullscreen(window: *mut SDL_Window, flags: Uint32) -> libc::c_int;
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
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Window {
    pub handle: *mut SDL_Window,
    pub context: SDL_GLContext,
    pub mode: WindowMode,
}
pub type WindowMode = uint32;
pub type SDL_GLContext = *mut libc::c_void;

pub type WindowPos = libc::c_int;
pub type Uint32 = uint32_t;
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
    let mut self_0: *mut Window = MemAlloc(
        ::core::mem::size_of::<Window>() as usize,
    ) as *mut Window;
    mode |= SDL_WINDOW_OPENGL as libc::c_int as libc::c_uint;
    (*self_0).handle = SDL_CreateWindow(title, x, y, sx, sy, mode);
    (*self_0).context = SDL_GL_CreateContext((*self_0).handle);
    (*self_0).mode = mode;
    if ((*self_0).context).is_null() {
        Fatal(
            b"Failed to create OpenGL context for window\0" as *const u8
                as *const libc::c_char,
        );
    }
    OpenGL_Init();
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Window_Free(mut self_0: *mut Window) {
    SDL_GL_DeleteContext((*self_0).context);
    SDL_DestroyWindow((*self_0).handle);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Window_BeginDraw(mut self_0: *mut Window) {
    let mut size: IVec2 = IVec2::new(0, 0);
    SDL_GL_MakeCurrent((*self_0).handle, (*self_0).context);
    Window_GetSize(self_0, &mut size);
    Viewport_Push(
        0 as libc::c_int,
        0 as libc::c_int,
        size.x,
        size.y,
        1 as libc::c_int != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Window_EndDraw(mut self_0: *mut Window) {
    Viewport_Pop();
    SDL_GL_SwapWindow((*self_0).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetSize(mut self_0: *mut Window, mut out: *mut IVec2) {
    SDL_GetWindowSize((*self_0).handle, &mut (*out).x, &mut (*out).y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetPosition(
    mut self_0: *mut Window,
    mut out: *mut IVec2,
) {
    SDL_GetWindowPosition((*self_0).handle, &mut (*out).x, &mut (*out).y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_GetTitle(mut self_0: *mut Window) -> cstr {
    return SDL_GetWindowTitle((*self_0).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetFullscreen(mut self_0: *mut Window, mut fs: bool) {
    SDL_SetWindowFullscreen(
        (*self_0).handle,
        if fs as libc::c_int != 0 {
            WindowMode_Fullscreen
        } else {
            0 as libc::c_int as libc::c_uint
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetPosition(
    mut self_0: *mut Window,
    mut x: WindowPos,
    mut y: WindowPos,
) {
    SDL_SetWindowPosition((*self_0).handle, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetSize(
    mut self_0: *mut Window,
    mut sx: libc::c_int,
    mut sy: libc::c_int,
) {
    SDL_SetWindowSize((*self_0).handle, sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetTitle(mut self_0: *mut Window, mut title: cstr) {
    SDL_SetWindowTitle((*self_0).handle, title);
}
#[no_mangle]
pub unsafe extern "C" fn Window_SetVsync(mut self_0: *mut Window, mut vsync: bool) {
    SDL_GL_SetSwapInterval(
        if vsync as libc::c_int != 0 { 1 as libc::c_int } else { 0 as libc::c_int },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Window_ToggleFullscreen(mut self_0: *mut Window) {
    if (*self_0).mode & WindowMode_Fullscreen != 0 {
        SDL_SetWindowFullscreen((*self_0).handle, 0 as libc::c_int as Uint32);
    } else {
        SDL_SetWindowFullscreen((*self_0).handle, WindowMode_Fullscreen);
    }
    (*self_0).mode ^= WindowMode_Fullscreen;
}
#[no_mangle]
pub unsafe extern "C" fn Window_Hide(mut self_0: *mut Window) {
    SDL_HideWindow((*self_0).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Window_Show(mut self_0: *mut Window) {
    SDL_ShowWindow((*self_0).handle);
}
