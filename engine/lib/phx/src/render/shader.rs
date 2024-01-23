use std::ffi::CStr;
use std::ffi::CString;

use internal::*;

use super::*;
use crate::common::*;
use crate::logging::warn;
use crate::math::*;
use crate::system::*;
use crate::*;

#[derive(Default)]
#[repr(C)]
pub struct Shader {
    pub _refCount: u32,
    pub name: String,
    pub vs: u32,
    pub fs: u32,
    pub program: u32,
    pub texIndex: u32,
    pub vars: Vec<ShaderVar>,
}

pub struct ShaderVar {
    pub type_0: ShaderVarType,
    pub name: String,
    pub index: i32,
}

const INCLUDE_PATH: &str = "include/";

static mut versionString: *const libc::c_char =
    c_str!("#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n");

static mut current: *mut Shader = std::ptr::null_mut();

static mut cache: *mut StrMap = std::ptr::null_mut();

extern "C" fn GetUniformIndex(this: Option<&mut Shader>, name: *const libc::c_char) -> i32 {
    0
    // if this.is_none() {
    //     panic!("GetUniformIndex: No shader is bound");
    // }
    // let index: i32 = gl_get_uniform_location(this.unwrap().program, name);
    // index
}

// unsafe fn create_gl_shader(src: &str, type_0: gl::types::GLenum) -> u32 {
// let this = gl_create_shader(type_0);
//     let c_src = CString::new(src).unwrap();

//     let mut srcs: [*const libc::c_char; 2] = [versionString, c_src.as_ptr()];

//     gl_shader_source(
//         this,
//         2,
//         srcs.as_mut_ptr() as *const *const gl::types::GLchar,
//         std::ptr::null(),
//     );
//     gl_compile_shader(this);

//     /* Check for compile errors. */
//     let mut status = 0;
//     gl_get_shaderiv(this, gl::COMPILE_STATUS, &mut status);

//     if status != gl::TRUE as i32 {
//         let mut length = 0;
//         gl_get_shaderiv(this, gl::INFO_LOG_LENGTH, &mut length);

//         let mut info_log = vec![0; length as usize + 1];
//         gl_get_shader_info_log(
//             this,
//             length,
//             std::ptr::null_mut(),
//             info_log.as_mut_ptr() as *mut i8,
//         );

//         panic!(
//             "CreateGLShader: Failed to compile shader[{length}]:\n{}",
//             String::from_utf8(info_log).unwrap()
//         );
//     }

//     this
// }

extern "C" fn CreateGLProgram(vs: u32, fs: u32) -> u32 {
    0
    // let this: u32 = gl_create_program();
    // gl_attach_shader(this, vs);
    // gl_attach_shader(this, fs);

    // /* TODO : Replace with custom directives. */
    // gl_bind_attrib_location(this, 0, c_str!("vertex_position"));
    // gl_bind_attrib_location(this, 1, c_str!("vertex_normal"));
    // gl_bind_attrib_location(this, 2, c_str!("vertex_uv"));

    // gl_link_program(this);

    // /* Check for link errors. */
    // let mut status: i32 = 0;
    // gl_get_programiv(this, gl::LINK_STATUS, &mut status);

    // if status != gl::TRUE as i32 {
    //     let mut length: i32 = 0;
    //     gl_get_programiv(this, gl::INFO_LOG_LENGTH, &mut length);

    //     let mut info_log = vec![0; length as usize + 1];
    //     gl_get_program_info_log(
    //         this,
    //         length,
    //         std::ptr::null_mut(),
    //         info_log.as_mut_ptr() as *mut i8,
    //     );

    //     panic!(
    //         "CreateGLProgram: Failed to link program[{length}]:\n{:?}",
    //         CStr::from_bytes_with_nul(info_log.as_slice())
    //     );
    // }

    // this
}

/* BUG : Cache does not contain information about custom preprocessor
 *       directives, hence cached shaders with custom directives do not work */
// unsafe fn glsl_load(name: &str, this: &mut Shader) -> String {
//     if cache.is_null() {
//         cache = StrMap_Create(16);
//     }

//     let c_name = CString::new(name).unwrap();
//     let cached: *mut libc::c_void = StrMap_Get(&mut *cache, c_name.as_ptr());

//     if !cached.is_null() {
//         return (cached as *const libc::c_char).as_string();
//     }

//     let rawCode = Resource::load_string(ResourceType::Shader, name);
//     let code = rawCode.replace("\r\n", "\n");
//     let c_code = glsl_preprocess(&code, this);

//     /* BUG : Disable GLSL caching until preprocessor cache works. */
//     // StrMap_Set(cache, name, (void*)code);

//     c_code
// }

unsafe fn glsl_preprocess(code: &str, this: &mut Shader) -> String {
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

unsafe fn parse_include(val: &str, this: &mut Shader) -> String {
    let path = format!("{INCLUDE_PATH}{val}");
    String::new()

    // glsl_load(&path, this)
}

fn parse_autovar(val: &str, this: &mut Shader) {
    // let line_tokens: Vec<_> = val.split(" ").collect();

    // if line_tokens.len() == 2 {
    //     let var_type = line_tokens[0];
    //     let var_name = line_tokens[1];
    //     let var = ShaderVar {
    //         type_0: ShaderVarType::from_str(var_type),
    //         name: var_name.into(),
    //         index: -1,
    //     };

    //     if var.type_0 == ShaderVarType::UNKNOWN {
    //         panic!(
    //             "GLSL_Preprocess: Unknown shader variable type <{var_type}> in autovar directive:\n  {val}"
    //         );
    //     }

    //     this.vars.push(var);
    // } else {
    //     panic!("GLSL_Preprocess: Failed to parse autovar directive:\n  {val}");
    // }
}

extern "C" fn Shader_BindVariables(this: &mut Shader) {
    // let mut i: i32 = 0;

    // while i < this.vars.len() as i32 {
    //     let var = &mut this.vars[i as usize];
    //     let c_name = static_string!((*var).name.as_str());

    //     (*var).index = gl_get_uniform_location(this.program, c_name);

    //     if (*var).index < 0 {
    //         warn!("Shader_BindVariables: Automatic shader variable <{}> does not exist in shader <{}>",
    //             var.name,
    //             this.name,
    //         );
    //     }

    //     i += 1;
    // }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Create(
    vs: *const libc::c_char,
    fs: *const libc::c_char,
) -> Box<Shader> {
    shader_create(&vs.as_str(), &fs.as_str())
}

pub unsafe fn shader_create(vs: &str, fs: &str) -> Box<Shader> {
    let mut this = Box::new(Shader::default());

    // this._refCount = 1;
    // this.vars = Vec::new();

    // let vs = glsl_preprocess(&vs.replace("\r\n", "\n"), this.as_mut());
    // let fs = glsl_preprocess(&fs.replace("\r\n", "\n"), this.as_mut());

    // this.vs = create_gl_shader(&vs, gl::VERTEX_SHADER);
    // this.fs = create_gl_shader(&fs, gl::FRAGMENT_SHADER);
    // this.program = CreateGLProgram(this.vs, this.fs);
    // this.texIndex = 1;
    // this.name = format!("[anonymous shader @ {:p}]", &*this);

    // Shader_BindVariables(this.as_mut());

    this
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Load(
    vName: *const libc::c_char,
    fName: *const libc::c_char,
) -> Box<Shader> {
    let mut this = Box::new(Shader::default());

    // this._refCount = 1;
    // this.vars = Vec::new();

    // let vs = glsl_load(&vName.as_str(), this.as_mut());
    // let fs = glsl_load(&fName.as_str(), this.as_mut());

    // this.vs = create_gl_shader(&vs, gl::VERTEX_SHADER);
    // this.fs = create_gl_shader(&fs, gl::FRAGMENT_SHADER);
    // this.program = CreateGLProgram(this.vs, this.fs);
    // this.texIndex = 1;
    // this.name = format!("[vs: {} , fs: {}]", vName.as_str(), fName.as_str());

    // Shader_BindVariables(this.as_mut());

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
        // gl_delete_shader((*this).vs);
        // gl_delete_shader((*this).fs);
        // gl_delete_program((*this).program);
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

    // gl_use_program(this.program);

    current = this;
    this.texIndex = 1;

    /* Fetch & bind automatic variables from the shader var stack. */
    let mut i: i32 = 0;
    while i < this.vars.len() as i32 {
        let var = &mut this.vars[i as usize];

        if !((*var).index < 0) {
            // TODO: investigate why pinning is needed here
            let c_name = static_string!((*var).name.as_str());
            let pValue = ShaderVar_Get(c_name, (*var).type_0);

            if pValue.is_null() {
                panic!(
                    "Shader_Start: Shader variable stack does not contain variable <{}>",
                    (*var).name,
                );
            }

            // match (*var).type_0.value() {
            //     1 => {
            //         let value: f32 = *(pValue as *mut f32);
            //         gl_uniform1f((*var).index, value);
            //     }
            //     2 => {
            //         let value_0 = *(pValue as *mut Vec2);
            //         gl_uniform2f((*var).index, value_0.x, value_0.y);
            //     }
            //     3 => {
            //         let value_1: Vec3 = *(pValue as *mut Vec3);
            //         gl_uniform3f((*var).index, value_1.x, value_1.y, value_1.z);
            //     }
            //     4 => {
            //         let value_2: Vec4 = *(pValue as *mut Vec4);
            //         gl_uniform4f((*var).index, value_2.x, value_2.y, value_2.z, value_2.w);
            //     }
            //     5 => {
            //         let value_3: i32 = *(pValue as *mut i32);
            //         gl_uniform1i((*var).index, value_3);
            //     }
            //     6 => {
            //         let value_4: IVec2 = *(pValue as *mut IVec2);
            //         gl_uniform2i((*var).index, value_4.x, value_4.y);
            //     }
            //     7 => {
            //         let value_5: IVec3 = *(pValue as *mut IVec3);
            //         gl_uniform3i((*var).index, value_5.x, value_5.y, value_5.z);
            //     }
            //     8 => {
            //         let value_6: IVec4 = *(pValue as *mut IVec4);
            //         gl_uniform4i((*var).index, value_6.x, value_6.y, value_6.z, value_6.w);
            //     }
            //     9 => {
            //         Shader_ISetMatrix((*var).index, &mut **(pValue as *mut *mut Matrix));
            //     }
            //     10 => {
            //         Shader_ISetTex1D((*var).index, &mut **(pValue as *mut *mut Tex1D));
            //     }
            //     11 => {
            //         Shader_ISetTex2D((*var).index, &mut **(pValue as *mut *mut Tex2D));
            //     }
            //     12 => {
            //         Shader_ISetTex3D((*var).index, &mut **(pValue as *mut *mut Tex3D));
            //     }
            //     13 => {
            //         Shader_ISetTexCube((*var).index, &mut **(pValue as *mut *mut TexCube));
            //     }
            //     _ => {}
            // }
        }
        i += 1;
    }

    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Stop(_s: *mut Shader) {
    // gl_use_program(0);
    current = std::ptr::null_mut();
}

unsafe extern "C" fn ShaderCache_FreeElem(_s: *const libc::c_char, data: *mut libc::c_void) {
    MemFree(data);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ClearCache() {
    if !cache.is_null() {
        StrMap_FreeEx(
            &mut *cache,
            Some(
                ShaderCache_FreeElem
                    as unsafe extern "C" fn(*const libc::c_char, *mut libc::c_void) -> (),
            ),
        );
        cache = std::ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn Shader_GetHandle(this: &mut Shader) -> u32 {
    this.program
}

#[no_mangle]
pub extern "C" fn Shader_GetVariable(this: &mut Shader, name: *const libc::c_char) -> i32 {
    // let index: i32 = gl_get_uniform_location(this.program, name);
    // if index == -1 {
    //     panic!(
    //         "Shader_GetVariable: Shader <{}> has no variable <{}>",
    //         this.name,
    //         name.as_str(),
    //     );
    // }
    // index
    0
}

#[no_mangle]
pub extern "C" fn Shader_HasVariable(this: &mut Shader, name: *const libc::c_char) -> bool {
    // gl_get_uniform_location(this.program, name) > -1
    false
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ResetTexIndex() {
    (*current).texIndex = 1;
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat(name: *const libc::c_char, value: f32) {
    // gl_uniform1f(GetUniformIndex(current.as_mut(), name), value);
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat(index: i32, value: f32) {
    // gl_uniform1f(index, value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat2(name: *const libc::c_char, x: f32, y: f32) {
    // gl_uniform2f(GetUniformIndex(current.as_mut(), name), x, y);
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat2(index: i32, x: f32, y: f32) {
    // gl_uniform2f(index, x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat3(name: *const libc::c_char, x: f32, y: f32, z: f32) {
    // gl_uniform3f(GetUniformIndex(current.as_mut(), name), x, y, z);
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat3(index: i32, x: f32, y: f32, z: f32) {
    // gl_uniform3f(index, x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat4(
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    // gl_uniform4f(GetUniformIndex(current.as_mut(), name), x, y, z, w);
}

#[no_mangle]
pub extern "C" fn Shader_ISetFloat4(index: i32, x: f32, y: f32, z: f32, w: f32) {
    // gl_uniform4f(index, x, y, z, w);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetInt(name: *const libc::c_char, value: i32) {
    // gl_uniform1i(GetUniformIndex(current.as_mut(), name), value);
}

#[no_mangle]
pub extern "C" fn Shader_ISetInt(index: i32, value: i32) {
    // gl_uniform1i(index, value);
}

#[no_mangle]
pub extern "C" fn Shader_SetMatrix(name: *const libc::c_char, value: &mut Matrix) {
    // gl_uniform_matrix4fv(
    //     unsafe { GetUniformIndex(current.as_mut(), name) },
    //     1,
    //     gl::FALSE,
    //     value as *mut Matrix as *mut f32,
    // );
}

#[no_mangle]
pub extern "C" fn Shader_SetMatrixT(name: *const libc::c_char, value: &mut Matrix) {
    // gl_uniform_matrix4fv(
    //     unsafe { GetUniformIndex(current.as_mut(), name) },
    //     1,
    //     gl::TRUE,
    //     value as *mut Matrix as *mut f32,
    // );
}

#[no_mangle]
pub extern "C" fn Shader_ISetMatrix(index: i32, value: &mut Matrix) {
    // gl_uniform_matrix4fv(index, 1, gl::FALSE, value as *mut Matrix as *mut f32);
}

#[no_mangle]
pub extern "C" fn Shader_ISetMatrixT(index: i32, value: &mut Matrix) {
    // gl_uniform_matrix4fv(index, 1, gl::TRUE, value as *mut Matrix as *mut f32);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex1D(name: *const libc::c_char, value: &mut Tex1D) {
    // gl_uniform1i(
    //     GetUniformIndex(current.as_mut(), name),
    //     (*current).texIndex as i32,
    // );

    // let fresh14 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh14));
    // gl_bind_texture(gl::TEXTURE_1D, Tex1D_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex1D(index: i32, value: &mut Tex1D) {
    // gl_uniform1i(index, (*current).texIndex as i32);

    // let fresh15 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh15));
    // gl_bind_texture(gl::TEXTURE_1D, Tex1D_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex2D(name: *const libc::c_char, value: &mut Tex2D) {
    //     gl_uniform1i(
    //         GetUniformIndex(current.as_mut(), name),
    //         (*current).texIndex as i32,
    //     );

    //     let fresh16 = (*current).texIndex;
    //     (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    //     gl_active_texture((gl::TEXTURE0).wrapping_add(fresh16));
    //     gl_bind_texture(gl::TEXTURE_2D, Tex2D_GetHandle(value));
    //     gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex2D(index: i32, value: &mut Tex2D) {
    // gl_uniform1i(index, (*current).texIndex as i32);

    // let fresh17 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh17));
    // gl_bind_texture(gl::TEXTURE_2D, Tex2D_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex3D(name: *const libc::c_char, value: &mut Tex3D) {
    // gl_uniform1i(
    //     GetUniformIndex(current.as_mut(), name),
    //     (*current).texIndex as i32,
    // );

    // let fresh18 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh18));
    // gl_bind_texture(gl::TEXTURE_3D, Tex3D_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex3D(index: i32, value: &mut Tex3D) {
    // gl_uniform1i(index, (*current).texIndex as i32);

    // let fresh19 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh19));
    // gl_bind_texture(gl::TEXTURE_3D, Tex3D_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTexCube(name: *const libc::c_char, value: &mut TexCube) {
    // gl_uniform1i(
    //     GetUniformIndex(current.as_mut(), name),
    //     (*current).texIndex as i32,
    // );

    // let fresh20 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh20));
    // gl_bind_texture(gl::TEXTURE_CUBE_MAP, TexCube_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTexCube(index: i32, value: &mut TexCube) {
    // gl_uniform1i(index, (*current).texIndex as i32);

    // let fresh21 = (*current).texIndex;
    // (*current).texIndex = ((*current).texIndex).wrapping_add(1);

    // gl_active_texture((gl::TEXTURE0).wrapping_add(fresh21));
    // gl_bind_texture(gl::TEXTURE_CUBE_MAP, TexCube_GetHandle(value));
    // gl_active_texture(gl::TEXTURE0);
}
