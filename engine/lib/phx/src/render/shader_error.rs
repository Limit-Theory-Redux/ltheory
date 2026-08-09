//! Shader compile/reload error queue.
//!
//! Lets a shader that fails to compile keep rendering (with the previous
//! program, or a placeholder on first load) while surfacing the failure to
//! an in-game overlay instead of only a log line.

use std::collections::VecDeque;

use crate::render::Renderer;

/// Maximum number of errors retained; oldest is evicted first.
const MAX_ERRORS: usize = 10;

/// A single shader compilation or link error.
#[derive(Clone, Debug)]
pub struct ShaderErrorInfo {
    pub shader_key: String,
    pub error_type: String,
    pub message: String,
    pub timestamp: u64,
}

/// Owned by `RendererData` (see `render/thread/renderer_data.rs`) and reached
/// via `r.data.shader_errors` - no `static`/global, matching `ShaderVarMap`.
pub struct ShaderErrorQueue {
    errors: VecDeque<ShaderErrorInfo>,
    frame_counter: u64,
    has_new_errors: bool,
}

impl ShaderErrorQueue {
    pub fn new() -> Self {
        Self {
            errors: VecDeque::new(),
            frame_counter: 0,
            has_new_errors: false,
        }
    }

    pub(crate) fn push(&mut self, shader_key: &str, error_type: &str, message: &str) {
        if self.errors.len() >= MAX_ERRORS {
            self.errors.pop_front();
        }

        // OpenGL error strings may contain null bytes.
        let clean_message = message.replace('\0', "");

        tracing::error!("Shader {error_type} error for '{shader_key}': {clean_message}");

        self.errors.push_back(ShaderErrorInfo {
            shader_key: shader_key.to_string(),
            error_type: error_type.to_string(),
            message: clean_message,
            timestamp: self.frame_counter,
        });
        self.has_new_errors = true;
    }
}

impl Default for ShaderErrorQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// FFI wrapper for the shader error queue.
pub struct ShaderError;

#[luajit_ffi_gen::luajit_ffi]
impl ShaderError {
    /// Returns the number of shader errors in the queue.
    pub fn get_count(r: &Renderer) -> i32 {
        r.data.shader_errors.errors.len() as i32
    }

    /// Returns whether there are new (unacknowledged) errors.
    pub fn has_new_errors(r: &Renderer) -> bool {
        r.data.shader_errors.has_new_errors
    }

    /// Acknowledges all current errors (clears the "new" flag).
    pub fn acknowledge_errors(r: &mut Renderer) {
        r.data.shader_errors.has_new_errors = false;
    }

    /// Gets the shader key for the error at index (0-based).
    pub fn get_shader_key(r: &Renderer, index: i32) -> Option<String> {
        r.data
            .shader_errors
            .errors
            .get(index as usize)
            .map(|e| e.shader_key.clone())
    }

    /// Gets the error type for the error at index ("compile" or "link").
    pub fn get_error_type(r: &Renderer, index: i32) -> Option<String> {
        r.data
            .shader_errors
            .errors
            .get(index as usize)
            .map(|e| e.error_type.clone())
    }

    /// Gets the error message for the error at index.
    pub fn get_message(r: &Renderer, index: i32) -> Option<String> {
        r.data
            .shader_errors
            .errors
            .get(index as usize)
            .map(|e| e.message.clone())
    }

    /// Gets the timestamp for the error at index.
    pub fn get_timestamp(r: &Renderer, index: i32) -> u64 {
        r.data
            .shader_errors
            .errors
            .get(index as usize)
            .map(|e| e.timestamp)
            .unwrap_or(0)
    }

    /// Clears all errors from the queue.
    pub fn clear(r: &mut Renderer) {
        r.data.shader_errors.errors.clear();
        r.data.shader_errors.has_new_errors = false;
    }

    /// Clears a specific error by index.
    pub fn clear_at(r: &mut Renderer, index: i32) {
        let errors = &mut r.data.shader_errors.errors;
        if (index as usize) < errors.len() {
            errors.remove(index as usize);
        }
    }

    /// Clears all errors for a specific shader key.
    pub fn clear_for_shader(r: &mut Renderer, shader_key: &str) {
        let queue = &mut r.data.shader_errors;
        queue.errors.retain(|e| e.shader_key != shader_key);
        if queue.errors.is_empty() {
            queue.has_new_errors = false;
        }
    }

    /// Called each frame to update internal state.
    pub fn update(r: &mut Renderer) {
        r.data.shader_errors.frame_counter += 1;
    }

    /// Gets the most recent error message (for quick display).
    pub fn get_latest_message(r: &Renderer) -> Option<String> {
        r.data
            .shader_errors
            .errors
            .back()
            .map(|e| format!("[{}] {}: {}", e.error_type, e.shader_key, e.message))
    }

    /// Gets the most recent shader key that had an error.
    pub fn get_latest_shader_key(r: &Renderer) -> Option<String> {
        r.data
            .shader_errors
            .errors
            .back()
            .map(|e| e.shader_key.clone())
    }
}
