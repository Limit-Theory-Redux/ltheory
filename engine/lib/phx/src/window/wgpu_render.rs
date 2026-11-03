//! wgpu surface/device/queue creation from the winit 0.30 window: turns the
//! `winit::window::Window` into a wgpu instance/adapter/device/queue with a
//! configured swapchain. The bundle moves to the render thread, where
//! `WgpuCommandExecutor` owns it.
//!
//! Present-mode mapping lives in [`crate::window::present_mode`]
//! (`Vsync -> Fifo`, `NoVsync -> AutoNoVsync`).

use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use tracing::info;

use crate::window::PresentMode;

const WGPU_STARTUP_DEADLINE: Duration = Duration::from_secs(10);

/// Poll a wgpu startup future with an application-level deadline. Dropping a
/// pending future bounds Rust-owned startup state; it cannot interrupt a
/// foreign driver call that is already executing inside one `poll`.
pub(crate) fn poll_startup_future<F: Future>(future: F) -> Result<F::Output, &'static str> {
    let mut future = Box::pin(future);
    let mut context = Context::from_waker(Waker::noop());
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
    /// No adapter matched (no Vulkan/Metal/DX12 driver, or surface incompatible).
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

/// Owned wgpu startup handles, transferred to the render thread. The surface
/// holds a reference to the window, so neither outlives the other.
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
        // Timestamps at the pass boundaries (GPU time per pass), unless
        // LTHEORY_GPU_TIMING=0.
        | crate::render::wgpu_timing_features(available)
}

/// Create instance → adapter → device/queue → surface (owned) from the winit
/// window, and configure the swapchain for `present_mode`.
pub fn create_surface_bundle(
    window: Arc<winit::window::Window>,
    present_mode: PresentMode,
    width: u32,
    height: u32,
) -> Result<WgpuStartupBundle, WgpuError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        // WGPU_BACKEND (e.g. `dx12`) and the WGPU_VALIDATION/WGPU_DEBUG
        // variables override the defaults (diagnostics). The default is the
        // native API of each platform: Vulkan, Metal or DX12.
        backends: wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY),
        flags: wgpu::InstanceFlags::default().with_env(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    });

    let surface = instance
        .create_surface(window)
        .map_err(|e| WgpuError::CreateSurface(e.to_string()))?;

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
    // (already display-referred) pixels the engine renders, so take the
    // linear twin of the format.
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
