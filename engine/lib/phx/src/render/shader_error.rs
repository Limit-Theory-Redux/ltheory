//! Shader error handling with graceful fallback support.
//!
//! Provides a global error queue for shader compilation failures, allowing
//! the application to display errors to the user while continuing to use
//! the last working shader.

use std::sync::Mutex;
use std::collections::VecDeque;

/// Maximum number of errors to keep in the queue
const MAX_ERRORS: usize = 10;

/// A shader compilation or link error
#[derive(Clone, Debug)]
pub struct ShaderErrorInfo {
    /// Shader name/key (e.g., "vertex/wvp:fragment/material/solidcolor")
    pub shader_key: String,
    /// Type of error: "compile" or "link"
    pub error_type: String,
    /// The actual error message from OpenGL
    pub message: String,
    /// Timestamp when the error occurred (frame number or monotonic time)
    pub timestamp: u64,
}

/// Global shader error queue
static SHADER_ERRORS: Mutex<ShaderErrorState> = Mutex::new(ShaderErrorState::new());

struct ShaderErrorState {
    errors: VecDeque<ShaderErrorInfo>,
    frame_counter: u64,
    /// Whether there are unacknowledged errors
    has_new_errors: bool,
}

impl ShaderErrorState {
    const fn new() -> Self {
        Self {
            errors: VecDeque::new(),
            frame_counter: 0,
            has_new_errors: false,
        }
    }
}

/// Internal: Push a shader error to the queue
pub fn push_shader_error(shader_key: &str, error_type: &str, message: &str) {
    let mut state = SHADER_ERRORS.lock().unwrap();

    // Remove oldest if at capacity
    if state.errors.len() >= MAX_ERRORS {
        state.errors.pop_front();
    }

    // Strip null bytes from message (OpenGL error strings may contain them)
    let clean_message = message.replace('\0', "");

    let timestamp = state.frame_counter;
    state.errors.push_back(ShaderErrorInfo {
        shader_key: shader_key.to_string(),
        error_type: error_type.to_string(),
        message: clean_message.clone(),
        timestamp,
    });
    state.has_new_errors = true;

    tracing::error!(
        "Shader {} error for '{}': {}",
        error_type,
        shader_key,
        clean_message
    );
}

/// FFI wrapper for shader error system
pub struct ShaderError;

#[luajit_ffi_gen::luajit_ffi]
impl ShaderError {
    /// Returns the number of shader errors in the queue
    #[bind(name = "GetCount")]
    pub fn get_count() -> i32 {
        SHADER_ERRORS.lock().unwrap().errors.len() as i32
    }

    /// Returns whether there are new (unacknowledged) errors
    #[bind(name = "HasNewErrors")]
    pub fn has_new_errors() -> bool {
        SHADER_ERRORS.lock().unwrap().has_new_errors
    }

    /// Acknowledges all current errors (clears the "new" flag)
    #[bind(name = "AcknowledgeErrors")]
    pub fn acknowledge_errors() {
        SHADER_ERRORS.lock().unwrap().has_new_errors = false;
    }

    /// Gets the shader key for error at index (0-based)
    #[bind(name = "GetShaderKey")]
    pub fn get_shader_key(index: i32) -> Option<String> {
        let state = SHADER_ERRORS.lock().unwrap();
        state.errors.get(index as usize).map(|e| e.shader_key.clone())
    }

    /// Gets the error type for error at index ("compile" or "link")
    #[bind(name = "GetErrorType")]
    pub fn get_error_type(index: i32) -> Option<String> {
        let state = SHADER_ERRORS.lock().unwrap();
        state.errors.get(index as usize).map(|e| e.error_type.clone())
    }

    /// Gets the error message for error at index
    #[bind(name = "GetMessage")]
    pub fn get_message(index: i32) -> Option<String> {
        let state = SHADER_ERRORS.lock().unwrap();
        state.errors.get(index as usize).map(|e| e.message.clone())
    }

    /// Gets the timestamp for error at index
    #[bind(name = "GetTimestamp")]
    pub fn get_timestamp(index: i32) -> u64 {
        let state = SHADER_ERRORS.lock().unwrap();
        state.errors.get(index as usize).map(|e| e.timestamp).unwrap_or(0)
    }

    /// Clears all errors from the queue
    #[bind(name = "Clear")]
    pub fn clear() {
        let mut state = SHADER_ERRORS.lock().unwrap();
        state.errors.clear();
        state.has_new_errors = false;
    }

    /// Clears a specific error by index
    #[bind(name = "ClearAt")]
    pub fn clear_at(index: i32) {
        let mut state = SHADER_ERRORS.lock().unwrap();
        if (index as usize) < state.errors.len() {
            state.errors.remove(index as usize);
        }
    }

    /// Clears all errors for a specific shader key
    #[bind(name = "ClearForShader")]
    pub fn clear_for_shader(shader_key: &str) {
        let mut state = SHADER_ERRORS.lock().unwrap();
        state.errors.retain(|e| e.shader_key != shader_key);
        // If we cleared all errors, also clear the new flag
        if state.errors.is_empty() {
            state.has_new_errors = false;
        }
    }

    /// Called each frame to update internal state
    #[bind(name = "Update")]
    pub fn update() {
        SHADER_ERRORS.lock().unwrap().frame_counter += 1;
    }

    /// Gets the most recent error message (for quick display)
    #[bind(name = "GetLatestMessage")]
    pub fn get_latest_message() -> Option<String> {
        let state = SHADER_ERRORS.lock().unwrap();
        state.errors.back().map(|e| {
            format!("[{}] {}: {}", e.error_type, e.shader_key, e.message)
        })
    }

    /// Gets the most recent shader key that had an error
    #[bind(name = "GetLatestShaderKey")]
    pub fn get_latest_shader_key() -> Option<String> {
        let state = SHADER_ERRORS.lock().unwrap();
        state.errors.back().map(|e| e.shader_key.clone())
    }
}
