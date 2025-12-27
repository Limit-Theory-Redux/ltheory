//! Global render mode configuration.
//!
//! This module provides a global flag to determine whether rendering should
//! use direct GL calls or emit commands to the render thread.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use super::{RenderCommand, RenderThreadHandle, ResourceId};

/// Global flag indicating if command mode (render thread) is active
static COMMAND_MODE: AtomicBool = AtomicBool::new(false);

/// Global flag indicating GL context is unavailable (lost after render thread shutdown)
static GL_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// Global render thread handle (Arc for safe sharing, RwLock for mutation)
static RENDER_HANDLE: OnceLock<RwLock<Option<Arc<RenderThreadHandle>>>> = OnceLock::new();

/// Global counter for generating unique ResourceIds
static NEXT_RESOURCE_ID: AtomicU64 = AtomicU64::new(1);

fn get_handle_lock() -> &'static RwLock<Option<Arc<RenderThreadHandle>>> {
    RENDER_HANDLE.get_or_init(|| RwLock::new(None))
}

/// Generate a unique ResourceId for command mode resource creation
pub fn next_resource_id() -> ResourceId {
    ResourceId(NEXT_RESOURCE_ID.fetch_add(1, Ordering::Relaxed))
}

// =============================================================================
// FFI-exposed RenderThread control API
// =============================================================================

/// Render thread control - provides FFI interface for Lua to manage render mode
pub struct RenderThread;

#[luajit_ffi_gen::luajit_ffi]
impl RenderThread {
    /// Check if command mode is enabled (render thread handles GL calls)
    #[bind(name = "IsCommandMode")]
    pub fn is_command_mode_ffi() -> bool {
        is_command_mode()
    }

    /// Check if GL context is available for rendering
    /// Returns false if GL context was lost (e.g., after render thread shutdown)
    #[bind(name = "IsGLAvailable")]
    pub fn is_gl_available_ffi() -> bool {
        !is_gl_unavailable()
    }

    /// Enable command mode
    /// Call this after the render thread has been started and GL context transferred
    #[bind(name = "EnableCommandMode")]
    pub fn enable_command_mode_ffi() {
        enable_command_mode();
    }

    /// Disable command mode
    /// Call this before stopping the render thread
    #[bind(name = "DisableCommandMode")]
    pub fn disable_command_mode_ffi() {
        disable_command_mode();
    }
}

/// Check if command mode is active (render thread is handling GL calls)
#[inline]
pub fn is_command_mode() -> bool {
    COMMAND_MODE.load(Ordering::Relaxed)
}

/// Enable command mode (called when render thread starts)
pub fn enable_command_mode() {
    COMMAND_MODE.store(true, Ordering::Relaxed);
}

/// Disable command mode (called when render thread stops)
pub fn disable_command_mode() {
    COMMAND_MODE.store(false, Ordering::Relaxed);
}

/// Check if GL context is unavailable (lost after render thread shutdown)
#[inline]
pub fn is_gl_unavailable() -> bool {
    GL_UNAVAILABLE.load(Ordering::Relaxed)
}

/// Mark GL context as unavailable (called when render thread cannot return context)
pub fn set_gl_unavailable() {
    GL_UNAVAILABLE.store(true, Ordering::Relaxed);
    tracing::warn!("GL context marked as unavailable - rendering will be skipped");
}

/// Clear GL unavailable flag (called when GL context is restored)
pub fn clear_gl_unavailable() {
    GL_UNAVAILABLE.store(false, Ordering::Relaxed);
}

/// Set the global render handle for command submission (safe version using Arc)
pub fn set_render_handle(handle: Arc<RenderThreadHandle>) {
    let lock = get_handle_lock();
    if let Ok(mut guard) = lock.write() {
        *guard = Some(handle);
    }
}

/// Clear the global render handle
pub fn clear_render_handle() {
    let lock = get_handle_lock();
    if let Ok(mut guard) = lock.write() {
        *guard = None;
    }
}

/// Submit a command to the render thread (if in command mode)
/// Returns true if the command was submitted, false if in direct GL mode
pub fn submit_command(cmd: RenderCommand) -> bool {
    if !is_command_mode() {
        return false;
    }

    let lock = get_handle_lock();
    if let Ok(guard) = lock.read() {
        if let Some(ref handle) = *guard {
            handle.submit(cmd);
            return true;
        } else {
            tracing::error!("submit_command: command mode enabled but no render handle set!");
        }
    } else {
        tracing::error!("submit_command: failed to acquire read lock!");
    }
    false
}

/// Submit a command to the render thread without blocking (if in command mode).
/// Returns true if the command was submitted, false if the channel was full or not in command mode.
/// Use this for commands that can be safely dropped (like resize events during rapid window resizing).
pub fn try_submit_command(cmd: RenderCommand) -> bool {
    if !is_command_mode() {
        return false;
    }

    let lock = get_handle_lock();
    if let Ok(guard) = lock.read() {
        if let Some(ref handle) = *guard {
            return handle.try_submit(cmd);
        }
    }
    false
}

/// Submit multiple commands to the render thread
pub fn submit_commands(cmds: impl IntoIterator<Item = RenderCommand>) -> bool {
    if !is_command_mode() {
        return false;
    }

    let lock = get_handle_lock();
    if let Ok(guard) = lock.read() {
        if let Some(ref handle) = *guard {
            for cmd in cmds {
                handle.submit(cmd);
            }
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_mode_toggle() {
        assert!(!is_command_mode());
        enable_command_mode();
        assert!(is_command_mode());
        disable_command_mode();
        assert!(!is_command_mode());
    }
}
