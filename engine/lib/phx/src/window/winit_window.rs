use std::num::NonZeroU32;
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct WinitWindow {
    window: winit::window::Window,
}

impl WinitWindow {
    pub fn new(
        window: winit::window::Window,
    ) -> Self {
        Self {
            window,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resume(&mut self) {
        debug!("WinitWindow::resume");
    }

    pub fn suspend(&mut self) {
        debug!("WinitWindow::suspend");
    }

    pub fn resize(&self, width: u32, height: u32) {
        if let Some(width) = NonZeroU32::new(width) {
            if let Some(height) = NonZeroU32::new(height) {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                // if let GlState::Current { context, surface } = &self.gl_state {
                //     surface.resize(context, width, height);

                //     glutin_render::resize(width.get() as i32, height.get() as i32);
                // }
            }
        }
    }

    pub fn redraw(&self) {
        // if let GlState::Current { context, surface } = &self.gl_state {
        //     self.window.request_redraw();

        //     surface.swap_buffers(context).expect("Cannot redraw");
        // }
    }
}
