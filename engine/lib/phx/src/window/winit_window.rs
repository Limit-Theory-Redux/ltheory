use std::num::NonZeroU32;
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct WinitWindow {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: winit::window::Window,
}

impl WinitWindow {
    pub fn new(
        surface: wgpu::Surface,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        size: winit::dpi::PhysicalSize<u32>,
        window: winit::window::Window,
    ) -> Self {
        Self {
            surface,
            device,
            queue,
            config,
            size,
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

    pub fn resize(&mut self, width: u32, height: u32) {
        if width != 0 && height != 0 {
            self.size.width = width;
            self.size.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn redraw(&mut self) {
        // if let GlState::Current { context, surface } = &self.gl_state {
        //     self.window.request_redraw();

        //     surface.swap_buffers(context).expect("Cannot redraw");
        // }
    }
}
