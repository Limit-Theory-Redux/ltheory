use crate::common::*;
use crate::math::IVec2;
use crate::open_gl::*;
use crate::resource::*;
use crate::resource_type::*;
use crate::tex2d_load::*;
use crate::viewport::*;
use crate::window_mode::*;
use crate::window_pos::WindowPos;
use sdl2_sys::*;

#[repr(C)]
pub struct Window {
    pub handle: *mut SDL_Window,
    pub context: SDL_GLContext,
    pub mode: WindowMode,
    pub cursor: *mut SDL_Cursor,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            SDL_FreeCursor(self.cursor); // Can take null.
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
    let modeComplete = mode | SDL_WindowFlags::SDL_WINDOW_OPENGL as WindowMode;
    // | SDL_WindowFlags::SDL_WINDOW_ALLOW_HIGHDPI as WindowMode;
    let handle = SDL_CreateWindow(title, x, y, sx, sy, modeComplete);
    let context = SDL_GL_CreateContext(handle);
    if context.is_null() {
        CFatal!("Failed to create OpenGL context for window");
    }
    OpenGL_Init();

    Box::new(Window {
        handle,
        context,
        mode: modeComplete,
        cursor: std::ptr::null_mut(),
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
pub unsafe extern "C" fn Window_SetWindowGrab(w: &Window, grabbed: bool) {
    SDL_SetWindowGrab(
        w.handle,
        if grabbed {
            SDL_bool::SDL_TRUE
        } else {
            SDL_bool::SDL_FALSE
        },
    );
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetMousePosition(w: &Window, position: &IVec2) {
    SDL_WarpMouseInWindow(w.handle, position.x, position.y);
}

#[no_mangle]
pub unsafe extern "C" fn Window_SetCursor(
    w: &mut Window,
    name: *const libc::c_char,
    hotx: i32,
    hoty: i32,
) {
    SDL_FreeCursor(w.cursor); // Can take null.

    let path = Resource_GetPath(ResourceType_Tex2D, name);

    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut components: i32 = 0;
    let data = Tex2D_LoadRaw(path, &mut width, &mut height, &mut components);

    let pixelFormat = if components == 3 {
        SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB24
    } else {
        SDL_PixelFormatEnum::SDL_PIXELFORMAT_ABGR8888
    };
    let surface = SDL_CreateRGBSurfaceWithFormatFrom(
        data as *mut _,
        width,
        height,
        components * 8,
        width * components,
        pixelFormat as u32,
    );
    if surface.is_null() {
        CFatal!("Failed to create custom cursor surface for window");
    }

    w.cursor = SDL_CreateColorCursor(surface, hotx, hoty);
    if w.cursor.is_null() {
        SDL_FreeSurface(surface);
        CFatal!("Failed to create custom cursor for window");
    }

    SDL_FreeSurface(surface);
    SDL_SetCursor(w.cursor);
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
