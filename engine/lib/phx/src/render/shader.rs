use std::collections::HashSet;
use std::ffi::CStr;
use std::ffi::CString;

use internal::*;

use super::*;
use crate::common::*;
use crate::logging::warn;
use crate::math::*;
use crate::rf::*;
use crate::system::*;

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
}

struct SetUniformOp {
    index: gl::types::GLint,
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
    name: String,
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
                name: var_name.into(),
                index: -1,
            });
        } else {
            warn!("Failed to parse autovar directive:\n  {val}");
        }
    }
}

impl Shader {
    fn from_preprocessed(name: String, vs_code: GLSLCode, mut fs_code: GLSLCode) -> Shader {
        let vs = create_gl_shader(&vs_code.code, gl::VERTEX_SHADER);
        let fs = create_gl_shader(&fs_code.code, gl::FRAGMENT_SHADER);
        let program = create_gl_program(vs, fs);

        // Combine autovars from all shaders.
        let mut auto_vars = vs_code.auto_vars;
        auto_vars.append(&mut fs_code.auto_vars);

        // Check for autovar conflicts.
        let mut auto_var_keys: HashSet<&str> = HashSet::new();
        for v in auto_vars.iter() {
            if auto_var_keys.contains(v.name.as_str()) {
                warn!("Shader <{}> contains duplicate #autovar <{}>", name, v.name);
                continue;
            };
            auto_var_keys.insert(v.name.as_str());
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
            }),
        };
        shader.bind_auto_variables();
        shader
    }

    pub fn get_uniform_index(&self, name: &str) -> Option<gl::types::GLint> {
        let c_name = CString::new(name).expect("name must be utf-8");
        let index = glcheck!(gl::GetUniformLocation(
            self.shared.as_ref().program,
            c_name.as_ptr()
        ));
        if index >= 0 {
            Some(index)
        } else {
            None
        }
    }

    fn bind_auto_variables(&mut self) {
        let s = &mut *self.shared.as_mut();
        for var in s.auto_vars.iter_mut() {
            let c_name = CString::new(var.name.as_str()).expect("name must be utf-8");
            var.index = glcheck!(gl::GetUniformLocation(s.program, c_name.as_ptr()));
            if var.index < 0 {
                warn!(
                    "Automatic shader variable <{}> does not exist in shader <{}>",
                    var.name, s.name,
                )
            }
        }
    }

    pub fn set_uniform(&mut self, name: &str, data: ShaderVarData) {
        if let Some(index) = self.get_uniform_index(name) {
            self.index_set_uniform(index, data);
        }
    }

    pub fn index_set_uniform(&mut self, index: i32, data: ShaderVarData) {
        self.shared.as_mut().index_set_uniform(index, data);
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

    pub fn apply_uniform(&mut self, index: i32, data: &ShaderVarData) {
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
                glcheck!(gl::BindTexture(gl::TEXTURE_1D, Tex1D_GetHandle(&mut **t)));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
            ShaderVarData::Tex2D(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(gl::TEXTURE_2D, Tex2D_GetHandle(&mut **t)));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
            ShaderVarData::Tex3D(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(gl::TEXTURE_3D, Tex3D_GetHandle(&mut **t)));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
            ShaderVarData::TexCube(t) => {
                let tex_index = self.next_tex_index();

                glcheck!(gl::Uniform1i(index, tex_index as i32));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
                glcheck!(gl::BindTexture(
                    gl::TEXTURE_CUBE_MAP,
                    TexCube_GetHandle(&mut **t)
                ));
                glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            }
        }
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
        )
    }

    pub fn load(vs_name: &str, fs_name: &str) -> Shader {
        Self::from_preprocessed(
            format!("[vs: {} , fs: {}]", vs_name, fs_name),
            GLSLCode::load(vs_name),
            GLSLCode::load(fs_name),
        )
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
        self.set_uniform(name, ShaderVarData::Matrix(*value));
    }

    #[bind(name = "ISetMatrix")]
    pub fn index_set_matrix(&mut self, index: i32, value: &Matrix) {
        self.index_set_uniform(index, ShaderVarData::Matrix(*value));
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
        self.set_uniform(name, ShaderVarData::Tex1D(value as *mut _));
    }

    #[bind(name = "ISetTex1D")]
    pub fn index_set_tex1d(&mut self, index: i32, value: &mut Tex1D) {
        self.index_set_uniform(index, ShaderVarData::Tex1D(value as *mut _));
    }

    pub fn set_tex2d(&mut self, name: &str, value: &mut Tex2D) {
        self.set_uniform(name, ShaderVarData::Tex2D(value as *mut _));
    }

    #[bind(name = "ISetTex2D")]
    pub fn index_set_tex2d(&mut self, index: i32, value: &mut Tex2D) {
        self.index_set_uniform(index, ShaderVarData::Tex2D(value as *mut _));
    }

    pub fn set_tex3d(&mut self, name: &str, value: &mut Tex3D) {
        self.set_uniform(name, ShaderVarData::Tex3D(value as *mut _));
    }

    #[bind(name = "ISetTex3D")]
    pub fn index_set_tex3d(&mut self, index: i32, value: &mut Tex3D) {
        self.index_set_uniform(index, ShaderVarData::Tex3D(value as *mut _));
    }

    pub fn set_tex_cube(&mut self, name: &str, value: &mut TexCube) {
        self.set_uniform(name, ShaderVarData::TexCube(value as *mut _));
    }

    #[bind(name = "ISetTexCube")]
    pub fn index_set_tex_cube(&mut self, index: i32, value: &mut TexCube) {
        self.index_set_uniform(index, ShaderVarData::TexCube(value as *mut _));
    }

    // Singleton based shader functions - Old API.
    pub fn start(&mut self) {
        unsafe {
            Profiler_Begin(c_str!("Shader_Start"));
        }
        
        let s = &mut *self.shared.as_mut();

        glcheck!(gl::UseProgram(s.program));

        // Reset the tex index counter.
        s.tex_index = 0;

        // Apply pending uniforms.
        for p in std::mem::replace(&mut s.pending_uniforms, vec![]) {
            s.apply_uniform(p.index, &p.data);
        }

        // Fetch and bind automatic variables from the shader var stack.
        for i in 0..s.auto_vars.len() {
            if s.auto_vars[i].index == -1 {
                continue;
            }

            let Some(shader_var) = ShaderVar::get(s.auto_vars[i].name.as_str()) else {
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

            s.index_set_uniform(s.auto_vars[i].index, shader_var);
        }

        unsafe {
            Profiler_End();
        }
    }

    pub fn stop(&self) {
        glcheck!(gl::UseProgram(0));
    }
}

fn create_gl_shader(src: &str, shader_type: gl::types::GLenum) -> u32 {
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
    let mut status = 0;
    glcheck!(gl::GetShaderiv(this, gl::COMPILE_STATUS, &mut status));

    if status != gl::TRUE as i32 {
        let mut length = 0;
        glcheck!(gl::GetShaderiv(this, gl::INFO_LOG_LENGTH, &mut length));

        let mut info_log = vec![0; length as usize + 1];
        glcheck!(gl::GetShaderInfoLog(
            this,
            length,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut i8,
        ));

        warn!("Shader:\n{src}");

        panic!(
            "CreateGLShader: Failed to compile shader [{length}]:\n{}",
            String::from_utf8(info_log).unwrap()
        );
    }

    this
}

fn create_gl_program(vs: gl::types::GLuint, fs: gl::types::GLuint) -> gl::types::GLuint {
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
    let mut status: i32 = 0;
    glcheck!(gl::GetProgramiv(this, gl::LINK_STATUS, &mut status));

    if status != gl::TRUE as i32 {
        let mut length: i32 = 0;
        glcheck!(gl::GetProgramiv(this, gl::INFO_LOG_LENGTH, &mut length));

        let mut info_log = vec![0; length as usize + 1];
        glcheck!(gl::GetProgramInfoLog(
            this,
            length,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut i8,
        ));

        panic!(
            "create_gl_program: Failed to link program[{length}]:\n{:?}",
            CStr::from_bytes_with_nul(info_log.as_slice())
        );
    }

    this
}
