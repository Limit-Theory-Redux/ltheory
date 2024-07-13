use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;

use internal::*;

use super::*;
use crate::common::*;
use crate::logging::warn;
use crate::math::*;
use crate::system::*;

#[derive(Default)]
pub struct Shader {
    _refCount: u32,
    name: String,
    vs: u32,
    fs: u32,
    program: u32,
    texIndex: u32,
    auto_vars: Vec<ShaderAutoVar>,
}

struct ShaderAutoVar {
    type_name: String,
    name: String,
    index: i32,
}

const INCLUDE_PATH: &str = "include/";

static mut versionString: *const libc::c_char =
    c_str!("#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n");

static mut current: *mut Shader = std::ptr::null_mut();

static mut CACHE: *mut HashMap<String, String> = std::ptr::null_mut();

fn GetUniformIndex(this: Option<&mut Shader>, name: *const libc::c_char) -> i32 {
    if this.is_none() {
        panic!("GetUniformIndex: No shader is bound");
    }
    let index: i32 = glcheck!(gl::GetUniformLocation(this.unwrap().program, name));
    index
}

fn create_gl_shader(src: &str, type_0: gl::types::GLenum) -> u32 {
    let this = glcheck!(gl::CreateShader(type_0));
    let c_src = CString::new(src).unwrap();

    let mut srcs: [*const libc::c_char; 2] = unsafe { [versionString, c_src.as_ptr()] };
    glcheck!(gl::ShaderSource(
        this,
        2,
        srcs.as_mut_ptr() as *const *const gl::types::GLchar,
        std::ptr::null(),
    ));
    glcheck!(gl::CompileShader(this));

    /* Check for compile errors. */
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

fn CreateGLProgram(vs: gl::types::GLuint, fs: gl::types::GLuint) -> gl::types::GLuint {
    let this: u32 = glcheck!(gl::CreateProgram());
    glcheck!(gl::AttachShader(this, vs));
    glcheck!(gl::AttachShader(this, fs));

    /* TODO : Replace with custom directives. */
    glcheck!(gl::BindAttribLocation(this, 0, c_str!("vertex_position")));
    glcheck!(gl::BindAttribLocation(this, 1, c_str!("vertex_normal")));
    glcheck!(gl::BindAttribLocation(this, 2, c_str!("vertex_uv")));

    glcheck!(gl::LinkProgram(this));

    /* Check for link errors. */
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
            "CreateGLProgram: Failed to link program[{length}]:\n{:?}",
            CStr::from_bytes_with_nul(info_log.as_slice())
        );
    }

    this
}

/* BUG : Cache does not contain information about custom preprocessor
 *       directives, hence cached shaders with custom directives do not work */
fn glsl_load(name: &str, this: &mut Shader) -> String {
    unsafe {
        if CACHE.is_null() {
            let cache = Box::new(HashMap::with_capacity(16));
            CACHE = Box::into_raw(cache);
        }

        let cached = (*CACHE).get(name).cloned();

        if cached.is_some() {
            return cached.unwrap();
        }
    }

    let rawCode = Resource::load_string(ResourceType::Shader, name);
    let code = rawCode.replace("\r\n", "\n");
    let c_code = glsl_preprocess(&code, this);

    /* BUG : Disable GLSL caching until preprocessor cache works. */
    //(*CACHE).insert(name.to_string(), c_code.clone());

    c_code
}

fn glsl_preprocess(code: &str, this: &mut Shader) -> String {
    let mut result = String::new();

    for line in code.lines() {
        if let Some(include_val) = line.strip_prefix("#include ") {
            let include_data = parse_include(include_val, this);

            result += &include_data;
            result += "\n";
        } else if let Some(autovar_val) = line.strip_prefix("#autovar ") {
            parse_autovar(autovar_val, this);
        } else {
            result += line;
            result += "\n";
        }
    }

    result
}

fn parse_include(val: &str, this: &mut Shader) -> String {
    let path = format!("{INCLUDE_PATH}{val}");

    glsl_load(&path, this)
}

fn parse_autovar(val: &str, this: &mut Shader) {
    let line_tokens: Vec<_> = val.split(" ").collect();
    if line_tokens.len() == 2 {
        let var_type = line_tokens[0];
        let var_name = line_tokens[1];
        this.auto_vars.push(ShaderAutoVar {
            type_name: var_type.into(),
            name: var_name.into(),
            index: -1,
        });
    } else {
        warn!("GLSL_Preprocess: Failed to parse autovar directive:\n  {val}");
    }
}

fn Shader_BindVariables(this: &mut Shader) {
    for var in this.auto_vars.iter_mut() {
        let c_str = CString::new(var.name.as_str()).unwrap();
        var.index = glcheck!(gl::GetUniformLocation(this.program, c_str.as_ptr()));
        if var.index < 0 {
            warn!("Shader_BindVariables: Automatic shader variable <{}> does not exist in shader <{}>",
                var.name,
                this.name,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Create(
    vs: *const libc::c_char,
    fs: *const libc::c_char,
) -> Box<Shader> {
    shader_create(&vs.as_str(), &fs.as_str())
}

fn shader_create(vs: &str, fs: &str) -> Box<Shader> {
    let mut this = Box::new(Shader::default());

    this._refCount = 1;
    this.auto_vars = Vec::new();

    let vs = glsl_preprocess(&vs.replace("\r\n", "\n"), this.as_mut());
    let fs = glsl_preprocess(&fs.replace("\r\n", "\n"), this.as_mut());

    this.vs = create_gl_shader(&vs, gl::VERTEX_SHADER);
    this.fs = create_gl_shader(&fs, gl::FRAGMENT_SHADER);
    this.program = CreateGLProgram(this.vs, this.fs);
    this.texIndex = 1;
    this.name = format!("[anonymous shader @ {:p}]", &*this);

    Shader_BindVariables(this.as_mut());

    this
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Load(
    vName: *const libc::c_char,
    fName: *const libc::c_char,
) -> Box<Shader> {
    let mut this = Box::new(Shader::default());

    this._refCount = 1;
    this.auto_vars = Vec::new();

    let vs = glsl_load(&vName.as_str(), this.as_mut());
    let fs = glsl_load(&fName.as_str(), this.as_mut());

    this.vs = create_gl_shader(&vs, gl::VERTEX_SHADER);
    this.fs = create_gl_shader(&fs, gl::FRAGMENT_SHADER);
    this.program = CreateGLProgram(this.vs, this.fs);
    this.texIndex = 1;
    this.name = format!("[vs: {} , fs: {}]", vName.as_str(), fName.as_str());

    Shader_BindVariables(this.as_mut());

    this
}

#[no_mangle]
pub extern "C" fn Shader_Acquire(this: &mut Shader) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Free(this: *mut Shader) {
    if {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        glcheck!(gl::DeleteShader((*this).vs));
        glcheck!(gl::DeleteShader((*this).fs));
        glcheck!(gl::DeleteProgram((*this).program));
        drop(Box::from_raw(this));
    }
}

#[no_mangle]
pub extern "C" fn Shader_ToShaderState(this: &mut Shader) -> Box<ShaderState> {
    ShaderState_Create(this)
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Start(this: &mut Shader) {
    Profiler_Begin(c_str!("Shader_Start"));

    glcheck!(gl::UseProgram(this.program));

    current = this;
    this.texIndex = 1;

    // Fetch and bind automatic variables from the shader var stack.
    for var in this.auto_vars.iter() {
        if var.index == -1 {
            continue;
        }

        let Some(shader_var) = ShaderVar::get(var.name.as_str()) else {
            warn!(
                "Shader_Start: Shader variable stack does not contain variable <{}>",
                var.name,
            );
            continue;
        };

        if shader_var.get_glsl_type() != var.type_name {
            warn!(
                "Attempting to get stack of type <{:?}> for shader variable <{:?}> when existing stack has type <{:?}>",
                var.type_name,
                var.name,
                shader_var.get_glsl_type(),
            );
            continue;
        }

        match shader_var {
            ShaderVarData::Float(v) => glcheck!(gl::Uniform1f(var.index, v)),
            ShaderVarData::Float2(v) => glcheck!(gl::Uniform2f(var.index, v.x, v.y)),
            ShaderVarData::Float3(v) => glcheck!(gl::Uniform3f(var.index, v.x, v.y, v.z)),
            ShaderVarData::Float4(v) => glcheck!(gl::Uniform4f(var.index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Int(v) => glcheck!(gl::Uniform1i(var.index, v)),
            ShaderVarData::Int2(v) => glcheck!(gl::Uniform2i(var.index, v.x, v.y)),
            ShaderVarData::Int3(v) => glcheck!(gl::Uniform3i(var.index, v.x, v.y, v.z)),
            ShaderVarData::Int4(v) => glcheck!(gl::Uniform4i(var.index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Matrix(m) => Shader_ISetMatrix(var.index, &m),
            ShaderVarData::Tex1D(t) => Shader_ISetTex1D(var.index, &mut *t),
            ShaderVarData::Tex2D(t) => Shader_ISetTex2D(var.index, &mut *t),
            ShaderVarData::Tex3D(t) => Shader_ISetTex3D(var.index, &mut *t),
            ShaderVarData::TexCube(t) => Shader_ISetTexCube(var.index, &mut *t),
        }
    }

    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Stop(_: &Shader) {
    glcheck!(gl::UseProgram(0));
    current = std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn Shader_ClearCache() {
    unsafe {
        if !CACHE.is_null() {
            (*CACHE).clear();
            let _cache = Box::from(CACHE);
            CACHE = std::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub extern "C" fn Shader_GetHandle(this: &mut Shader) -> u32 {
    this.program
}

#[no_mangle]
pub extern "C" fn Shader_GetVariable(this: &mut Shader, name: *const libc::c_char) -> i32 {
    let index = glcheck!(gl::GetUniformLocation(this.program, name));
    if index == -1 {
        panic!(
            "Shader_GetVariable: Shader <{}> has no variable <{}>",
            this.name,
            name.as_str(),
        );
    }
    index
}

#[no_mangle]
pub extern "C" fn Shader_HasVariable(this: &mut Shader, name: *const libc::c_char) -> bool {
    glcheck!(gl::GetUniformLocation(this.program, name)) > -1
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ResetTexIndex() {
    (*current).texIndex = 1;
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat(name: *const libc::c_char, value: f32) {
    glcheck!(gl::Uniform1f(
        GetUniformIndex(current.as_mut(), name),
        value
    ));
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat(index: i32, value: f32) {
    glcheck!(gl::Uniform1f(index, value));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat2(name: *const libc::c_char, x: f32, y: f32) {
    glcheck!(gl::Uniform2f(GetUniformIndex(current.as_mut(), name), x, y));
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat2(index: i32, x: f32, y: f32) {
    glcheck!(gl::Uniform2f(index, x, y));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat3(name: *const libc::c_char, x: f32, y: f32, z: f32) {
    glcheck!(gl::Uniform3f(
        GetUniformIndex(current.as_mut(), name),
        x,
        y,
        z
    ));
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat3(index: i32, x: f32, y: f32, z: f32) {
    glcheck!(gl::Uniform3f(index, x, y, z));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat4(
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    glcheck!(gl::Uniform4f(
        GetUniformIndex(current.as_mut(), name),
        x,
        y,
        z,
        w
    ));
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat4(index: i32, x: f32, y: f32, z: f32, w: f32) {
    glcheck!(gl::Uniform4f(index, x, y, z, w));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetInt(name: *const libc::c_char, value: i32) {
    glcheck!(gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        value
    ));
}

#[no_mangle]
pub extern "C" fn Shader_ISetInt(index: i32, value: i32) {
    glcheck!(gl::Uniform1i(index, value));
}

#[no_mangle]
pub extern "C" fn Shader_SetMatrix(name: *const libc::c_char, value: &Matrix) {
    glcheck!(gl::UniformMatrix4fv(
        GetUniformIndex(current.as_mut(), name),
        1,
        gl::FALSE,
        value as *const Matrix as *const f32,
    ));
}

#[no_mangle]
pub extern "C" fn Shader_SetMatrixT(name: *const libc::c_char, value: &Matrix) {
    glcheck!(gl::UniformMatrix4fv(
        GetUniformIndex(current.as_mut(), name),
        1,
        gl::TRUE,
        value as *const Matrix as *const f32,
    ));
}

#[no_mangle]
pub extern "C" fn Shader_ISetMatrix(index: i32, value: &Matrix) {
    glcheck!(gl::UniformMatrix4fv(
        index,
        1,
        gl::FALSE,
        value as *const Matrix as *const f32
    ));
}

#[no_mangle]
pub extern "C" fn Shader_ISetMatrixT(index: i32, value: &Matrix) {
    glcheck!(gl::UniformMatrix4fv(
        index,
        1,
        gl::TRUE,
        value as *const Matrix as *const f32
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex1D(name: *const libc::c_char, value: &mut Tex1D) {
    glcheck!(gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    ));

    let fresh14 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh14)));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, Tex1D_GetHandle(value)));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex1D(index: i32, value: &mut Tex1D) {
    glcheck!(gl::Uniform1i(index, (*current).texIndex as i32));

    let fresh15 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh15)));
    glcheck!(gl::BindTexture(gl::TEXTURE_1D, Tex1D_GetHandle(value)));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex2D(name: *const libc::c_char, value: &mut Tex2D) {
    glcheck!(gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    ));

    let fresh16 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh16)));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, Tex2D_GetHandle(value)));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex2D(index: i32, value: &mut Tex2D) {
    glcheck!(gl::Uniform1i(index, (*current).texIndex as i32));

    let fresh17 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh17)));
    glcheck!(gl::BindTexture(gl::TEXTURE_2D, Tex2D_GetHandle(value)));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex3D(name: *const libc::c_char, value: &mut Tex3D) {
    glcheck!(gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    ));

    let fresh18 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh18)));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, Tex3D_GetHandle(value)));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex3D(index: i32, value: &mut Tex3D) {
    glcheck!(gl::Uniform1i(index, (*current).texIndex as i32));

    let fresh19 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh19)));
    glcheck!(gl::BindTexture(gl::TEXTURE_3D, Tex3D_GetHandle(value)));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTexCube(name: *const libc::c_char, value: &mut TexCube) {
    glcheck!(gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    ));

    let fresh20 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh20)));
    glcheck!(gl::BindTexture(
        gl::TEXTURE_CUBE_MAP,
        TexCube_GetHandle(value)
    ));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTexCube(index: i32, value: &mut TexCube) {
    glcheck!(gl::Uniform1i(index, (*current).texIndex as i32));

    let fresh21 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    glcheck!(gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh21)));
    glcheck!(gl::BindTexture(
        gl::TEXTURE_CUBE_MAP,
        TexCube_GetHandle(value)
    ));
    glcheck!(gl::ActiveTexture(gl::TEXTURE0));
}

pub fn get_current_program() -> Option<gl::types::GLuint> {
    unsafe { current.as_ref().map(|r| r.program) }
}
