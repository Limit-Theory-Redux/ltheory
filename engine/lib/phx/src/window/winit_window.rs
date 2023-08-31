use std::num::NonZeroU32;

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlDisplay, NotCurrentGlContextSurfaceAccessor, PossiblyCurrentGlContext};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::GlWindow;
use tracing::{error, warn};

use crate::window::glutin_render;

#[derive(Debug)]
enum GlState {
    Current {
        context: PossiblyCurrentContext,
        surface: Surface<WindowSurface>,
    },
    NotCurrent {
        context: NotCurrentContext,
    },
    Undefined,
}

impl GlState {
    fn make_current(&mut self, surface: Surface<WindowSurface>) -> bool {
        let old_self = std::mem::replace(self, Self::Undefined);

        match old_self {
            Self::Current { .. } => {
                warn!("Context is already current");

                *self = old_self;

                false
            }
            Self::NotCurrent { context } => {
                let context = context
                    .make_current(&surface)
                    .expect("Cannot make context current");

                // Try setting vsync.
                if let Err(res) = surface
                    .set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    warn!("Error setting vsync: {res:?}");
                }

                *self = Self::Current { context, surface };

                true
            }
            Self::Undefined => panic!("Context is undefined"),
        }
    }

    fn make_not_current(&mut self) -> bool {
        let old_self = std::mem::replace(self, Self::Undefined);

        match old_self {
            Self::Current { context, .. } => {
                let context = context
                    .make_not_current()
                    .expect("Cannot make context not current");

                *self = Self::NotCurrent { context };

                true
            }
            Self::NotCurrent { .. } => {
                warn!("Context is already not current");

                *self = old_self;

                false
            }
            Self::Undefined => panic!("Context is undefined"),
        }
    }
}

#[derive(Debug)]
pub struct WinitWindow {
    window: winit::window::Window,
    gl_config: glutin::config::Config,
    gl_state: GlState,
}

impl WinitWindow {
    pub fn new(
        window: winit::window::Window,
        gl_config: glutin::config::Config,
        context: NotCurrentContext,
    ) -> Self {
        glutin_render::init_renderer(&gl_config.display());

        Self {
            window,
            gl_config,
            gl_state: GlState::NotCurrent { context },
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resume(&mut self) {
        let attrs = self.window.build_surface_attributes(<_>::default());
        let gl_surface = unsafe {
            self.gl_config
                .display()
                .create_window_surface(&self.gl_config, &attrs)
                .unwrap()
        };

        if self.gl_state.make_current(gl_surface) {
            // The context needs to be current for the Renderer to set up shaders and
            // buffers. It also performs function loading, which needs a current context on
            // WGL.
            glutin_render::init_renderer(&self.gl_config.display());
        }
    }

    pub fn suspend(&mut self) {
        self.gl_state.make_not_current();
    }

    pub fn resize(&self, width: u32, height: u32) {
        if let Some(width) = NonZeroU32::new(width) {
            if let Some(height) = NonZeroU32::new(height) {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let GlState::Current { context, surface } = &self.gl_state {
                    surface.resize(context, width, height);

                    glutin_render::resize(width.get() as i32, height.get() as i32);
                }
            }
        }
    }

    pub fn redraw(&self) {
        if let GlState::Current { context, surface } = &self.gl_state {
            self.window.request_redraw();

            surface.swap_buffers(context).expect("Cannot redraw");
        }
    }
}
