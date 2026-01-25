use std::num::NonZeroU32;

use glutin::context::{NotCurrentContext, PossiblyCurrentContext};
use glutin::prelude::{NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::{GlSurface, Surface, WindowSurface};

use crate::window::WindowError;

/// Data needed to initialize GL context on the render thread.
/// This is extracted from WinitWindow and sent to the render thread.
/// Contains both context and the already-created surface (macOS requires
/// surfaces to be created on the main thread).
pub struct WindowGlContext {
    /// The not-current GL context that can be made current on render thread
    pub context: NotCurrentContext,
    /// The surface (already created on main thread)
    pub surface: Surface<WindowSurface>,
}

// Safety: NotCurrentContext and Surface are both Send when not current
// TODO: unsafe impl Send for RenderThreadGlContext {}

impl WindowGlContext {
    /// Make the context current on the render thread and return the active context + surface.
    /// This should only be called once from the render thread.
    pub fn make_current(self) -> Result<WindowActiveGlContext, WindowError> {
        let context = self.context.make_current(&self.surface)?;

        Ok(WindowActiveGlContext {
            context,
            surface: self.surface,
        })
    }
}

/// Active GL context on the render thread.
pub struct WindowActiveGlContext {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
}

impl WindowActiveGlContext {
    /// Swap buffers
    pub fn swap_buffers(&self) -> Result<(), WindowError> {
        self.surface.swap_buffers(&self.context)?;
        Ok(())
    }

    /// Resize the surface
    pub fn resize(&self, width: u32, height: u32) {
        if let Some(width) = NonZeroU32::new(width) {
            if let Some(height) = NonZeroU32::new(height) {
                self.surface.resize(&self.context, width, height);
            }
        }
    }

    /// Make the context not current (for cleanup)
    pub fn make_not_current(self) -> Result<NotCurrentContext, WindowError> {
        let context = self.context.make_not_current()?;
        Ok(context)
    }

    /// Make context not current and return both context and surface for transfer back to main thread.
    /// This is used when shutting down the render thread to return the GL resources.
    ///
    /// On macOS, make_not_current() uses run_on_main() which deadlocks when main thread
    /// is blocked waiting for this function to complete. So we skip it on macOS and just
    /// drop the context (it will be destroyed when the thread exits).
    #[cfg(not(target_os = "macos"))]
    pub fn release_for_main_thread(
        self,
    ) -> Result<(NotCurrentContext, Surface<WindowSurface>), WindowError> {
        let context = self.context.make_not_current()?;
        Ok((context, self.surface))
    }

    #[cfg(target_os = "macos")]
    pub fn release_for_main_thread(
        self,
    ) -> Result<(NotCurrentContext, Surface<WindowSurface>), WindowError> {
        // On macOS, make_not_current() uses run_on_main() which deadlocks when main
        // thread is blocked waiting for this function.
        //
        // We cannot safely release the context back to main thread without causing
        // a deadlock. Use std::mem::forget to skip destructors (which might also
        // use run_on_main). The GL resources will be leaked but cleaned up by the
        // OS when the process exits.
        //
        // This means we cannot restore direct GL mode on macOS after using the
        // render thread, but that's an acceptable limitation.
        std::mem::forget(self.context);
        std::mem::forget(self.surface);
        Err(WindowError::MacOsReleaseContext)
    }
}
