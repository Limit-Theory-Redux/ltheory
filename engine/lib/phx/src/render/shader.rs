use std::cell::RefCell;
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

static mut CURRENT: *mut Shader = std::ptr::null_mut();

#[derive(Clone)]
pub struct Shader {
    shared: Rf<ShaderShared>,
}

struct ShaderShared {
    name: String,
    vs: gl::types::GLuint,
    fs: gl::types::GLuint,
    program: gl::types::GLuint,
    tex_index: RefCell<gl::types::GLenum>,
    auto_vars: Vec<ShaderAutoVar>,
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
        let line_tokens: Vec<_> = val.split(" ").collect();
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
                tex_index: RefCell::new(0),
                auto_vars,
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

    // Increments the current texture index and returns the next free one.
    fn next_tex_index(&mut self) -> gl::types::GLenum {
        let s = self.shared.as_ref();
        let tex_index = &mut *s.tex_index.borrow_mut();
        *tex_index += 1;
        *tex_index
    }

    fn set_current(current: Option<&Shader>) {
        unsafe {
            if !CURRENT.is_null() {
                // Free the existing current shader if it exists by converting it back into the box immediately dropping it.
                let _ = Box::from_raw(CURRENT);
            }
            CURRENT = current.map_or(std::ptr::null_mut(), |r| Box::into_raw(Box::new(r.clone())))
        }
    }

    fn get_current() -> Option<&'static mut Shader> {
        unsafe { CURRENT.as_mut() }
    }

    fn get_current_checked() -> &'static mut Shader {
        Self::get_current().expect("No shader is bound")
    }

    pub fn set_uniform(name: &str, data: &ShaderVarData) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_uniform(index, data);
        }
    }

    pub fn index_set_uniform(index: i32, data: &ShaderVarData) {
        match *data {
            ShaderVarData::Float(v) => glcheck!(gl::Uniform1f(index, v)),
            ShaderVarData::Float2(v) => glcheck!(gl::Uniform2f(index, v.x, v.y)),
            ShaderVarData::Float3(v) => glcheck!(gl::Uniform3f(index, v.x, v.y, v.z)),
            ShaderVarData::Float4(v) => glcheck!(gl::Uniform4f(index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Int(v) => glcheck!(gl::Uniform1i(index, v)),
            ShaderVarData::Int2(v) => glcheck!(gl::Uniform2i(index, v.x, v.y)),
            ShaderVarData::Int3(v) => glcheck!(gl::Uniform3i(index, v.x, v.y, v.z)),
            ShaderVarData::Int4(v) => glcheck!(gl::Uniform4i(index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Matrix(m) => Self::index_set_matrix(index, &m),
            ShaderVarData::Tex1D(t) => Self::index_set_tex1d(index, unsafe { &mut *t }),
            ShaderVarData::Tex2D(t) => Self::index_set_tex2d(index, unsafe { &mut *t }),
            ShaderVarData::Tex3D(t) => Self::index_set_tex3d(index, unsafe { &mut *t }),
            ShaderVarData::TexCube(t) => Self::index_set_tex_cube(index, unsafe { &mut *t }),
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

    // Singleton based shader functions - Old API.
    pub fn start(&mut self) {
        unsafe {
            Profiler_Begin(c_str!("Shader_Start"));
        }

        Self::set_current(Some(self));

        glcheck!(gl::UseProgram(self.shared.as_ref().program));

        // Reset the tex index counter.
        *self.shared.as_mut().tex_index.borrow_mut() = 0;

        // Fetch and bind automatic variables from the shader var stack.
        for var in self.shared.as_ref().auto_vars.iter() {
            if var.index == -1 {
                continue;
            }

            let Some(shader_var) = ShaderVar::get(var.name.as_str()) else {
                warn!(
                    "Shader variable stack does not contain variable <{}>",
                    var.name,
                );
                continue;
            };

            if shader_var.get_glsl_type() != var.type_name {
                warn!(
                    "Attempting to get stack of type <{}> for shader variable <{}> when existing stack has type <{}>",
                    var.type_name,
                    var.name,
                    shader_var.get_glsl_type(),
                );
                continue;
            }

            Self::index_set_uniform(var.index, &shader_var);
        }

        unsafe {
            Profiler_End();
        }
    }

    pub fn stop(&self) {
        glcheck!(gl::UseProgram(0));
        Self::set_current(None);
    }

    pub fn reset_tex_index() {
        *Self::get_current_checked()
            .shared
            .as_ref()
            .tex_index
            .borrow_mut() = 0;
    }

    pub fn set_float(name: &str, value: f32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_float(index, value);
        }
    }

    #[bind(name = "ISetFloat")]
    pub fn index_set_float(index: i32, value: f32) {
        glcheck!(gl::Uniform1f(index, value));
    }

    pub fn set_float2(name: &str, x: f32, y: f32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_float2(index, x, y);
        }
    }

    #[bind(name = "ISetFloat2")]
    pub fn index_set_float2(index: i32, x: f32, y: f32) {
        glcheck!(gl::Uniform2f(index, x, y));
    }

    pub fn set_float3(name: &str, x: f32, y: f32, z: f32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_float3(index, x, y, z);
        }
    }

    #[bind(name = "ISetFloat3")]
    pub fn index_set_float3(index: i32, x: f32, y: f32, z: f32) {
        glcheck!(gl::Uniform3f(index, x, y, z));
    }

    pub fn set_float4(name: &str, x: f32, y: f32, z: f32, w: f32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_float4(index, x, y, z, w);
        }
    }

    #[bind(name = "ISetFloat4")]
    pub fn index_set_float4(index: i32, x: f32, y: f32, z: f32, w: f32) {
        glcheck!(gl::Uniform4f(index, x, y, z, w));
    }

    pub fn set_int(name: &str, value: i32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_int(index, value);
        }
    }

    #[bind(name = "ISetInt")]
    pub fn index_set_int(index: i32, value: i32) {
        glcheck!(gl::Uniform1i(index, value));
    }

    pub fn set_int2(name: &str, x: i32, y: i32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_int2(index, x, y);
        }
    }

    #[bind(name = "ISetInt2")]
    pub fn index_set_int2(index: i32, x: i32, y: i32) {
        glcheck!(gl::Uniform2i(index, x, y));
    }

    pub fn set_int3(name: &str, x: i32, y: i32, z: i32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_int3(index, x, y, z);
        }
    }

    #[bind(name = "ISetInt3")]
    pub fn index_set_int3(index: i32, x: i32, y: i32, z: i32) {
        glcheck!(gl::Uniform3i(index, x, y, z));
    }

    pub fn set_int4(name: &str, x: i32, y: i32, z: i32, w: i32) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_int4(index, x, y, z, w);
        }
    }

    #[bind(name = "ISetInt4")]
    pub fn index_set_int4(index: i32, x: i32, y: i32, z: i32, w: i32) {
        glcheck!(gl::Uniform4i(index, x, y, z, w));
    }

    pub fn set_matrix(name: &str, value: &Matrix) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_matrix(index, value);
        }
    }

    #[bind(name = "ISetMatrix")]
    pub fn index_set_matrix(index: i32, value: &Matrix) {
        glcheck!(gl::UniformMatrix4fv(
            index,
            1,
            gl::FALSE,
            value as *const Matrix as *const f32
        ));
    }

    #[bind(name = "SetMatrixT")]
    pub fn set_matrix_transpose(name: &str, value: &Matrix) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_matrix_transpose(index, value);
        }
    }

    #[bind(name = "ISetMatrixT")]
    pub fn index_set_matrix_transpose(index: i32, value: &Matrix) {
        glcheck!(gl::UniformMatrix4fv(
            index,
            1,
            gl::TRUE,
            value as *const Matrix as *const f32
        ));
    }

    pub fn set_tex1d(name: &str, value: &mut Tex1D) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_tex1d(index, value);
        }
    }

    #[bind(name = "ISetTex1D")]
    pub fn index_set_tex1d(index: i32, value: &mut Tex1D) {
        let s = Self::get_current_checked();

        let tex_index = s.next_tex_index();
        glcheck!(gl::Uniform1i(index, tex_index as i32));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, Tex1D_GetHandle(value)));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    }

    pub fn set_tex2d(name: &str, value: &mut Tex2D) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_tex2d(index, value);
        }
    }

    #[bind(name = "ISetTex2D")]
    pub fn index_set_tex2d(index: i32, value: &mut Tex2D) {
        let s = Self::get_current_checked();

        let tex_index = s.next_tex_index();
        glcheck!(gl::Uniform1i(index, tex_index as i32));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, Tex2D_GetHandle(value)));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    }

    pub fn set_tex3d(name: &str, value: &mut Tex3D) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_tex3d(index, value);
        }
    }

    #[bind(name = "ISetTex3D")]
    pub fn index_set_tex3d(index: i32, value: &mut Tex3D) {
        let s = Self::get_current_checked();

        let tex_index = s.next_tex_index();
        glcheck!(gl::Uniform1i(index, tex_index as i32));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, Tex3D_GetHandle(value)));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    }

    pub fn set_tex_cube(name: &str, value: &mut TexCube) {
        if let Some(index) = Self::get_current_checked().get_uniform_index(name) {
            Self::index_set_tex_cube(index, value);
        }
    }

    #[bind(name = "ISetTexCube")]
    pub fn index_set_tex_cube(index: i32, value: &mut TexCube) {
        let s = Self::get_current_checked();

        let tex_index = s.next_tex_index();
        glcheck!(gl::Uniform1i(index, tex_index as i32));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0 + tex_index));
        glcheck!(gl::BindTexture(
            gl::TEXTURE_CUBE_MAP,
            TexCube_GetHandle(value)
        ));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
    }
}

fn create_gl_shader(src: &str, shader_type: gl::types::GLenum) -> u32 {
    let this = glcheck!(gl::CreateShader(shader_type));

    let version_string = c_str!(
        "#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n"
    );
    let shader_source = CString::new(src).expect("Shader source must be utf-8");
    let mut srcs: [*const libc::c_char; 2] = [version_string, shader_source.as_ptr()];
    glcheck!(gl::ShaderSource(
        this,
        2,
        srcs.as_mut_ptr() as *const *const gl::types::GLchar,
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

        panic!(
            "CreateGLShader: Failed to compile shader[{length}]:\n{}",
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

pub fn get_current_program() -> Option<gl::types::GLuint> {
    Shader::get_current().map(|s| s.handle())
}
