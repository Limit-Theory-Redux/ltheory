use crate::window::PresentMode;
use super::*;

// Low level renderer class, managing WGPU command buffers and render passes.
#[derive(Debug)]
pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    frame: Option<OpaqueFrame>,

    current_shader: Option<*const Shader>,
}

impl Renderer {
    pub fn new(winit_window: &winit::window::Window, present_mode: PresentMode) -> Renderer {
        // Set up WGPU.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&winit_window) }.unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        ))
        .unwrap();

        let size = winit_window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: present_mode.into(),
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Renderer {
            surface,
            device,
            queue,
            config,
            frame: None,
            current_shader: None,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn begin_frame<'a>(&mut self) -> Result<Box<Frame<'a>>, wgpu::SurfaceError> {
        // Create texture view for the current texture in the swapchain.
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Set up the command encoder.
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut frame = Box::new(Frame {
            output,
            output_view: view,
            encoder,
            render_pass: None,
        });

        self.frame = Some(OpaqueFrame::new(&mut *frame));

        Ok(frame)
    }

    pub fn end_frame(&mut self, frame: Box<Frame>) {
        // Deconstruct the Frame object.
        self.frame = None;
        let Frame {
            output,
            output_view,
            encoder,
            mut render_pass,
        } = *frame;

        // Finish the current render pass.
        render_pass = None;

        // Submit the queue and present the output.
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // per-frame resources get freed at the end of this scope.
    }

    pub fn get_frame(&mut self) -> &mut Frame<'_> {
        self.frame
            .as_mut()
            .expect("Cannot access frame object outside AppFrame")
            .get_mut()
    }

    pub fn use_shader(&mut self, sh: Option<&Shader>) {
        self.current_shader = sh.map(|r| r as *const _);
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.device
    }
}

// Rather than infect Renderer and the surrounding classes with lifetime specifiers, use a bit of pointer trickery to hide the lifetime. This exposes us to potential undefined behaviour, but we combat this by ensuring that OpaqueFrame's lifetime is <= Frame's lifetime.
#[derive(Debug)]
pub struct OpaqueFrame {
    ptr: *mut (),
}

impl OpaqueFrame {
    pub fn new<'a>(frame: &mut Frame<'a>) -> OpaqueFrame {
        OpaqueFrame {
            ptr: frame as *mut Frame<'a> as *mut (),
        }
    }

    pub fn get(&self) -> &Frame<'_> {
        unsafe { &*(self.ptr as *mut Frame<'_>) }
    }

    pub fn get_mut(&self) -> &mut Frame<'_> {
        unsafe { &mut *(self.ptr as *mut Frame<'_>) }
    }
}

// Per-frame resources.
#[derive(Debug)]
pub struct Frame<'a> {
    pub output: wgpu::SurfaceTexture,
    pub output_view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,

    // Any pending render passes must have a lifetime <= the lifetime of the CommandEncoder, which is 'a.
    pub render_pass: Option<wgpu::RenderPass<'a>>,
}

impl<'a> Frame<'a> {
    pub fn start_render_pass(&'a mut self, color: wgpu::Color) {
        //-> &mut wgpu::RenderPass<'a>  {
        {
            let pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view, // TODO: This should be the current render target (?)
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.render_pass = Some(pass);
        }
        // self.current_render_pass()
    }

    pub fn current_render_pass(&mut self) -> &mut wgpu::RenderPass<'a> {
        // TODO: We should be able to not crash here by starting a default render pass with no clear operation if none is defined.
        self.render_pass
            .as_mut()
            .expect("Need to start a render pass by calling Draw.Clear first.")
    }
}
