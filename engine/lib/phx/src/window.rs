use crate::common::*;
use crate::internal::*;
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

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Window {
    #[bind(name = "Create")]
    pub fn new(
        title: &str,
        x: WindowPos,
        y: WindowPos,
        sx: i32,
        sy: i32,
        mode: WindowMode,
    ) -> Self {
        unsafe {
            let modeComplete = mode | SDL_WindowFlags::SDL_WINDOW_OPENGL as WindowMode;
            // | SDL_WindowFlags::SDL_WINDOW_ALLOW_HIGHDPI as WindowMode;
            let handle = SDL_CreateWindow(
                title.as_ptr() as *const libc::c_char,
                x,
                y,
                sx,
                sy,
                modeComplete,
            );
            let context = SDL_GL_CreateContext(handle);
            if context.is_null() {
                CFatal!("Failed to create OpenGL context for window");
            }
            OpenGL_Init();

            Self {
                handle,
                context,
                mode: modeComplete,
                cursor: std::ptr::null_mut(),
            }
        }
    }

    pub fn begin_draw(&self) {
        unsafe {
            let mut size: IVec2 = IVec2::ZERO;

            SDL_GL_MakeCurrent(self.handle, self.context);
            self.get_size(&mut size);
            Viewport_Push(0, 0, size.x, size.y, true);
        }
    }

    pub fn end_draw(&self) {
        unsafe {
            Viewport_Pop();
            SDL_GL_SwapWindow(self.handle);
        }
    }

    pub fn get_position(&self, out: &mut IVec2) {
        unsafe {
            SDL_GetWindowPosition(self.handle, &mut out.x, &mut out.y);
        }
    }

    pub fn get_size(&self, out: &mut IVec2) {
        unsafe {
            SDL_GetWindowSize(self.handle, &mut out.x, &mut out.y);
        }
    }

    pub fn get_title(&self) -> String {
        unsafe { SDL_GetWindowTitle(self.handle) }.as_string()
    }

    pub fn set_fullscreen(&self, fs: bool) {
        unsafe {
            SDL_SetWindowFullscreen(self.handle, if fs { WindowMode_Fullscreen } else { 0 });
        }
    }

    pub fn set_position(&self, x: WindowPos, y: WindowPos) {
        unsafe {
            SDL_SetWindowPosition(self.handle, x, y);
        }
    }

    pub fn set_size(&self, sx: i32, sy: i32) {
        unsafe {
            SDL_SetWindowSize(self.handle, sx, sy);
        }
    }

    pub fn set_title(&self, title: &str) {
        unsafe {
            SDL_SetWindowTitle(self.handle, title.as_ptr() as *const libc::c_char);
        }
    }

    pub fn set_vsync(&self, vsync: bool) {
        unsafe {
            SDL_GL_SetSwapInterval(if vsync { 1 } else { 0 });
        }
    }

    pub fn set_cursor(&mut self, name: &str, hotx: i32, hoty: i32) {
        unsafe {
            SDL_FreeCursor(self.cursor); // Can take null.

            let path = Resource_GetPath(ResourceType_Tex2D, name.as_ptr() as *const libc::c_char);

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

            self.cursor = SDL_CreateColorCursor(surface, hotx, hoty);
            if self.cursor.is_null() {
                SDL_FreeSurface(surface);
                CFatal!("Failed to create custom cursor for window");
            }

            SDL_FreeSurface(surface);
            SDL_SetCursor(self.cursor);
        }
    }

    pub fn set_mouse_position(&self, position: &IVec2) {
        unsafe {
            SDL_WarpMouseInWindow(self.handle, position.x, position.y);
        }
    }

    pub fn set_window_grab(&self, grabbed: bool) {
        unsafe {
            SDL_SetWindowGrab(
                self.handle,
                if grabbed {
                    SDL_bool::SDL_TRUE
                } else {
                    SDL_bool::SDL_FALSE
                },
            );
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        unsafe {
            if self.mode & WindowMode_Fullscreen != 0 {
                SDL_SetWindowFullscreen(self.handle, 0);
            } else {
                SDL_SetWindowFullscreen(self.handle, WindowMode_Fullscreen);
            }
            self.mode ^= WindowMode_Fullscreen;
        }
    }

    pub fn hide(&self) {
        unsafe {
            SDL_HideWindow(self.handle);
        }
    }

    pub fn show(&self) {
        unsafe {
            SDL_ShowWindow(self.handle);
        }
    }
}
