use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::Math::Vec3;
use crate::phx::Math::Vec4;
use crate::phx::Math::{IVec2, IVec3, IVec4, Vec2};
use crate::phx::Matrix::*;
use crate::phx::Profiler::*;
use crate::phx::Resource::*;
use crate::phx::ResourceType::*;
use crate::phx::ShaderState::*;
use crate::phx::ShaderVar::*;
use crate::phx::ShaderVarType::*;
use crate::phx::StrMap::*;
use crate::phx::Tex1D::*;
use crate::phx::Tex2D::*;
use crate::phx::Tex3D::*;
use crate::phx::TexCube::*;
use crate::phx::GL::gl;
use crate::phx::internal::ffi;

#[derive(Clone)]
#[repr(C)]
pub struct Shader {
    pub _refCount: u32,
    pub name: *const libc::c_char,
    pub vs: u32,
    pub fs: u32,
    pub program: u32,
    pub texIndex: u32,
    pub vars: Vec<ShaderVar>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShaderVar {
    pub type_0: ShaderVarType,
    pub name: *const libc::c_char,
    pub index: i32,
}

static mut includePath: *const libc::c_char = c_str!("include/");

static mut versionString: *const libc::c_char =
    c_str!("#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n");

static mut current: *mut Shader = std::ptr::null_mut();

static mut cache: *mut StrMap = std::ptr::null_mut();

unsafe extern "C" fn GetUniformIndex(this: Option<&mut Shader>, name: *const libc::c_char) -> i32 {
    if this.is_none() {
        CFatal!("GetUniformIndex: No shader is bound");
    }
    let index: i32 = gl::GetUniformLocation(this.unwrap().program, name);
    index
}

unsafe fn CreateGLShader(src: *const libc::c_char, type_0: gl::types::GLenum) -> u32 {
    let this: u32 = gl::CreateShader(type_0);

    let mut srcs: [*const libc::c_char; 2] = [versionString, src];

    gl::ShaderSource(
        this,
        2,
        srcs.as_mut_ptr() as *const *const gl::types::GLchar,
        std::ptr::null(),
    );
    gl::CompileShader(this);

    /* Check for compile errors. */
    let mut status: i32 = 0;
    gl::GetShaderiv(this, gl::COMPILE_STATUS, &mut status);
    if status == 0 {
        let mut length: i32 = 0;
        gl::GetShaderiv(this, gl::INFO_LOG_LENGTH, &mut length);
        let infoLog: *mut libc::c_char = MemAllocZero((length + 1) as usize) as *mut libc::c_char;
        gl::GetShaderInfoLog(this, length, std::ptr::null_mut(), infoLog);
        CFatal!("CreateGLShader: Failed to compile shader:\n%s", infoLog,);
    }
    this
}

unsafe extern "C" fn CreateGLProgram(vs: u32, fs: u32) -> u32 {
    let this: u32 = gl::CreateProgram();
    gl::AttachShader(this, vs);
    gl::AttachShader(this, fs);

    /* TODO : Replace with custom directives. */
    gl::BindAttribLocation(this, 0, c_str!("vertex_position"));
    gl::BindAttribLocation(this, 1, c_str!("vertex_normal"));
    gl::BindAttribLocation(this, 2, c_str!("vertex_uv"));

    gl::LinkProgram(this);

    /* Check for link errors. */
    let mut status: i32 = 0;
    gl::GetProgramiv(this, gl::LINK_STATUS, &mut status);
    if status == 0 {
        let mut length: i32 = 0;
        gl::GetProgramiv(this, gl::INFO_LOG_LENGTH, &mut length);
        let infoLog: *mut libc::c_char = MemAllocZero((length + 1) as usize) as *mut libc::c_char;
        gl::GetProgramInfoLog(this, length, std::ptr::null_mut(), infoLog);
        CFatal!("CreateGLProgram: Failed to link program:\n%s", infoLog,);
    }
    this
}

/* BUG : Cache does not contain information about custom preprocessor
 *       directives, hence cached shaders with custom directives do not work */
unsafe fn GLSL_Load(name: *const libc::c_char, this: &mut Shader) -> *const libc::c_char {
    if cache.is_null() {
        cache = StrMap_Create(16);
    }
    let cached: *mut libc::c_void = StrMap_Get(&mut *cache, name);
    if !cached.is_null() {
        return cached as *const libc::c_char;
    }
    let rawCode: *const libc::c_char = Resource_LoadCstr(ResourceType_Shader, name);
    let mut code: *const libc::c_char = StrReplace(rawCode, c_str!("\r\n"), c_str!("\n"));
    StrFree(rawCode);
    code = GLSL_Preprocess(code, this);
    /* BUG : Disable GLSL caching until preprocessor cache works. */
    // StrMap_Set(cache, name, (void*)code);
    code
}

unsafe fn GLSL_Preprocess(mut code: *const libc::c_char, this: &mut Shader) -> *const libc::c_char {
    let lenInclude: i32 = StrLen(c_str!("#include")) as i32;
    let mut begin: *const libc::c_char = std::ptr::null();

    /* Parse Includes. */
    loop {
        begin = StrFind(code, c_str!("#include"));
        if begin.is_null() {
            break;
        }
        let end: *const libc::c_char = StrFind(begin, c_str!("\n"));
        let name: *const libc::c_char = StrSubStr(begin.offset(lenInclude as isize).offset(1), end);
        let path: *const libc::c_char = StrAdd(includePath, name);
        let prev: *const libc::c_char = code;
        code = StrSub(code, begin, end, GLSL_Load(path, this));
        StrFree(prev);
        StrFree(path);
        StrFree(name);
    }

    /* Parse automatic ShaderVar stack bindings. */
    loop {
        begin = StrFind(code, c_str!("#autovar"));
        if begin.is_null() {
            break;
        }
        let end_0: *const libc::c_char = StrFind(begin, c_str!("\n"));
        let line: *const libc::c_char = StrSubStr(begin, end_0);

        let lineStr = ffi::PtrAsString(line);
        let lineTokens: Vec<&str> = lineStr.split(" ").collect();
        if lineTokens.len() == 3 && lineTokens[0] == "#autovar" {
            let varType = ffi::NewCString(lineTokens[1].to_string());
            let varName = ffi::NewCString(lineTokens[2].to_string());
            let mut var: ShaderVar = ShaderVar {
                type_0: 0,
                name: std::ptr::null(),
                index: 0,
            };
            var.type_0 = ShaderVarType_FromStr(varType.as_ptr());
            if var.type_0 == 0 {
                CFatal!(
                    "GLSL_Preprocess: Unknown shader variable type <%s> in directive:\n  %s",
                    varType.as_ptr(),
                    line,
                );
            }
            var.name = StrDup(varName.as_ptr());
            var.index = -1;
            this.vars.push(var);
        } else {
            CFatal!("GLSL_Preprocess: Failed to parse directive:\n  %s", line,);
        }

        let prev_0: *const libc::c_char = code;
        code = StrSub(code, begin, end_0, c_str!(""));
        StrFree(prev_0);
        StrFree(line);
    }
    code
}

unsafe extern "C" fn Shader_BindVariables(this: &mut Shader) {
    let mut i: i32 = 0;
    while i < this.vars.len() as i32 {
        let var: &mut ShaderVar = &mut this.vars[i as usize];
        (*var).index = gl::GetUniformLocation(this.program, (*var).name);
        if (*var).index < 0 {
            CWarn!("Shader_BindVariables: Automatic shader variable <%s> does not exist in shader <%s>",
                (*var).name,
                this.name,
            );
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Create(
    vs: *const libc::c_char,
    fs: *const libc::c_char,
) -> *mut Shader {
    let this = MemNew!(Shader);
    (*this)._refCount = 1;
    (*this).vars = Vec::new();
    let vs = GLSL_Preprocess(StrReplace(vs, c_str!("\r\n"), c_str!("\n")), &mut *this);
    let fs = GLSL_Preprocess(StrReplace(fs, c_str!("\r\n"), c_str!("\n")), &mut *this);
    (*this).vs = CreateGLShader(vs, gl::VERTEX_SHADER);
    (*this).fs = CreateGLShader(fs, gl::FRAGMENT_SHADER);
    (*this).program = CreateGLProgram((*this).vs, (*this).fs);
    (*this).texIndex = 1;
    (*this).name = StrFormat(c_str!("[anonymous shader @ %p]"), this);
    StrFree(vs);
    StrFree(fs);
    Shader_BindVariables(&mut *this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Load(
    vName: *const libc::c_char,
    fName: *const libc::c_char,
) -> *mut Shader {
    let this = MemNew!(Shader);
    (*this)._refCount = 1;
    (*this).vars = Vec::new();
    let vs: *const libc::c_char = GLSL_Load(vName, &mut *this);
    let fs: *const libc::c_char = GLSL_Load(fName, &mut *this);
    (*this).vs = CreateGLShader(vs, gl::VERTEX_SHADER);
    (*this).fs = CreateGLShader(fs, gl::FRAGMENT_SHADER);
    (*this).program = CreateGLProgram((*this).vs, (*this).fs);
    (*this).texIndex = 1;
    (*this).name = StrFormat(c_str!("[vs: %s , fs: %s]"), vName, fName);
    Shader_BindVariables(&mut *this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Acquire(this: &mut Shader) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Free(this: *mut Shader) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        gl::DeleteShader((*this).vs);
        gl::DeleteShader((*this).fs);
        gl::DeleteProgram((*this).program);
        StrFree((*this).name);
        MemDelete!(this);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ToShaderState(this: &mut Shader) -> *mut ShaderState {
    ShaderState_Create(this)
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Start(this: &mut Shader) {
    Profiler_Begin(c_str!("Shader_Start"));
    gl::UseProgram(this.program);
    current = this;
    this.texIndex = 1;

    /* Fetch & bind automatic variables from the shader var stack. */
    let mut i: i32 = 0;
    while i < this.vars.len() as i32 {
        let var: &mut ShaderVar = &mut this.vars[i as usize];
        if !((*var).index < 0) {
            let pValue: *mut libc::c_void = ShaderVar_Get((*var).name, (*var).type_0);
            if pValue.is_null() {
                CFatal!(
                    "Shader_Start: Shader variable stack does not contain variable <%s>",
                    (*var).name,
                );
            }

            match (*var).type_0 {
                1 => {
                    let value: f32 = *(pValue as *mut f32);
                    gl::Uniform1f((*var).index, value);
                }
                2 => {
                    let value_0 = *(pValue as *mut Vec2);
                    gl::Uniform2f((*var).index, value_0.x, value_0.y);
                }
                3 => {
                    let value_1: Vec3 = *(pValue as *mut Vec3);
                    gl::Uniform3f((*var).index, value_1.x, value_1.y, value_1.z);
                }
                4 => {
                    let value_2: Vec4 = *(pValue as *mut Vec4);
                    gl::Uniform4f((*var).index, value_2.x, value_2.y, value_2.z, value_2.w);
                }
                5 => {
                    let value_3: i32 = *(pValue as *mut i32);
                    gl::Uniform1i((*var).index, value_3);
                }
                6 => {
                    let value_4: IVec2 = *(pValue as *mut IVec2);
                    gl::Uniform2i((*var).index, value_4.x, value_4.y);
                }
                7 => {
                    let value_5: IVec3 = *(pValue as *mut IVec3);
                    gl::Uniform3i((*var).index, value_5.x, value_5.y, value_5.z);
                }
                8 => {
                    let value_6: IVec4 = *(pValue as *mut IVec4);
                    gl::Uniform4i((*var).index, value_6.x, value_6.y, value_6.z, value_6.w);
                }
                9 => {
                    Shader_ISetMatrix((*var).index, &mut **(pValue as *mut *mut Matrix));
                }
                10 => {
                    Shader_ISetTex1D((*var).index, &mut **(pValue as *mut *mut Tex1D));
                }
                11 => {
                    Shader_ISetTex2D((*var).index, &mut **(pValue as *mut *mut Tex2D));
                }
                12 => {
                    Shader_ISetTex3D((*var).index, &mut **(pValue as *mut *mut Tex3D));
                }
                13 => {
                    Shader_ISetTexCube((*var).index, &mut **(pValue as *mut *mut TexCube));
                }
                _ => {}
            }
        }
        i += 1;
    }
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Stop(_s: *mut Shader) {
    gl::UseProgram(0);
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
pub unsafe extern "C" fn Shader_GetHandle(this: &mut Shader) -> u32 {
    this.program
}

#[no_mangle]
pub unsafe extern "C" fn Shader_GetVariable(this: &mut Shader, name: *const libc::c_char) -> i32 {
    let index: i32 = gl::GetUniformLocation(this.program, name);
    if index == -1 {
        CFatal!(
            "Shader_GetVariable: Shader <%s> has no variable <%s>",
            this.name,
            name,
        );
    }
    index
}

#[no_mangle]
pub unsafe extern "C" fn Shader_HasVariable(this: &mut Shader, name: *const libc::c_char) -> bool {
    gl::GetUniformLocation(this.program, name) > -1
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ResetTexIndex() {
    (*current).texIndex = 1;
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat(name: *const libc::c_char, value: f32) {
    gl::Uniform1f(GetUniformIndex(current.as_mut(), name), value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat(index: i32, value: f32) {
    gl::Uniform1f(index, value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat2(name: *const libc::c_char, x: f32, y: f32) {
    gl::Uniform2f(GetUniformIndex(current.as_mut(), name), x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat2(index: i32, x: f32, y: f32) {
    gl::Uniform2f(index, x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat3(name: *const libc::c_char, x: f32, y: f32, z: f32) {
    gl::Uniform3f(GetUniformIndex(current.as_mut(), name), x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat3(index: i32, x: f32, y: f32, z: f32) {
    gl::Uniform3f(index, x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat4(
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    gl::Uniform4f(GetUniformIndex(current.as_mut(), name), x, y, z, w);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat4(index: i32, x: f32, y: f32, z: f32, w: f32) {
    gl::Uniform4f(index, x, y, z, w);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetInt(name: *const libc::c_char, value: i32) {
    gl::Uniform1i(GetUniformIndex(current.as_mut(), name), value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetInt(index: i32, value: i32) {
    gl::Uniform1i(index, value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetMatrix(name: *const libc::c_char, value: &mut Matrix) {
    gl::UniformMatrix4fv(
        GetUniformIndex(current.as_mut(), name),
        1,
        gl::TRUE,
        value as *mut Matrix as *mut f32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetMatrixT(name: *const libc::c_char, value: &mut Matrix) {
    gl::UniformMatrix4fv(
        GetUniformIndex(current.as_mut(), name),
        1,
        gl::FALSE,
        value as *mut Matrix as *mut f32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetMatrix(index: i32, value: &mut Matrix) {
    gl::UniformMatrix4fv(index, 1, gl::TRUE, value as *mut Matrix as *mut f32);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetMatrixT(index: i32, value: &mut Matrix) {
    gl::UniformMatrix4fv(index, 1, gl::FALSE, value as *mut Matrix as *mut f32);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex1D(name: *const libc::c_char, value: &mut Tex1D) {
    gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    );
    let fresh14 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh14));
    gl::BindTexture(gl::TEXTURE_1D, Tex1D_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex1D(index: i32, value: &mut Tex1D) {
    gl::Uniform1i(index, (*current).texIndex as i32);
    let fresh15 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh15));
    gl::BindTexture(gl::TEXTURE_1D, Tex1D_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex2D(name: *const libc::c_char, value: &mut Tex2D) {
    gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    );
    let fresh16 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh16));
    gl::BindTexture(gl::TEXTURE_2D, Tex2D_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex2D(index: i32, value: &mut Tex2D) {
    gl::Uniform1i(index, (*current).texIndex as i32);
    let fresh17 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh17));
    gl::BindTexture(gl::TEXTURE_2D, Tex2D_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex3D(name: *const libc::c_char, value: &mut Tex3D) {
    gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    );
    let fresh18 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh18));
    gl::BindTexture(gl::TEXTURE_3D, Tex3D_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex3D(index: i32, value: &mut Tex3D) {
    gl::Uniform1i(index, (*current).texIndex as i32);
    let fresh19 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh19));
    gl::BindTexture(gl::TEXTURE_3D, Tex3D_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTexCube(name: *const libc::c_char, value: &mut TexCube) {
    gl::Uniform1i(
        GetUniformIndex(current.as_mut(), name),
        (*current).texIndex as i32,
    );
    let fresh20 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh20));
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, TexCube_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTexCube(index: i32, value: &mut TexCube) {
    gl::Uniform1i(index, (*current).texIndex as i32);
    let fresh21 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    gl::ActiveTexture((gl::TEXTURE0).wrapping_add(fresh21));
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, TexCube_GetHandle(value));
    gl::ActiveTexture(gl::TEXTURE0);
}
