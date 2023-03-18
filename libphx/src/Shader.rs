use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::Math::Vec4;
use crate::Math::{IVec2, IVec3, IVec4, Vec2};
use crate::Matrix::*;
use crate::Profiler::*;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::ShaderState::*;
use crate::ShaderVar::*;
use crate::ShaderVarType::*;
use crate::StrMap::*;
use crate::Tex1D::*;
use crate::Tex2D::*;
use crate::Tex3D::*;
use crate::TexCube::*;
use libc;

extern "C" {
    fn glBindTexture(target: GLenum, texture: GLu32);
    static mut __glewActiveTexture: PFNGLACTIVETEXTUREPROC;
    static mut __glewAttachShader: PFNGLATTACHSHADERPROC;
    static mut __glewBindAttribLocation: PFNGLBINDATTRIBLOCATIONPROC;
    static mut __glewCompileShader: PFNGLCOMPILESHADERPROC;
    static mut __glewCreateProgram: PFNGLCREATEPROGRAMPROC;
    static mut __glewCreateShader: PFNGLCREATESHADERPROC;
    static mut __glewDeleteProgram: PFNGLDELETEPROGRAMPROC;
    static mut __glewDeleteShader: PFNGLDELETESHADERPROC;
    static mut __glewGetProgramInfoLog: PFNGLGETPROGRAMINFOLOGPROC;
    static mut __glewGetProgramiv: PFNGLGETPROGRAMIVPROC;
    static mut __glewGetShaderInfoLog: PFNGLGETSHADERINFOLOGPROC;
    static mut __glewGetShaderiv: PFNGLGETSHADERIVPROC;
    static mut __glewGetUniformLocation: PFNGLGETUNIFORMLOCATIONPROC;
    static mut __glewLinkProgram: PFNGLLINKPROGRAMPROC;
    static mut __glewShaderSource: PFNGLSHADERSOURCEPROC;
    static mut __glewUniform1f: PFNGLUNIFORM1FPROC;
    static mut __glewUniform1i: PFNGLUNIFORM1IPROC;
    static mut __glewUniform2f: PFNGLUNIFORM2FPROC;
    static mut __glewUniform2i: PFNGLUNIFORM2IPROC;
    static mut __glewUniform3f: PFNGLUNIFORM3FPROC;
    static mut __glewUniform3i: PFNGLUNIFORM3IPROC;
    static mut __glewUniform4f: PFNGLUNIFORM4FPROC;
    static mut __glewUniform4i: PFNGLUNIFORM4IPROC;
    static mut __glewUniformMatrix4fv: PFNGLUNIFORMMATRIX4FVPROC;
    static mut __glewUseProgram: PFNGLUSEPROGRAMPROC;
}
pub type __builtin_va_list = *mut libc::c_char;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Shader {
    pub _refCount: u32,
    pub name: *const libc::c_char,
    pub vs: u32,
    pub fs: u32,
    pub program: u32,
    pub texIndex: u32,
    pub vars_size: i32,
    pub vars_capacity: i32,
    pub vars_data: *mut ShaderVar,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShaderVar {
    pub type_0: ShaderVarType,
    pub name: *const libc::c_char,
    pub index: i32,
}
pub type ShaderVarType = i32;
pub type ResourceType = i32;
pub type GLenum = u32;
pub type GLu32 = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type GLboolean = libc::c_uchar;
pub type GLfloat = f32;
pub type GLchar = libc::c_char;
pub type PFNGLACTIVETEXTUREPROC = Option<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLATTACHSHADERPROC = Option<unsafe extern "C" fn(GLu32, GLu32) -> ()>;
pub type PFNGLBINDATTRIBLOCATIONPROC =
    Option<unsafe extern "C" fn(GLu32, GLu32, *const GLchar) -> ()>;
pub type PFNGLCOMPILESHADERPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type PFNGLCREATEPROGRAMPROC = Option<unsafe extern "C" fn() -> GLu32>;
pub type PFNGLCREATESHADERPROC = Option<unsafe extern "C" fn(GLenum) -> GLu32>;
pub type PFNGLDELETEPROGRAMPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type PFNGLDELETESHADERPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type PFNGLGETPROGRAMINFOLOGPROC =
    Option<unsafe extern "C" fn(GLu32, GLsizei, *mut GLsizei, *mut GLchar) -> ()>;
pub type PFNGLGETPROGRAMIVPROC = Option<unsafe extern "C" fn(GLu32, GLenum, *mut GLint) -> ()>;
pub type PFNGLGETSHADERINFOLOGPROC =
    Option<unsafe extern "C" fn(GLu32, GLsizei, *mut GLsizei, *mut GLchar) -> ()>;
pub type PFNGLGETSHADERIVPROC = Option<unsafe extern "C" fn(GLu32, GLenum, *mut GLint) -> ()>;
pub type PFNGLGETUNIFORMLOCATIONPROC = Option<unsafe extern "C" fn(GLu32, *const GLchar) -> GLint>;
pub type PFNGLLINKPROGRAMPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type PFNGLSHADERSOURCEPROC =
    Option<unsafe extern "C" fn(GLu32, GLsizei, *const *const GLchar, *const GLint) -> ()>;
pub type PFNGLUNIFORM1FPROC = Option<unsafe extern "C" fn(GLint, GLfloat) -> ()>;
pub type PFNGLUNIFORM1IPROC = Option<unsafe extern "C" fn(GLint, GLint) -> ()>;
pub type PFNGLUNIFORM2FPROC = Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat) -> ()>;
pub type PFNGLUNIFORM2IPROC = Option<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>;
pub type PFNGLUNIFORM3FPROC = Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat) -> ()>;
pub type PFNGLUNIFORM3IPROC = Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> ()>;
pub type PFNGLUNIFORM4FPROC =
    Option<unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat, GLfloat) -> ()>;
pub type PFNGLUNIFORM4IPROC = Option<unsafe extern "C" fn(GLint, GLint, GLint, GLint, GLint) -> ()>;
pub type PFNGLUNIFORMMATRIX4FVPROC =
    Option<unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> ()>;
pub type PFNGLUSEPROGRAMPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type va_list = __builtin_va_list;

static mut includePath: *const libc::c_char = b"include/\0" as *const u8 as *const libc::c_char;

static mut versionString: *const libc::c_char =
    b"#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n\0"
        as *const u8 as *const libc::c_char;

static mut current: *mut Shader = std::ptr::null_mut();

static mut cache: *mut StrMap = std::ptr::null_mut();
unsafe extern "C" fn GetUniformIndex(mut this: *mut Shader, mut name: *const libc::c_char) -> i32 {
    if this.is_null() {
        Fatal(b"GetUniformIndex: No shader is bound\0" as *const u8 as *const libc::c_char);
    }
    let mut index: i32 =
        __glewGetUniformLocation.expect("non-null function pointer")((*this).program, name);
    index
}

unsafe extern "C" fn CreateGLShader(mut src: *const libc::c_char, mut type_0: GLenum) -> u32 {
    let mut this: u32 = __glewCreateShader.expect("non-null function pointer")(type_0);
    let mut srcs: [*const libc::c_char; 2] = [versionString, src];
    __glewShaderSource.expect("non-null function pointer")(
        this,
        2_i32,
        srcs.as_mut_ptr() as *const *const GLchar,
        std::ptr::null(),
    );
    __glewCompileShader.expect("non-null function pointer")(this);
    let mut status: i32 = 0;
    __glewGetShaderiv.expect("non-null function pointer")(this, 0x8b81_i32 as GLenum, &mut status);
    if status == 0_i32 {
        let mut length: i32 = 0;
        __glewGetShaderiv.expect("non-null function pointer")(
            this,
            0x8b84_i32 as GLenum,
            &mut length,
        );
        let mut infoLog: *mut libc::c_char =
            MemAllocZero((length + 1_i32) as usize) as *mut libc::c_char;
        __glewGetShaderInfoLog.expect("non-null function pointer")(
            this,
            length,
            std::ptr::null_mut(),
            infoLog,
        );
        Fatal(
            b"CreateGLShader: Failed to compile shader:\n%s\0" as *const u8 as *const libc::c_char,
            infoLog,
        );
    }
    this
}

unsafe extern "C" fn CreateGLProgram(mut vs: u32, mut fs: u32) -> u32 {
    let mut this: u32 = __glewCreateProgram.expect("non-null function pointer")();
    __glewAttachShader.expect("non-null function pointer")(this, vs);
    __glewAttachShader.expect("non-null function pointer")(this, fs);
    __glewBindAttribLocation.expect("non-null function pointer")(
        this,
        0_i32 as GLu32,
        b"vertex_position\0" as *const u8 as *const libc::c_char,
    );
    __glewBindAttribLocation.expect("non-null function pointer")(
        this,
        1_i32 as GLu32,
        b"vertex_normal\0" as *const u8 as *const libc::c_char,
    );
    __glewBindAttribLocation.expect("non-null function pointer")(
        this,
        2_i32 as GLu32,
        b"vertex_uv\0" as *const u8 as *const libc::c_char,
    );
    __glewLinkProgram.expect("non-null function pointer")(this);
    let mut status: i32 = 0;
    __glewGetProgramiv.expect("non-null function pointer")(this, 0x8b82_i32 as GLenum, &mut status);
    if status == 0_i32 {
        let mut length: i32 = 0;
        __glewGetProgramiv.expect("non-null function pointer")(
            this,
            0x8b84_i32 as GLenum,
            &mut length,
        );
        let mut infoLog: *mut libc::c_char =
            MemAllocZero((length + 1_i32) as usize) as *mut libc::c_char;
        __glewGetProgramInfoLog.expect("non-null function pointer")(
            this,
            length,
            std::ptr::null_mut(),
            infoLog,
        );
        Fatal(
            b"CreateGLProgram: Failed to link program:\n%s\0" as *const u8 as *const libc::c_char,
            infoLog,
        );
    }
    this
}

unsafe extern "C" fn GLSL_Load(
    mut name: *const libc::c_char,
    mut this: *mut Shader,
) -> *const libc::c_char {
    if cache.is_null() {
        cache = StrMap_Create(16_i32 as u32);
    }
    let mut cached: *mut libc::c_void = StrMap_Get(cache, name);
    if !cached.is_null() {
        return cached as *const libc::c_char;
    }
    let mut rawCode: *const libc::c_char = Resource_LoadCstr(ResourceType_Shader, name);
    let mut code: *const libc::c_char = StrReplace(
        rawCode,
        b"\r\n\0" as *const u8 as *const libc::c_char,
        b"\n\0" as *const u8 as *const libc::c_char,
    );
    StrFree(rawCode);
    code = GLSL_Preprocess(code, this);
    code
}

unsafe extern "C" fn GLSL_Preprocess(
    mut code: *const libc::c_char,
    mut this: *mut Shader,
) -> *const libc::c_char {
    let lenInclude: i32 = StrLen(b"#include\0" as *const u8 as *const libc::c_char) as i32;
    let mut begin: *const libc::c_char = std::ptr::null();
    loop {
        begin = StrFind(code, b"#include\0" as *const u8 as *const libc::c_char);
        if begin.is_null() {
            break;
        }
        let mut end: *const libc::c_char =
            StrFind(begin, b"\n\0" as *const u8 as *const libc::c_char);
        let mut name: *const libc::c_char =
            StrSubStr(begin.offset(lenInclude as isize).offset(1), end);
        let mut path: *const libc::c_char = StrAdd(includePath, name);
        let mut prev: *const libc::c_char = code;
        code = StrSub(code, begin, end, GLSL_Load(path, this));
        StrFree(prev);
        StrFree(path);
        StrFree(name);
    }
    loop {
        begin = StrFind(code, b"#autovar\0" as *const u8 as *const libc::c_char);
        if begin.is_null() {
            break;
        }
        let mut end_0: *const libc::c_char =
            StrFind(begin, b"\n\0" as *const u8 as *const libc::c_char);
        let mut line: *const libc::c_char = StrSubStr(begin, end_0);
        let mut varType: [libc::c_char; 32] = [
            0_i32 as libc::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut varName: [libc::c_char; 32] = [
            0_i32 as libc::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        if libc::sscanf(
            line,
            b"#autovar %31s %31s\0" as *const u8 as *const libc::c_char,
            varType.as_mut_ptr(),
            varName.as_mut_ptr(),
        ) == 2_i32
        {
            let mut var: ShaderVar = ShaderVar {
                type_0: 0_i32,
                name: std::ptr::null(),
                index: 0,
            };
            var.type_0 = ShaderVarType_FromStr(varType.as_mut_ptr() as *const libc::c_char);
            if var.type_0 == 0_i32 {
                Fatal(
                    b"GLSL_Preprocess: Unknown shader variable type <%s> in directive:\n  %s\0"
                        as *const u8 as *const libc::c_char,
                    varType.as_mut_ptr(),
                    line,
                );
            }
            var.name = StrDup(varName.as_mut_ptr() as *const libc::c_char);
            var.index = -1_i32;
            if ((*this).vars_capacity == (*this).vars_size) as i32 as libc::c_long != 0 {
                (*this).vars_capacity = if (*this).vars_capacity != 0 {
                    (*this).vars_capacity * 2_i32
                } else {
                    1_i32
                };
                let mut elemSize: usize = ::core::mem::size_of::<ShaderVar>();
                let mut pData: *mut *mut libc::c_void =
                    &mut (*this).vars_data as *mut *mut ShaderVar as *mut *mut libc::c_void;
                *pData = MemRealloc(
                    (*this).vars_data as *mut libc::c_void,
                    ((*this).vars_capacity as usize).wrapping_mul(elemSize),
                );
            }
            let fresh13 = (*this).vars_size;
            (*this).vars_size += 1;
            *((*this).vars_data).offset(fresh13 as isize) = var;
        } else {
            Fatal(
                b"GLSL_Preprocess: Failed to parse directive:\n  %s\0" as *const u8
                    as *const libc::c_char,
                line,
            );
        }
        let mut prev_0: *const libc::c_char = code;
        code = StrSub(
            code,
            begin,
            end_0,
            b"\0" as *const u8 as *const libc::c_char,
        );
        StrFree(prev_0);
        StrFree(line);
    }
    code
}

unsafe extern "C" fn Shader_BindVariables(mut this: *mut Shader) {
    let mut i: i32 = 0_i32;
    while i < (*this).vars_size {
        let mut var: *mut ShaderVar = ((*this).vars_data).offset(i as isize);
        (*var).index = __glewGetUniformLocation.expect("non-null function pointer")(
            (*this).program,
            (*var).name,
        );
        if (*var).index < 0_i32 {
            Warn(
                b"Shader_BindVariables: Automatic shader variable <%s> does not exist in shader <%s>\0"
                    as *const u8 as *const libc::c_char,
                (*var).name,
                (*this).name,
            );
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Create(
    mut vs: *const libc::c_char,
    mut fs: *const libc::c_char,
) -> *mut Shader {
    let mut this: *mut Shader = MemAlloc(::core::mem::size_of::<Shader>()) as *mut Shader;
    (*this)._refCount = 1_i32 as u32;
    (*this).vars_capacity = 0_i32;
    (*this).vars_size = 0_i32;
    (*this).vars_data = std::ptr::null_mut();
    vs = GLSL_Preprocess(
        StrReplace(
            vs,
            b"\r\n\0" as *const u8 as *const libc::c_char,
            b"\n\0" as *const u8 as *const libc::c_char,
        ),
        this,
    );
    fs = GLSL_Preprocess(
        StrReplace(
            fs,
            b"\r\n\0" as *const u8 as *const libc::c_char,
            b"\n\0" as *const u8 as *const libc::c_char,
        ),
        this,
    );
    (*this).vs = CreateGLShader(vs, 0x8b31_i32 as GLenum);
    (*this).fs = CreateGLShader(fs, 0x8b30_i32 as GLenum);
    (*this).program = CreateGLProgram((*this).vs, (*this).fs);
    (*this).texIndex = 1_i32 as u32;
    (*this).name = StrFormat(
        b"[anonymous shader @ %p]\0" as *const u8 as *const libc::c_char,
        this,
    );
    StrFree(vs);
    StrFree(fs);
    Shader_BindVariables(this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Load(
    mut vName: *const libc::c_char,
    mut fName: *const libc::c_char,
) -> *mut Shader {
    let mut this: *mut Shader = MemAlloc(::core::mem::size_of::<Shader>()) as *mut Shader;
    (*this)._refCount = 1_i32 as u32;
    (*this).vars_capacity = 0_i32;
    (*this).vars_size = 0_i32;
    (*this).vars_data = std::ptr::null_mut();
    let mut vs: *const libc::c_char = GLSL_Load(vName, this);
    let mut fs: *const libc::c_char = GLSL_Load(fName, this);
    (*this).vs = CreateGLShader(vs, 0x8b31_i32 as GLenum);
    (*this).fs = CreateGLShader(fs, 0x8b30_i32 as GLenum);
    (*this).program = CreateGLProgram((*this).vs, (*this).fs);
    (*this).texIndex = 1_i32 as u32;
    (*this).name = StrFormat(
        b"[vs: %s , fs: %s]\0" as *const u8 as *const libc::c_char,
        vName,
        fName,
    );
    Shader_BindVariables(this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Acquire(mut this: *mut Shader) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Free(mut this: *mut Shader) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        __glewDeleteShader.expect("non-null function pointer")((*this).vs);
        __glewDeleteShader.expect("non-null function pointer")((*this).fs);
        __glewDeleteProgram.expect("non-null function pointer")((*this).program);
        MemFree((*this).vars_data as *const libc::c_void);
        StrFree((*this).name);
        MemFree(this as *const libc::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ToShaderState(mut this: *mut Shader) -> *mut ShaderState {
    ShaderState_Create(this)
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Start(mut this: *mut Shader) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Shader_Start\0")).as_ptr(),
    );
    __glewUseProgram.expect("non-null function pointer")((*this).program);
    current = this;
    (*this).texIndex = 1_i32 as u32;
    let mut i: i32 = 0_i32;
    while i < (*this).vars_size {
        let mut var: *mut ShaderVar = ((*this).vars_data).offset(i as isize);
        if !((*var).index < 0_i32) {
            let mut pValue: *mut libc::c_void = ShaderVar_Get((*var).name, (*var).type_0);
            if pValue.is_null() {
                Fatal(
                    b"Shader_Start: Shader variable stack does not contain variable <%s>\0"
                        as *const u8 as *const libc::c_char,
                    (*var).name,
                );
            }
            match (*var).type_0 {
                1 => {
                    let mut value: f32 = *(pValue as *mut f32);
                    __glewUniform1f.expect("non-null function pointer")((*var).index, value);
                }
                2 => {
                    let mut value_0 = *(pValue as *mut Vec2);
                    __glewUniform2f.expect("non-null function pointer")(
                        (*var).index,
                        value_0.x,
                        value_0.y,
                    );
                }
                3 => {
                    let mut value_1: Vec3 = *(pValue as *mut Vec3);
                    __glewUniform3f.expect("non-null function pointer")(
                        (*var).index,
                        value_1.x,
                        value_1.y,
                        value_1.z,
                    );
                }
                4 => {
                    let mut value_2: Vec4 = *(pValue as *mut Vec4);
                    __glewUniform4f.expect("non-null function pointer")(
                        (*var).index,
                        value_2.x,
                        value_2.y,
                        value_2.z,
                        value_2.w,
                    );
                }
                5 => {
                    let mut value_3: i32 = *(pValue as *mut i32);
                    __glewUniform1i.expect("non-null function pointer")((*var).index, value_3);
                }
                6 => {
                    let mut value_4: IVec2 = *(pValue as *mut IVec2);
                    __glewUniform2i.expect("non-null function pointer")(
                        (*var).index,
                        value_4.x,
                        value_4.y,
                    );
                }
                7 => {
                    let mut value_5: IVec3 = *(pValue as *mut IVec3);
                    __glewUniform3i.expect("non-null function pointer")(
                        (*var).index,
                        value_5.x,
                        value_5.y,
                        value_5.z,
                    );
                }
                8 => {
                    let mut value_6: IVec4 = *(pValue as *mut IVec4);
                    __glewUniform4i.expect("non-null function pointer")(
                        (*var).index,
                        value_6.x,
                        value_6.y,
                        value_6.z,
                        value_6.w,
                    );
                }
                9 => {
                    Shader_ISetMatrix((*var).index, *(pValue as *mut *mut Matrix));
                }
                10 => {
                    Shader_ISetTex1D((*var).index, *(pValue as *mut *mut Tex1D));
                }
                11 => {
                    Shader_ISetTex2D((*var).index, *(pValue as *mut *mut Tex2D));
                }
                12 => {
                    Shader_ISetTex3D((*var).index, *(pValue as *mut *mut Tex3D));
                }
                13 => {
                    Shader_ISetTexCube((*var).index, *(pValue as *mut *mut TexCube));
                }
                _ => {}
            }
        }
        i += 1;
    }
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Shader_Stop(mut _s: *mut Shader) {
    __glewUseProgram.expect("non-null function pointer")(0_i32 as GLu32);
    current = std::ptr::null_mut();
}

unsafe extern "C" fn ShaderCache_FreeElem(
    mut _s: *const libc::c_char,
    mut data: *mut libc::c_void,
) {
    MemFree(data);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ClearCache() {
    if !cache.is_null() {
        StrMap_FreeEx(
            cache,
            Some(
                ShaderCache_FreeElem
                    as unsafe extern "C" fn(*const libc::c_char, *mut libc::c_void) -> (),
            ),
        );
        cache = std::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Shader_GetHandle(mut this: *mut Shader) -> u32 {
    (*this).program
}

#[no_mangle]
pub unsafe extern "C" fn Shader_GetVariable(
    mut this: *mut Shader,
    mut name: *const libc::c_char,
) -> i32 {
    let mut index: i32 =
        __glewGetUniformLocation.expect("non-null function pointer")((*this).program, name);
    if index == -1_i32 {
        Fatal(
            b"Shader_GetVariable: Shader <%s> has no variable <%s>\0" as *const u8
                as *const libc::c_char,
            (*this).name,
            name,
        );
    }
    index
}

#[no_mangle]
pub unsafe extern "C" fn Shader_HasVariable(
    mut this: *mut Shader,
    mut name: *const libc::c_char,
) -> bool {
    __glewGetUniformLocation.expect("non-null function pointer")((*this).program, name) > -1_i32
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ResetTexIndex() {
    (*current).texIndex = 1_i32 as u32;
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat(mut name: *const libc::c_char, mut value: f32) {
    __glewUniform1f.expect("non-null function pointer")(GetUniformIndex(current, name), value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat(mut index: i32, mut value: f32) {
    __glewUniform1f.expect("non-null function pointer")(index, value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat2(mut name: *const libc::c_char, mut x: f32, mut y: f32) {
    __glewUniform2f.expect("non-null function pointer")(GetUniformIndex(current, name), x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat2(mut index: i32, mut x: f32, mut y: f32) {
    __glewUniform2f.expect("non-null function pointer")(index, x, y);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat3(
    mut name: *const libc::c_char,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) {
    __glewUniform3f.expect("non-null function pointer")(GetUniformIndex(current, name), x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat3(mut index: i32, mut x: f32, mut y: f32, mut z: f32) {
    __glewUniform3f.expect("non-null function pointer")(index, x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat4(
    mut name: *const libc::c_char,
    mut x: f32,
    mut y: f32,
    mut z: f32,
    mut w: f32,
) {
    __glewUniform4f.expect("non-null function pointer")(GetUniformIndex(current, name), x, y, z, w);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat4(
    mut index: i32,
    mut x: f32,
    mut y: f32,
    mut z: f32,
    mut w: f32,
) {
    __glewUniform4f.expect("non-null function pointer")(index, x, y, z, w);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetInt(mut name: *const libc::c_char, mut value: i32) {
    __glewUniform1i.expect("non-null function pointer")(GetUniformIndex(current, name), value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetInt(mut index: i32, mut value: i32) {
    __glewUniform1i.expect("non-null function pointer")(index, value);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetMatrix(mut name: *const libc::c_char, mut value: *mut Matrix) {
    __glewUniformMatrix4fv.expect("non-null function pointer")(
        GetUniformIndex(current, name),
        1_i32,
        1_i32 as GLboolean,
        value as *mut f32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetMatrixT(mut name: *const libc::c_char, mut value: *mut Matrix) {
    __glewUniformMatrix4fv.expect("non-null function pointer")(
        GetUniformIndex(current, name),
        1_i32,
        0_i32 as GLboolean,
        value as *mut f32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetMatrix(mut index: i32, mut value: *mut Matrix) {
    __glewUniformMatrix4fv.expect("non-null function pointer")(
        index,
        1_i32,
        1_i32 as GLboolean,
        value as *mut f32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetMatrixT(mut index: i32, mut value: *mut Matrix) {
    __glewUniformMatrix4fv.expect("non-null function pointer")(
        index,
        1_i32,
        0_i32 as GLboolean,
        value as *mut f32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex1D(mut name: *const libc::c_char, mut value: *mut Tex1D) {
    __glewUniform1i.expect("non-null function pointer")(
        GetUniformIndex(current, name),
        (*current).texIndex as GLint,
    );
    let fresh14 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh14),
    );
    glBindTexture(0xde0_i32 as GLenum, Tex1D_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex1D(mut index: i32, mut value: *mut Tex1D) {
    __glewUniform1i.expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh15 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh15),
    );
    glBindTexture(0xde0_i32 as GLenum, Tex1D_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex2D(mut name: *const libc::c_char, mut value: *mut Tex2D) {
    __glewUniform1i.expect("non-null function pointer")(
        GetUniformIndex(current, name),
        (*current).texIndex as GLint,
    );
    let fresh16 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh16),
    );
    glBindTexture(0xde1_i32 as GLenum, Tex2D_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex2D(mut index: i32, mut value: *mut Tex2D) {
    __glewUniform1i.expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh17 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh17),
    );
    glBindTexture(0xde1_i32 as GLenum, Tex2D_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex3D(mut name: *const libc::c_char, mut value: *mut Tex3D) {
    __glewUniform1i.expect("non-null function pointer")(
        GetUniformIndex(current, name),
        (*current).texIndex as GLint,
    );
    let fresh18 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh18),
    );
    glBindTexture(0x806f_i32 as GLenum, Tex3D_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex3D(mut index: i32, mut value: *mut Tex3D) {
    __glewUniform1i.expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh19 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh19),
    );
    glBindTexture(0x806f_i32 as GLenum, Tex3D_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_SetTexCube(mut name: *const libc::c_char, mut value: *mut TexCube) {
    __glewUniform1i.expect("non-null function pointer")(
        GetUniformIndex(current, name),
        (*current).texIndex as GLint,
    );
    let fresh20 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh20),
    );
    glBindTexture(0x8513_i32 as GLenum, TexCube_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTexCube(mut index: i32, mut value: *mut TexCube) {
    __glewUniform1i.expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh21 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture.expect("non-null function pointer")(
        (0x84c0_i32 as u32).wrapping_add(fresh21),
    );
    glBindTexture(0x8513_i32 as GLenum, TexCube_GetHandle(value));
    __glewActiveTexture.expect("non-null function pointer")(0x84c0_i32 as GLenum);
}
