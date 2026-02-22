/// Result of a shader reload operation
#[derive(Debug, Clone)]
pub struct ShaderReloadResult {
    /// The shader key that was reloaded
    pub shader_key: String,
    /// Whether compilation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// The new program handle (if success)
    pub program: u32,
}
