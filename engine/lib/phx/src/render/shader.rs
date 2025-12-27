use std::collections::HashSet;
use std::ffi::CString;
use std::sync::Arc;

use glam::{ivec2, ivec3, ivec4, vec2, vec3, vec4};

use super::{ShaderState, ShaderVarData, Tex1D, Tex2D, Tex3D, TexCube, gl};
use super::shader_watcher::{shader_watcher_register, shader_watcher_is_active};
use super::shader_error::push_shader_error;
use crate::common::c_str;
use crate::logging::warn;
use crate::math::Matrix;
use crate::render::{ShaderVar, glcheck, is_command_mode, is_gl_unavailable, submit_command, next_resource_id, RenderCommand, GpuHandle, ResourceId};
use crate::rf::Rf;
use crate::system::{Profiler, Resource, ResourceType};

const INCLUDE_PATH: &str = "include/";

#[derive(Clone)]
pub struct Shader {
    shared: Rf<ShaderShared>,
}

struct ShaderShared {
    name: String,
    vs: gl::types::GLuint,
    fs: gl::types::GLuint,
    program: gl::types::GLuint,
    auto_vars: Vec<ShaderAutoVar>,

    is_bound: bool,
    tex_index: gl::types::GLenum,
    pending_uniforms: Vec<SetUniformOp>,
    pending_uniforms_by_name: Vec<SetUniformByNameOp>,

    // Command mode support: store source for deferred creation on render thread
    vs_src: String,
    fs_src: String,
    resource_id: Option<ResourceId>,

    // Hot-reload support: store source file names for shader watcher registration
    vs_name: Option<String>,
    fs_name: Option<String>,
}

struct SetUniformOp {
    index: gl::types::GLint,
    data: ShaderVarData,
}

struct SetUniformByNameOp {
    name: String,
    data: ShaderVarData,
}

#[derive(Clone, Default)]
struct GLSLCode {
    code: String,
    auto_vars: Vec<ShaderAutoVar>,
}

#[derive(Clone)]
struct ShaderAutoVar {
    type_name: String,
    name: Arc<str>,  // Arc<str> for cheap cloning when setting uniforms each frame
    index: gl::types::GLint,
}

impl Drop for ShaderShared {
    fn drop(&mut self) {
        glcheck!(gl::DeleteShader(self.vs));
        glcheck!(gl::DeleteShader(self.fs));
        glcheck!(gl::DeleteProgram(self.program));
    }
}

impl GLSLCode {
    fn load(name: &str) -> GLSLCode {
        Self::preprocess(&Resource::load_string(ResourceType::Shader, name))
    }

    fn preprocess(code: &str) -> GLSLCode {
        let mut result = GLSLCode::default();

        for line in code.lines() {
            if let Some(include_val) = line.strip_prefix("#include ") {
                let path = format!("{INCLUDE_PATH}{include_val}");
                let mut include = Self::load(&path);

                result.code += &include.code;
                result.code += "\n";

                result.auto_vars.append(&mut include.auto_vars);
            } else if let Some(autovar_val) = line.strip_prefix("#autovar ") {
                Self::parse_autovar(autovar_val, &mut result.auto_vars);
            } else {
                result.code += line;
                result.code += "\n";
            }
        }

        result
    }

    fn parse_autovar(val: &str, auto_vars: &mut Vec<ShaderAutoVar>) {
        let line_tokens: Vec<_> = val.split(' ').collect();
        if line_tokens.len() == 2 {
            let var_type = line_tokens[0];
            let var_name = line_tokens[1];
            auto_vars.push(ShaderAutoVar {
                type_name: var_type.into(),
                name: Arc::from(var_name),
                index: -1,
            });
        } else {
            warn!("Failed to parse autovar directive:\n  {val}");
        }
    }
}

impl Shader {
    /// Try to compile a shader from preprocessed code, returning None on failure.
    /// Errors are pushed to the global shader error queue for display.
    fn try_from_preprocessed(
        name: String,
        vs_code: GLSLCode,
        mut fs_code: GLSLCode,
        vs_name: Option<String>,
        fs_name: Option<String>,
    ) -> Option<Shader> {
        // Compile vertex shader
        let vs = match create_gl_shader(&vs_code.code, gl::VERTEX_SHADER) {
            Ok(vs) => vs,
            Err(error_msg) => {
                let shader_key = format!("{}:{}",
                    vs_name.as_deref().unwrap_or("unknown"),
                    fs_name.as_deref().unwrap_or("unknown")
                );
                push_shader_error(&shader_key, "vertex compile", &error_msg);
                return None;
            }
        };

        // Compile fragment shader
        let fs = match create_gl_shader(&fs_code.code, gl::FRAGMENT_SHADER) {
            Ok(fs) => fs,
            Err(error_msg) => {
                // Clean up vertex shader since fragment failed
                glcheck!(gl::DeleteShader(vs));
                let shader_key = format!("{}:{}",
                    vs_name.as_deref().unwrap_or("unknown"),
                    fs_name.as_deref().unwrap_or("unknown")
                );
                push_shader_error(&shader_key, "fragment compile", &error_msg);
                return None;
            }
        };

        // Link program
        let program = match create_gl_program(vs, fs) {
            Ok(program) => program,
            Err(error_msg) => {
                // Clean up shaders since linking failed
                glcheck!(gl::DeleteShader(vs));
                glcheck!(gl::DeleteShader(fs));
                let shader_key = format!("{}:{}",
                    vs_name.as_deref().unwrap_or("unknown"),
                    fs_name.as_deref().unwrap_or("unknown")
                );
                push_shader_error(&shader_key, "link", &error_msg);
                return None;
            }
        };

        // Store source code for command mode (deferred creation on render thread)
        let vs_src = vs_code.code.clone();
        let fs_src = fs_code.code.clone();

        // Combine autovars from all shaders.
        let mut auto_vars = vs_code.auto_vars;
        auto_vars.append(&mut fs_code.auto_vars);

        // Check for autovar conflicts.
        let mut auto_var_keys: HashSet<&str> = HashSet::new();
        for v in auto_vars.iter() {
            if auto_var_keys.contains(&*v.name) {
                warn!("Shader <{}> contains duplicate #autovar <{}>", name, v.name);
                continue;
            };
            auto_var_keys.insert(&v.name);
        }

        let mut shader = Shader {
            shared: Rf::new(ShaderShared {
                name,
                vs,
                fs,
                program,
                auto_vars,
                tex_index: 0,
                is_bound: false,
                pending_uniforms: vec![],
                pending_uniforms_by_name: vec![],
                vs_src,
                fs_src,
                resource_id: None,
                vs_name,
                fs_name,
            }),
        };
        shader.bind_auto_variables();
        Some(shader)
    }

    /// Compile a shader, panicking on failure (legacy behavior for non-hot-reload paths)
    fn from_preprocessed(
        name: String,
        vs_code: GLSLCode,
        fs_code: GLSLCode,
        vs_name: Option<String>,
        fs_name: Option<String>,
    ) -> Shader {
        Self::try_from_preprocessed(name.clone(), vs_code, fs_code, vs_name, fs_name)
            .unwrap_or_else(|| panic!("Failed to compile shader: {}", name))
    }

    /// Registers this shader with the watcher for hot-reload support.
    /// Called automatically on load, but can be called manually for dynamically created shaders.
    pub fn register_for_hot_reload(&self) {
        if !shader_watcher_is_active() {
            return;
        }

        let s = self.shared.as_ref();
        if let (Some(vs_name), Some(fs_name)) = (&s.vs_name, &s.fs_name) {
            // Construct shader key matching Lua Cache.Shader key format
            let shader_key = format!("{}:{}", vs_name, fs_name);
            let vs_path = Resource::get_path(ResourceType::Shader, vs_name);
            let fs_path = Resource::get_path(ResourceType::Shader, fs_name);
            shader_watcher_register(&shader_key, &vs_path, &fs_path);
        }
    }

    pub fn get_uniform_index(&self, name: &str) -> Option<gl::types::GLint> {
        let c_name = CString::new(name).expect("name must be utf-8");
        let index = glcheck!(gl::GetUniformLocation(
            self.shared.as_ref().program,
            c_name.as_ptr()
        ));
        if index >= 0 { Some(index) } else { None }
    }

    fn bind_auto_variables(&mut self) {
        let s = &mut *self.shared.as_mut();
        for var in s.auto_vars.iter_mut() {
            let c_name = CString::new(&*var.name).expect("name must be utf-8");
            var.index = glcheck!(gl::GetUniformLocation(s.program, c_name.as_ptr()));
            if var.index < 0 {
                warn!(
                    "Automatic shader variable <{}> does not exist in shader {}",
                    var.name, s.name,
                )
            }
        }
    }

    pub fn set_uniform(&mut self, name: &str, data: ShaderVarData) {
        // In command mode, use name-based uniforms since the render thread's
        // shader has different uniform indices
        if is_command_mode() {
            self.shared.as_mut().name_set_uniform(name, data);
        } else if let Some(index) = self.get_uniform_index(name) {
            self.index_set_uniform(index, data);
        }
    }

    pub fn index_set_uniform(&mut self, index: i32, data: ShaderVarData) {
        self.shared.as_mut().index_set_uniform(index, data);
    }

    /// Apply uniform by name directly (for use when shader is already bound).
    /// Used by ShaderState in command mode to avoid is_bound check.
    pub fn name_set_uniform_on_shared(&mut self, name: &str, data: ShaderVarData) {
        self.shared.as_mut().apply_uniform_by_name(name, &data);
    }
}

impl ShaderShared {
    // Increments the current texture index and returns the next free one.
    fn next_tex_index(&mut self) -> gl::types::GLenum {
        self.tex_index += 1;
        self.tex_index
    }

    pub fn index_set_uniform(&mut self, index: i32, data: ShaderVarData) {
        if self.is_bound {
            self.apply_uniform(index, &data);
        } else {
            self.pending_uniforms.push(SetUniformOp { index, data });
        }
    }

    /// Set uniform by name - used in command mode where uniform indices differ
    pub fn name_set_uniform(&mut self, name: &str, data: ShaderVarData) {
        if self.is_bound {
            self.apply_uniform_by_name(name, &data);
        } else {
            // Queue for later application when shader is bound
            self.pending_uniforms_by_name.push(SetUniformByNameOp {
                name: name.to_string(),
                data,
            });
        }
    }

    pub fn apply_uniform(&mut self, index: i32, data: &ShaderVarData) {
        if is_command_mode() {
            self.apply_uniform_command(index, data);
        } else {
            self.apply_uniform_direct(index, data);
        }
    }

    /// Apply uniform by name - used for auto vars in command mode where
    /// the render thread has different uniform indices than the main thread
    pub fn apply_uniform_by_name(&mut self, name: &str, data: &ShaderVarData) {
        if is_command_mode() {
            self.apply_uniform_command_by_name(name, data);
        } else {
            // In direct mode, look up index and apply
            if let Some(index) = self.get_uniform_index_inner(name) {
                self.apply_uniform_direct(index, data);
            }
        }
    }

    /// Apply uniform by name with Arc<str> - O(1) cloning for uniform names
    pub fn apply_uniform_by_name_arc(&mut self, name: Arc<str>, data: &ShaderVarData) {
        if is_command_mode() {
            self.apply_uniform_command_by_name_arc(name, data);
        } else {
            // In direct mode, look up index and apply
            if let Some(index) = self.get_uniform_index_inner(&name) {
                self.apply_uniform_direct(index, data);
            }
        }
    }

    fn get_uniform_index_inner(&self, name: &str) -> Option<i32> {
        let c_name = CString::new(name).ok()?;
        let index = glcheck!(gl::GetUniformLocation(self.program, c_name.as_ptr()));
        if index >= 0 { Some(index) } else { None }
    }

    fn apply_uniform_command_by_name(&mut self, name: &str, data: &ShaderVarData) {
        // Create Arc<str> once, then clone (O(1)) for each command
        let name: Arc<str> = Arc::from(name);
        match data {
            ShaderVarData::Float(v) => {
                submit_command(RenderCommand::SetUniformFloatByName { name, value: *v });
            }
            ShaderVarData::Float2(v) => {
                submit_command(RenderCommand::SetUniformFloat2ByName { name, value: [v.x, v.y] });
            }
            ShaderVarData::Float3(v) => {
                submit_command(RenderCommand::SetUniformFloat3ByName { name, value: [v.x, v.y, v.z] });
            }
            ShaderVarData::Float4(v) => {
                submit_command(RenderCommand::SetUniformFloat4ByName { name, value: [v.x, v.y, v.z, v.w] });
            }
            ShaderVarData::Int(v) => {
                submit_command(RenderCommand::SetUniformIntByName { name, value: *v });
            }
            ShaderVarData::Int2(v) => {
                submit_command(RenderCommand::SetUniformInt2ByName { name, value: [v.x, v.y] });
            }
            ShaderVarData::Int3(v) => {
                submit_command(RenderCommand::SetUniformInt3ByName { name, value: [v.x, v.y, v.z] });
            }
            ShaderVarData::Int4(v) => {
                submit_command(RenderCommand::SetUniformInt4ByName { name, value: [v.x, v.y, v.z, v.w] });
            }
            ShaderVarData::Matrix(m) => {
                let value = m.to_cols_array();
                submit_command(RenderCommand::SetUniformMat4ByName { name, value });
            }
            ShaderVarData::Tex1D(_t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
            }
            ShaderVarData::Tex2D(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
                // First check if already has resource_id
                if let Some(resource_id) = t.get_resource_id() {
                    submit_command(RenderCommand::BindTexture2DByResource { slot: tex_index, id: resource_id });
                } else {
                    // Texture was created before command mode - lazily migrate to render thread
                    // Clone shares the same underlying data via Rf, so mutations persist
                    let mut tex = t.clone();
                    if let Some(resource_id) = tex.ensure_resource_id() {
                        submit_command(RenderCommand::BindTexture2DByResource { slot: tex_index, id: resource_id });
                    } else {
                        // No cached data - can't migrate, will likely render incorrectly
                        warn!("Tex2D bound in command mode without resource_id or cached data");
                        submit_command(RenderCommand::BindTexture2D { slot: tex_index, handle: GpuHandle(t.get_handle()) });
                    }
                }
            }
            ShaderVarData::Tex3D(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
                submit_command(RenderCommand::BindTexture3D { slot: tex_index, handle: GpuHandle(t.get_handle()) });
            }
            ShaderVarData::TexCube(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
                submit_command(RenderCommand::BindTextureCube { slot: tex_index, handle: GpuHandle(t.get_handle()) });
            }
        }
    }

    /// Apply uniform command with Arc<str> - O(1) cloning for uniform names
    fn apply_uniform_command_by_name_arc(&mut self, name: Arc<str>, data: &ShaderVarData) {
        match data {
            ShaderVarData::Float(v) => {
                submit_command(RenderCommand::SetUniformFloatByName { name, value: *v });
            }
            ShaderVarData::Float2(v) => {
                submit_command(RenderCommand::SetUniformFloat2ByName { name, value: [v.x, v.y] });
            }
            ShaderVarData::Float3(v) => {
                submit_command(RenderCommand::SetUniformFloat3ByName { name, value: [v.x, v.y, v.z] });
            }
            ShaderVarData::Float4(v) => {
                submit_command(RenderCommand::SetUniformFloat4ByName { name, value: [v.x, v.y, v.z, v.w] });
            }
            ShaderVarData::Int(v) => {
                submit_command(RenderCommand::SetUniformIntByName { name, value: *v });
            }
            ShaderVarData::Int2(v) => {
                submit_command(RenderCommand::SetUniformInt2ByName { name, value: [v.x, v.y] });
            }
            ShaderVarData::Int3(v) => {
                submit_command(RenderCommand::SetUniformInt3ByName { name, value: [v.x, v.y, v.z] });
            }
            ShaderVarData::Int4(v) => {
                submit_command(RenderCommand::SetUniformInt4ByName { name, value: [v.x, v.y, v.z, v.w] });
            }
            ShaderVarData::Matrix(m) => {
                let value = m.to_cols_array();
                submit_command(RenderCommand::SetUniformMat4ByName { name, value });
            }
            ShaderVarData::Tex1D(_t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
            }
            ShaderVarData::Tex2D(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
                if let Some(resource_id) = t.get_resource_id() {
                    submit_command(RenderCommand::BindTexture2DByResource { slot: tex_index, id: resource_id });
                } else {
                    let mut tex = t.clone();
                    if let Some(resource_id) = tex.ensure_resource_id() {
                        submit_command(RenderCommand::BindTexture2DByResource { slot: tex_index, id: resource_id });
                    } else {
                        warn!("Tex2D bound in command mode without resource_id or cached data");
                        submit_command(RenderCommand::BindTexture2D { slot: tex_index, handle: GpuHandle(t.get_handle()) });
                    }
                }
            }
            ShaderVarData::Tex3D(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
                submit_command(RenderCommand::BindTexture3D { slot: tex_index, handle: GpuHandle(t.get_handle()) });
            }
            ShaderVarData::TexCube(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformIntByName { name, value: tex_index as i32 });
                submit_command(RenderCommand::BindTextureCube { slot: tex_index, handle: GpuHandle(t.get_handle()) });
            }
        }
    }

    fn apply_uniform_command(&mut self, index: i32, data: &ShaderVarData) {
        match data {
            ShaderVarData::Float(v) => {
                submit_command(RenderCommand::SetUniformFloat { location: index, value: *v });
            }
            ShaderVarData::Float2(v) => {
                submit_command(RenderCommand::SetUniformFloat2 { location: index, value: [v.x, v.y] });
            }
            ShaderVarData::Float3(v) => {
                submit_command(RenderCommand::SetUniformFloat3 { location: index, value: [v.x, v.y, v.z] });
            }
            ShaderVarData::Float4(v) => {
                submit_command(RenderCommand::SetUniformFloat4 { location: index, value: [v.x, v.y, v.z, v.w] });
            }
            ShaderVarData::Int(v) => {
                submit_command(RenderCommand::SetUniformInt { location: index, value: *v });
            }
            ShaderVarData::Int2(v) => {
                submit_command(RenderCommand::SetUniformInt2 { location: index, value: [v.x, v.y] });
            }
            ShaderVarData::Int3(v) => {
                submit_command(RenderCommand::SetUniformInt3 { location: index, value: [v.x, v.y, v.z] });
            }
            ShaderVarData::Int4(v) => {
                submit_command(RenderCommand::SetUniformInt4 { location: index, value: [v.x, v.y, v.z, v.w] });
            }
            ShaderVarData::Matrix(m) => {
                let value = m.to_cols_array();
                submit_command(RenderCommand::SetUniformMat4 { location: index, value });
            }
            ShaderVarData::Tex1D(_t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformInt { location: index, value: tex_index as i32 });
                // Note: Tex1D binding not implemented in commands yet - would need new command
            }
            ShaderVarData::Tex2D(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformInt { location: index, value: tex_index as i32 });
                // First check if already has resource_id
                if let Some(resource_id) = t.get_resource_id() {
                    submit_command(RenderCommand::BindTexture2DByResource { slot: tex_index, id: resource_id });
                } else {
                    // Texture was created before command mode - lazily migrate to render thread
                    let mut tex = t.clone();
                    if let Some(resource_id) = tex.ensure_resource_id() {
                        submit_command(RenderCommand::BindTexture2DByResource { slot: tex_index, id: resource_id });
                    } else {
                        warn!("Tex2D bound in command mode without resource_id or cached data");
                        submit_command(RenderCommand::BindTexture2D { slot: tex_index, handle: GpuHandle(t.get_handle()) });
                    }
                }
            }
            ShaderVarData::Tex3D(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformInt { location: index, value: tex_index as i32 });
                submit_command(RenderCommand::BindTexture3D { slot: tex_index, handle: GpuHandle(t.get_handle()) });
            }
            ShaderVarData::TexCube(t) => {
                let tex_index = self.next_tex_index();
                submit_command(RenderCommand::SetUniformInt { location: index, value: tex_index as i32 });
                submit_command(RenderCommand::BindTextureCube { slot: tex_index, handle: GpuHandle(t.get_handle()) });
            }
        }
    }

    fn apply_uniform_direct(&mut self, index: i32, data: &ShaderVarData) {
        match data {
            ShaderVarData::Float(v) => glcheck!(gl::Uniform1f(index, *v)),
            ShaderVarData::Float2(v) => glcheck!(gl::Uniform2f(index, v.x, v.y)),
            ShaderVarData::Float3(v) => glcheck!(gl::Uniform3f(index, v.x, v.y, v.z)),
            ShaderVarData::Float4(v) => glcheck!(gl::Uniform4f(index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Int(v) => glcheck!(gl::Uniform1i(index, *v)),
            ShaderVarData::Int2(v) => glcheck!(gl::Uniform2i(index, v.x, v.y)),
            ShaderVarData::Int3(v) => glcheck!(gl::Uniform3i(index, v.x, v.y, v.z)),
            ShaderVarData::Int4(v) => glcheck!(gl::Uniform4i(index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Matrix(m) => {
                glcheck!(gl::UniformMatrix4fv(
                    index,
                    1,
                    gl::FALSE,
                    m as *const Matrix as *const f32
                ));
            }
            ShaderVarData::Tex1D(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(gl::TEXTURE_1D, t.get_handle()));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
            ShaderVarData::Tex2D(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(gl::TEXTURE_2D, t.get_handle()));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
            ShaderVarData::Tex3D(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(gl::TEXTURE_3D, t.get_handle()));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
            ShaderVarData::TexCube(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, t.get_handle(),));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
        }
    }
}

// Non-FFI methods for internal Rust use
impl Shader {
    /// Get preprocessed shader source code (for render thread compilation).
    /// Returns (vertex_src, fragment_src) tuple.
    pub fn get_preprocessed_source(vs_name: &str, fs_name: &str) -> (String, String) {
        let vs_code = GLSLCode::load(vs_name);
        let fs_code = GLSLCode::load(fs_name);
        (vs_code.code, fs_code.code)
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Shader {
    #[bind(name = "Create")]
    pub fn new(vs: &str, fs: &str) -> Shader {
        Self::from_preprocessed(
            "[anonymous shader]".into(),
            GLSLCode::preprocess(vs),
            GLSLCode::preprocess(fs),
            None,
            None,
        )
    }

    pub fn load(vs_name: &str, fs_name: &str) -> Shader {
        // Try to load the shader, falling back to error shader on failure
        Self::try_load(vs_name, fs_name).unwrap_or_else(|| {
            warn!("Shader compilation failed for vs={} fs={}, trying error shader", vs_name, fs_name);
            Self::create_error_shader(vs_name, fs_name)
                .expect("Failed to create error shader - GL context may not be available")
        })
        // Note: Hot-reload registration is done by Lua Cache.Shader() using its key format
    }

    /// Creates a minimal error shader that renders magenta to indicate shader failure
    fn create_error_shader(vs_name: &str, fs_name: &str) -> Option<Shader> {
        let error_vs = r#"#version 330
in vec3 vertex_position;
uniform mat4 mWorldViewProj;
void main() {
    gl_Position = mWorldViewProj * vec4(vertex_position, 1.0);
}
"#;
        let error_fs = r#"#version 330
out vec4 fragColor;
void main() {
    fragColor = vec4(1.0, 0.0, 1.0, 1.0); // Magenta = error
}
"#;
        // Create minimal GLSLCode with just the error shader code
        let vs_code = GLSLCode {
            code: error_vs.to_string(),
            auto_vars: vec![],
        };
        let fs_code = GLSLCode {
            code: error_fs.to_string(),
            auto_vars: vec![],
        };

        Self::try_from_preprocessed(
            format!("[ERROR: vs: {vs_name}, fs: {fs_name}]"),
            vs_code,
            fs_code,
            Some(vs_name.to_string()),
            Some(fs_name.to_string()),
        )
    }

    /// Try to load a shader, returning None on failure.
    /// Errors are pushed to the global shader error queue for display.
    /// Use this for hot-reload to gracefully fall back to the previous working shader.
    #[bind(name = "TryLoad")]
    pub fn try_load(vs_name: &str, fs_name: &str) -> Option<Shader> {
        Self::try_from_preprocessed(
            format!("[vs: {vs_name}, fs: {fs_name}]"),
            GLSLCode::load(vs_name),
            GLSLCode::load(fs_name),
            Some(vs_name.to_string()),
            Some(fs_name.to_string()),
        )
    }

    pub fn name(&self) -> String {
        self.shared.as_ref().name.clone()
    }

    #[bind(name = "Clone")]
    pub fn acquire(&self) -> Shader {
        self.clone()
    }

    pub fn to_shader_state(&self) -> ShaderState {
        ShaderState::new(self)
    }

    #[bind(name = "GetHandle")]
    pub fn handle(&self) -> u32 {
        self.shared.as_ref().program
    }

    #[bind(name = "GetVariable")]
    pub fn get_uniform_index_unchecked(&self, name: &str) -> i32 {
        self.get_uniform_index(name).unwrap_or_else(|| {
            panic!(
                "Shader <{}> has no variable <{}>",
                self.shared.as_ref().name,
                name,
            );
        })
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.get_uniform_index(name).is_some()
    }

    pub fn reset_tex_index(&mut self) {
        self.shared.as_mut().tex_index = 0;
    }

    pub fn set_float(&mut self, name: &str, value: f32) {
        self.set_uniform(name, ShaderVarData::Float(value));
    }

    #[bind(name = "ISetFloat")]
    pub fn index_set_float(&mut self, index: i32, value: f32) {
        self.index_set_uniform(index, ShaderVarData::Float(value));
    }

    pub fn set_float2(&mut self, name: &str, x: f32, y: f32) {
        self.set_uniform(name, ShaderVarData::Float2(vec2(x, y)));
    }

    #[bind(name = "ISetFloat2")]
    pub fn index_set_float2(&mut self, index: i32, x: f32, y: f32) {
        self.index_set_uniform(index, ShaderVarData::Float2(vec2(x, y)));
    }

    pub fn set_float3(&mut self, name: &str, x: f32, y: f32, z: f32) {
        self.set_uniform(name, ShaderVarData::Float3(vec3(x, y, z)));
    }

    #[bind(name = "ISetFloat3")]
    pub fn index_set_float3(&mut self, index: i32, x: f32, y: f32, z: f32) {
        self.index_set_uniform(index, ShaderVarData::Float3(vec3(x, y, z)));
    }

    pub fn set_float4(&mut self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        self.set_uniform(name, ShaderVarData::Float4(vec4(x, y, z, w)));
    }

    #[bind(name = "ISetFloat4")]
    pub fn index_set_float4(&mut self, index: i32, x: f32, y: f32, z: f32, w: f32) {
        self.index_set_uniform(index, ShaderVarData::Float4(vec4(x, y, z, w)));
    }

    pub fn set_int(&mut self, name: &str, value: i32) {
        self.set_uniform(name, ShaderVarData::Int(value));
    }

    #[bind(name = "ISetInt")]
    pub fn index_set_int(&mut self, index: i32, value: i32) {
        self.index_set_uniform(index, ShaderVarData::Int(value));
    }

    pub fn set_int2(&mut self, name: &str, x: i32, y: i32) {
        self.set_uniform(name, ShaderVarData::Int2(ivec2(x, y)));
    }

    #[bind(name = "ISetInt2")]
    pub fn index_set_int2(&mut self, index: i32, x: i32, y: i32) {
        self.index_set_uniform(index, ShaderVarData::Int2(ivec2(x, y)));
    }

    pub fn set_int3(&mut self, name: &str, x: i32, y: i32, z: i32) {
        self.set_uniform(name, ShaderVarData::Int3(ivec3(x, y, z)));
    }

    #[bind(name = "ISetInt3")]
    pub fn index_set_int3(&mut self, index: i32, x: i32, y: i32, z: i32) {
        self.index_set_uniform(index, ShaderVarData::Int3(ivec3(x, y, z)));
    }

    pub fn set_int4(&mut self, name: &str, x: i32, y: i32, z: i32, w: i32) {
        self.set_uniform(name, ShaderVarData::Int4(ivec4(x, y, z, w)));
    }

    #[bind(name = "ISetInt4")]
    pub fn index_set_int4(&mut self, index: i32, x: i32, y: i32, z: i32, w: i32) {
        self.index_set_uniform(index, ShaderVarData::Int4(ivec4(x, y, z, w)));
    }

    pub fn set_matrix(&mut self, name: &str, value: &Matrix) {
        self.set_uniform(name, ShaderVarData::Matrix(value.clone()));
    }

    #[bind(name = "ISetMatrix")]
    pub fn index_set_matrix(&mut self, index: i32, value: &Matrix) {
        self.index_set_uniform(index, ShaderVarData::Matrix(value.clone()));
    }

    #[bind(name = "SetMatrixT")]
    pub fn set_matrix_transpose(&mut self, name: &str, value: &Matrix) {
        self.set_uniform(name, ShaderVarData::Matrix(value.transpose()));
    }

    #[bind(name = "ISetMatrixT")]
    pub fn index_set_matrix_transpose(&mut self, index: i32, value: &Matrix) {
        self.index_set_uniform(index, ShaderVarData::Matrix(value.transpose()));
    }

    pub fn set_tex1d(&mut self, name: &str, value: &mut Tex1D) {
        self.set_uniform(name, ShaderVarData::Tex1D(value.clone()));
    }

    #[bind(name = "ISetTex1D")]
    pub fn index_set_tex1d(&mut self, index: i32, value: &mut Tex1D) {
        self.index_set_uniform(index, ShaderVarData::Tex1D(value.clone()));
    }

    pub fn set_tex2d(&mut self, name: &str, value: &Tex2D) {
        self.set_uniform(name, ShaderVarData::Tex2D(value.clone()));
    }

    #[bind(name = "ISetTex2D")]
    pub fn index_set_tex2d(&mut self, index: i32, value: &mut Tex2D) {
        self.index_set_uniform(index, ShaderVarData::Tex2D(value.clone()));
    }

    pub fn set_tex3d(&mut self, name: &str, value: &mut Tex3D) {
        self.set_uniform(name, ShaderVarData::Tex3D(value.clone()));
    }

    #[bind(name = "ISetTex3D")]
    pub fn index_set_tex3d(&mut self, index: i32, value: &mut Tex3D) {
        self.index_set_uniform(index, ShaderVarData::Tex3D(value.clone()));
    }

    pub fn set_tex_cube(&mut self, name: &str, value: &mut TexCube) {
        self.set_uniform(name, ShaderVarData::TexCube(value.clone()));
    }

    #[bind(name = "ISetTexCube")]
    pub fn index_set_tex_cube(&mut self, index: i32, value: &mut TexCube) {
        self.index_set_uniform(index, ShaderVarData::TexCube(value.clone()));
    }

    // Singleton based shader functions - Old API.
    pub fn start(&mut self) {
        // Skip if GL context is unavailable
        if is_gl_unavailable() {
            return;
        }

        Profiler::begin("Shader_Start");

        let s = &mut *self.shared.as_mut();

        // Bind shader program
        if is_command_mode() {
            // In command mode, we need to create the shader on the render thread if not done yet
            if s.resource_id.is_none() {
                let id = next_resource_id();
                tracing::trace!("Creating shader '{}' on render thread with ResourceId {:?}", s.name, id);
                let submitted = submit_command(RenderCommand::CreateShader {
                    id,
                    vertex_src: s.vs_src.clone(),
                    fragment_src: s.fs_src.clone(),
                });
                if !submitted {
                    tracing::error!("Failed to submit CreateShader command for '{}'!", s.name);
                }
                s.resource_id = Some(id);
            }

            // Now bind using the resource ID
            // Include shader_key for hot-reload lookup (matches Cache.lua key format: vs:fs)
            // Cache.lua key is short names without "vertex/" and "fragment/" prefixes
            let id = s.resource_id.unwrap();
            let shader_key = match (&s.vs_name, &s.fs_name) {
                (Some(vs), Some(fs)) => {
                    // Strip "vertex/" and "fragment/" prefixes to match Cache.lua format
                    let vs_short = vs.strip_prefix("vertex/").unwrap_or(vs);
                    let fs_short = fs.strip_prefix("fragment/").unwrap_or(fs);
                    Some(format!("{}:{}", vs_short, fs_short))
                },
                _ => None,
            };
            tracing::trace!("Binding shader '{}' with ResourceId {:?}, shader_key={:?}", s.name, id, shader_key);
            let submitted = submit_command(RenderCommand::BindShaderByResource { id, shader_key });
            if !submitted {
                tracing::error!("Failed to submit BindShaderByResource for '{}'!", s.name);
            }
        } else {
            glcheck!(gl::UseProgram(s.program));
        }
        s.is_bound = true;

        // Reset the tex index counter.
        s.tex_index = 0;

        // Apply pending uniforms (index-based).
        for p in std::mem::take(&mut s.pending_uniforms) {
            s.apply_uniform(p.index, &p.data);
        }

        // Apply pending uniforms (name-based, used in command mode).
        for p in std::mem::take(&mut s.pending_uniforms_by_name) {
            s.apply_uniform_by_name(&p.name, &p.data);
        }

        // Fetch and bind automatic variables from the shader var stack.
        // In command mode, use name-based uniforms since the render thread's
        // shader has different uniform indices than the main thread's shader.
        for i in 0..s.auto_vars.len() {
            // In command mode, we skip the index check since we'll look up by name
            if !is_command_mode() && s.auto_vars[i].index == -1 {
                continue;
            }

            let Some(shader_var) = ShaderVar::get(&s.auto_vars[i].name) else {
                warn!(
                    "Shader variable stack does not contain variable <{}>",
                    s.auto_vars[i].name,
                );
                continue;
            };

            if shader_var.get_glsl_type() != s.auto_vars[i].type_name {
                warn!(
                    "Attempting to get stack of type <{}> for shader variable <{}> when existing stack has type <{}>",
                    s.auto_vars[i].type_name,
                    s.auto_vars[i].name,
                    shader_var.get_glsl_type(),
                );
                continue;
            }

            // Use name-based uniforms in command mode to handle different shader program indices
            if is_command_mode() {
                // Arc<str>::clone() is O(1) - just increments refcount
                s.apply_uniform_by_name_arc(s.auto_vars[i].name.clone(), &shader_var);
            } else {
                s.index_set_uniform(s.auto_vars[i].index, shader_var);
            }
        }

        Profiler::end();
    }

    /// Invalidate the shader's render thread resource, forcing recreation on next start().
    /// Used after hot-reload to ensure fresh uniform bindings.
    #[bind(name = "Invalidate")]
    pub fn invalidate(&mut self) {
        let s = &mut *self.shared.as_mut();
        s.resource_id = None;
        tracing::debug!("Shader '{}' invalidated, will recreate on next start()", s.name);
    }

    pub fn stop(&self) {
        self.shared.as_mut().is_bound = false;
        if is_command_mode() {
            submit_command(RenderCommand::UnbindShader);
        } else {
            glcheck!(gl::UseProgram(0));
        }
    }
}

/// Result of shader compilation
type ShaderResult<T> = Result<T, String>;

fn create_gl_shader(src: &str, shader_type: gl::types::GLenum) -> ShaderResult<u32> {
    let this = glcheck!(gl::CreateShader(shader_type));

    let src_cstr = CString::new(src).expect("Shader source must be utf-8");
    glcheck!(gl::ShaderSource(
        this,
        1,
        &src_cstr.as_ptr(),
        std::ptr::null(),
    ));
    glcheck!(gl::CompileShader(this));

    // Check for compile errors.
    let mut status: gl::types::GLint = 0;
    glcheck!(gl::GetShaderiv(this, gl::COMPILE_STATUS, &mut status));

    if status == 0 {
        let mut length = 0;
        glcheck!(gl::GetShaderiv(this, gl::INFO_LOG_LENGTH, &mut length));

        let mut info_log = vec![0; length as usize + 1];
        glcheck!(gl::GetShaderInfoLog(
            this,
            length,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut i8,
        ));

        // Clean up the failed shader
        glcheck!(gl::DeleteShader(this));

        // Convert to string, stripping null bytes (CString can't have interior nulls)
        let error_msg = String::from_utf8_lossy(&info_log)
            .replace('\0', "")
            .trim()
            .to_string();
        return Err(if error_msg.is_empty() { "Unknown compile error".to_string() } else { error_msg });
    }

    Ok(this)
}

fn create_gl_program(vs: gl::types::GLuint, fs: gl::types::GLuint) -> ShaderResult<gl::types::GLuint> {
    let this = glcheck!(gl::CreateProgram());
    glcheck!(gl::AttachShader(this, vs));
    glcheck!(gl::AttachShader(this, fs));

    /* TODO : Replace with custom directives. */
    glcheck!(gl::BindAttribLocation(this, 0, c_str!("vertex_position")));
    glcheck!(gl::BindAttribLocation(this, 1, c_str!("vertex_normal")));
    glcheck!(gl::BindAttribLocation(this, 2, c_str!("vertex_uv")));
    glcheck!(gl::BindAttribLocation(this, 3, c_str!("vertex_color")));

    glcheck!(gl::LinkProgram(this));

    // Check for link errors.
    let mut status: gl::types::GLint = 0;
    glcheck!(gl::GetProgramiv(this, gl::LINK_STATUS, &mut status));

    if status == 0 {
        let mut length: i32 = 0;
        glcheck!(gl::GetProgramiv(this, gl::INFO_LOG_LENGTH, &mut length));

        let mut info_log = vec![0; length as usize + 1];
        glcheck!(gl::GetProgramInfoLog(
            this,
            length,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut i8,
        ));

        // Clean up the failed program
        glcheck!(gl::DeleteProgram(this));

        // Convert to string, stripping null bytes (CString can't have interior nulls)
        let error_msg = String::from_utf8_lossy(&info_log)
            .replace('\0', "")
            .trim()
            .to_string();
        return Err(if error_msg.is_empty() { "Unknown link error".to_string() } else { error_msg });
    }

    // Bind CameraUBO to binding point 0 (if the uniform block exists in this shader)
    let block_index = glcheck!(gl::GetUniformBlockIndex(this, c_str!("CameraUBO")));
    if block_index != gl::INVALID_INDEX {
        glcheck!(gl::UniformBlockBinding(this, block_index, 0));
    }

    // Bind LightUBO to binding point 2 (if the uniform block exists in this shader)
    let light_block_index = glcheck!(gl::GetUniformBlockIndex(this, c_str!("LightUBO")));
    if light_block_index != gl::INVALID_INDEX {
        glcheck!(gl::UniformBlockBinding(this, light_block_index, 2));
    }

    Ok(this)
}
