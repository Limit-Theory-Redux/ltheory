//! wgpu surface/device/queue creation from the existing winit 0.30 window.
//!
//! Stage 3 of the wgpu migration: this is the wgpu counterpart of
//! `glutin_render` + `window_gl_context` — it turns the same `winit::window::Window`
//! into a wgpu instance/adapter/device/queue with a configured swapchain, and
//! can present frames. The active renderer drives the owned startup bundle
//! through `WgpuCommandExecutor`; the legacy `WgpuRenderer::render_frame`
//! helper remains for the isolated surface smoke test.
//!
//! Present-mode mapping lives in [`crate::window::present_mode`]
//! (`Vsync -> Fifo`, `NoVsync -> Immediate`, mirroring the GL SwapInterval
//! Wait/DontWait semantics).

use crate::window::PresentMode;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};
use tracing::info;

const WGPU_STARTUP_DEADLINE: Duration = Duration::from_secs(10);

struct StartupWaker;

impl Wake for StartupWaker {
    fn wake(self: Arc<Self>) {}
}

/// Poll a wgpu startup future with an application-level deadline. Dropping a
/// pending future bounds Rust-owned startup state; it cannot interrupt a
/// foreign driver call that is already executing inside one `poll`.
pub(crate) fn poll_startup_future<F: Future>(future: F) -> Result<F::Output, &'static str> {
    let mut future = Box::pin(future);
    let waker: Waker = Arc::new(StartupWaker).into();
    let mut context = Context::from_waker(&waker);
    let deadline = Instant::now() + WGPU_STARTUP_DEADLINE;

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending if Instant::now() >= deadline => {
                return Err("wgpu startup future exceeded its deadline");
            }
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

/// Errors from wgpu surface/device creation and presentation.
#[derive(Debug)]
pub enum WgpuError {
    /// `Instance::create_surface` failed (unsupported window/backend).
    CreateSurface(String),
    /// No adapter matched (no Vulkan/DX12 driver, or surface incompatible).
    Adapter(String),
    /// `request_device` failed.
    Device(String),
    /// The surface offered no default configuration.
    NoDefaultConfig,
    /// Frame acquisition/presentation failed (see message).
    Present(String),
}

impl std::fmt::Display for WgpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuError::CreateSurface(e) => write!(f, "wgpu surface creation failed: {e}"),
            WgpuError::Adapter(e) => write!(f, "no suitable wgpu adapter: {e}"),
            WgpuError::Device(e) => write!(f, "wgpu device request failed: {e}"),
            WgpuError::NoDefaultConfig => write!(f, "surface offered no default configuration"),
            WgpuError::Present(e) => write!(f, "wgpu present failed: {e}"),
        }
    }
}

impl std::error::Error for WgpuError {}

/// Owns the wgpu instance/adapter/device/queue and the window's swapchain.
///
/// Borrows the winit window for the surface's lifetime. This helper is kept
/// separate from the active owned-bundle renderer for smoke testing.
pub struct WgpuRenderer<'a> {
    #[expect(dead_code)] // kept for re-adapting on surface loss
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    frames_presented: u64,
}

/// Owned wgpu startup handles, transferred to the application-thread backend
/// owner.
///
/// Unlike [`WgpuRenderer`] this borrows nothing at the Rust type level: the
/// surface uses raw window handles and therefore requires the `Engine` field
/// order to drop the renderer before `WinitWindow`.
pub struct WgpuStartupBundle {
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
}

/// The optional features the engine's executor uses when the adapter has them:
/// filtering and blending of 32-bit float textures (`R32F`/`RGBA32F` render
/// targets and data textures keep their precision instead of living in 16F),
/// 16-bit normalized formats and wireframe polygons.
pub fn wanted_features(available: wgpu::Features) -> wgpu::Features {
    available
        & (wgpu::Features::FLOAT32_FILTERABLE
            | wgpu::Features::FLOAT32_BLENDABLE
            | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
            | wgpu::Features::POLYGON_MODE_LINE)
}

/// Create instance → adapter → device/queue → surface (owned) from the winit
/// window, and configure the swapchain for `present_mode`.
#[allow(unsafe_code)] // raw-handle surface; safety contract documented below
pub fn create_surface_bundle(
    window: &winit::window::Window,
    present_mode: PresentMode,
    width: u32,
    height: u32,
) -> Result<WgpuStartupBundle, WgpuError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    });

    // winit 0.30 windows are NOT Clone (owned, Drop-closes), so the unsafe
    // raw-handle route is used for a 'static Surface. `Engine` declares the
    // renderer before `WinitWindow`, making the surface drop first.
    //
    // Safety: the raw handles must remain valid until the Surface is dropped.
    // The engine owns both the window (inside WinitWindow) and this surface;
    // declaration order makes the renderer/backend drop before WinitWindow.
    let surface: wgpu::Surface<'static> = {
        use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
        let window_handle = window
            .window_handle()
            .map_err(|e| WgpuError::CreateSurface(e.to_string()))?;
        let display_handle = window.display_handle().ok().map(|d| d.as_raw());
        unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: display_handle,
                raw_window_handle: window_handle.as_raw(),
            })
        }
        .map_err(|e| WgpuError::CreateSurface(e.to_string()))?
    };

    let adapter = poll_startup_future(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
        apply_limit_buckets: true,
    }))
    .map_err(|e| WgpuError::Adapter(e.to_string()))?
    .map_err(|e| WgpuError::Adapter(e.to_string()))?;

    let adapter_info = adapter.get_info();
    info!(
        "wgpu adapter: {} ({:?}, {:?})",
        adapter_info.name, adapter_info.backend, adapter_info.device_type
    );

    let (device, queue) = poll_startup_future(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("phx-wgpu-device"),
        required_features: wanted_features(adapter.features()),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    }))
    .map_err(|e| WgpuError::Device(e.to_string()))?
    .map_err(|e| WgpuError::Device(e.to_string()))?;

    let mut config = surface
        .get_default_config(&adapter, width, height)
        .ok_or(WgpuError::NoDefaultConfig)?;
    // The default config prefers an sRGB format, which would gamma-encode the
    // (already display-referred) pixels the engine renders. The GL default
    // framebuffer is not sRGB either, so take the linear twin of the format.
    let capabilities = surface.get_capabilities(&adapter);
    let linear = config.format.remove_srgb_suffix();
    if capabilities.formats.contains(&linear) {
        config.format = linear;
    }
    config.present_mode = present_mode.into();
    // The backbuffer can be read back (screenshots, captures) where the
    // surface allows copying from it.
    if surface
        .get_capabilities(&adapter)
        .usages
        .contains(wgpu::TextureUsages::COPY_SRC)
    {
        config.usage |= wgpu::TextureUsages::COPY_SRC;
    }
    surface.configure(&device, &config);
    info!(
        "wgpu surface configured: {}x{} format {:?} present {:?}",
        width, height, config.format, config.present_mode
    );

    Ok(WgpuStartupBundle {
        adapter,
        surface,
        device,
        queue,
        surface_config: config,
    })
}

impl<'a> WgpuRenderer<'a> {
    /// Create instance → adapter → device/queue → surface from the winit
    /// window, and configure the swapchain for `present_mode`.
    pub fn new(
        window: &'a winit::window::Window,
        present_mode: PresentMode,
        width: u32,
        height: u32,
    ) -> Result<Self, WgpuError> {
        let bundle = create_surface_bundle(window, present_mode, width, height)?;
        Ok(Self {
            adapter: bundle.adapter,
            device: bundle.device,
            queue: bundle.queue,
            surface: bundle.surface,
            config: bundle.surface_config,
            size: (width, height),
            frames_presented: 0,
        })
    }

    /// Re-configure the swapchain after a window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        if (width, height) == self.size {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.size = (width, height);
    }

    /// Acquire a frame, clear it to `color`, and present.
    pub fn render_frame(&mut self, color: [f32; 4]) -> Result<(), WgpuError> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                // Swapchain was invalidated (resize/minimize) — reconfigure and
                // let the caller decide whether to retry this frame.
                self.surface.configure(&self.device, &self.config);
                return Err(WgpuError::Present("surface outdated/lost".to_string()));
            }
            other => {
                return Err(WgpuError::Present(format!("frame acquisition: {other:?}")));
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("phx-wgpu-frame"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("phx-wgpu-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: color[0] as f64,
                            g: color[1] as f64,
                            b: color[2] as f64,
                            a: color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame); // wgpu 30: present moved to the queue
        self.frames_presented += 1;
        Ok(())
    }

    /// Number of successfully presented frames (smoke-test evidence).
    pub fn frames_presented(&self) -> u64 {
        self.frames_presented
    }

    /// Swapchain format (useful for pipeline creation in the shader stage).
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real-window smoke test: opens a winit window, creates the wgpu
    /// surface/device/queue, presents 30 frames, and drops cleanly.
    /// Ignored by default (opens a visible window); run with:
    /// `cargo test -p phx --lib wgpu_window_smoke -- --ignored --nocapture`
    #[test]
    #[ignore = "opens a real window; run explicitly"]
    #[allow(deprecated)] // EventLoop::create_window predates the ActiveEventLoop API
    fn wgpu_window_smoke_presents_frames() {
        use winit::window::WindowAttributes;

        // Tests run on a worker thread; winit requires main-thread event loops
        // unless explicitly opted out (Windows).
        #[cfg(target_os = "windows")]
        let event_loop = {
            use winit::platform::windows::EventLoopBuilderExtWindows;
            winit::event_loop::EventLoopBuilder::new()
                .with_any_thread(true)
                .build()
                .expect("create winit event loop")
        };
        #[cfg(not(target_os = "windows"))]
        let event_loop = {
            use winit::event_loop::EventLoop;
            EventLoop::new().expect("create winit event loop")
        };
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("phx wgpu smoke test")
                    .with_inner_size(winit::dpi::LogicalSize::new(320.0, 240.0)),
            )
            .expect("create winit window");

        let mut renderer =
            WgpuRenderer::new(&window, PresentMode::Vsync, 320, 240).expect("create wgpu renderer");

        // Rainbow sweep — proves each frame actually presents.
        for i in 0..30u32 {
            let t = i as f64 / 30.0;
            let color = [
                (0.5 + 0.5 * (t * std::f64::consts::TAU).sin()) as f32,
                (0.5 + 0.5 * (t * std::f64::consts::TAU + 2.094).sin()) as f32,
                (0.5 + 0.5 * (t * std::f64::consts::TAU + 4.189).sin()) as f32,
                1.0,
            ];
            renderer
                .render_frame(color)
                .expect("present frame without crash");
        }

        assert_eq!(renderer.frames_presented(), 30);
        println!(
            "OK: presented {} frames at {:?} on {:?}",
            renderer.frames_presented(),
            renderer.surface_format(),
            renderer.size,
        );
    }
}
