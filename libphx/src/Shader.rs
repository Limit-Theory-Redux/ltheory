use ::libc;
use glam::Vec3;
use glam::{IVec2, IVec3, IVec4, Vec2};
use crate::internal::Memory::*;
use crate::ResourceType::*;

extern "C" {
    pub type ShaderState;
    pub type StrMap;
    pub type Tex1D;
    pub type Tex2D;
    pub type Tex3D;
    pub type TexCube;
    pub type Matrix;
    fn Fatal(_: cstr, _: ...);
    fn Warn(_: cstr, _: ...);
    fn glBindTexture(target: GLenum, texture: GLuint);
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
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn Resource_LoadCstr(_: ResourceType, name: cstr) -> cstr;
    fn ShaderState_Create(_: *mut Shader) -> *mut ShaderState;
    fn ShaderVar_Get(_: cstr, _: ShaderVarType) -> *mut libc::c_void;
    fn ShaderVarType_FromStr(_: cstr) -> ShaderVarType;
    fn StrMap_Create(initCapacity: uint32) -> *mut StrMap;
    fn StrMap_FreeEx(
        _: *mut StrMap,
        freeFn: Option::<unsafe extern "C" fn(cstr, *mut libc::c_void) -> ()>,
    );
    fn StrMap_Get(_: *mut StrMap, key: cstr) -> *mut libc::c_void;
    fn vsnprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: __builtin_va_list,
    ) -> libc::c_int;
    fn sscanf(_: *const libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn Tex1D_GetHandle(_: *mut Tex1D) -> uint;
    fn Tex2D_GetHandle(_: *mut Tex2D) -> uint;
    fn Tex3D_GetHandle(_: *mut Tex3D) -> uint;
    fn TexCube_GetHandle(_: *mut TexCube) -> uint;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Shader {
    pub _refCount: uint32,
    pub name: cstr,
    pub vs: uint,
    pub fs: uint,
    pub program: uint,
    pub texIndex: uint,
    pub vars_size: int32,
    pub vars_capacity: int32,
    pub vars_data: *mut ShaderVar,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShaderVar {
    pub type_0: ShaderVarType,
    pub name: cstr,
    pub index: libc::c_int,
}
pub type ShaderVarType = int32;




#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type ResourceType = int32;
pub type GLenum = libc::c_uint;
pub type GLuint = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type GLboolean = libc::c_uchar;
pub type GLfloat = libc::c_float;
pub type GLchar = libc::c_char;
pub type PFNGLACTIVETEXTUREPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
pub type PFNGLATTACHSHADERPROC = Option::<unsafe extern "C" fn(GLuint, GLuint) -> ()>;
pub type PFNGLBINDATTRIBLOCATIONPROC = Option::<
    unsafe extern "C" fn(GLuint, GLuint, *const GLchar) -> (),
>;
pub type PFNGLCOMPILESHADERPROC = Option::<unsafe extern "C" fn(GLuint) -> ()>;
pub type PFNGLCREATEPROGRAMPROC = Option::<unsafe extern "C" fn() -> GLuint>;
pub type PFNGLCREATESHADERPROC = Option::<unsafe extern "C" fn(GLenum) -> GLuint>;
pub type PFNGLDELETEPROGRAMPROC = Option::<unsafe extern "C" fn(GLuint) -> ()>;
pub type PFNGLDELETESHADERPROC = Option::<unsafe extern "C" fn(GLuint) -> ()>;
pub type PFNGLGETPROGRAMINFOLOGPROC = Option::<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
>;
pub type PFNGLGETPROGRAMIVPROC = Option::<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
>;
pub type PFNGLGETSHADERINFOLOGPROC = Option::<
    unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar) -> (),
>;
pub type PFNGLGETSHADERIVPROC = Option::<
    unsafe extern "C" fn(GLuint, GLenum, *mut GLint) -> (),
>;
pub type PFNGLGETUNIFORMLOCATIONPROC = Option::<
    unsafe extern "C" fn(GLuint, *const GLchar) -> GLint,
>;
pub type PFNGLLINKPROGRAMPROC = Option::<unsafe extern "C" fn(GLuint) -> ()>;
pub type PFNGLSHADERSOURCEPROC = Option::<
    unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *const GLint) -> (),
>;
pub type PFNGLUNIFORM1FPROC = Option::<unsafe extern "C" fn(GLint, GLfloat) -> ()>;
pub type PFNGLUNIFORM1IPROC = Option::<unsafe extern "C" fn(GLint, GLint) -> ()>;
pub type PFNGLUNIFORM2FPROC = Option::<
    unsafe extern "C" fn(GLint, GLfloat, GLfloat) -> (),
>;
pub type PFNGLUNIFORM2IPROC = Option::<unsafe extern "C" fn(GLint, GLint, GLint) -> ()>;
pub type PFNGLUNIFORM3FPROC = Option::<
    unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat) -> (),
>;
pub type PFNGLUNIFORM3IPROC = Option::<
    unsafe extern "C" fn(GLint, GLint, GLint, GLint) -> (),
>;
pub type PFNGLUNIFORM4FPROC = Option::<
    unsafe extern "C" fn(GLint, GLfloat, GLfloat, GLfloat, GLfloat) -> (),
>;
pub type PFNGLUNIFORM4IPROC = Option::<
    unsafe extern "C" fn(GLint, GLint, GLint, GLint, GLint) -> (),
>;
pub type PFNGLUNIFORMMATRIX4FVPROC = Option::<
    unsafe extern "C" fn(GLint, GLsizei, GLboolean, *const GLfloat) -> (),
>;
pub type PFNGLUSEPROGRAMPROC = Option::<unsafe extern "C" fn(GLuint) -> ()>;
pub type va_list = __builtin_va_list;

static mut includePath: cstr = b"include/\0" as *const u8 as *const libc::c_char;
static mut versionString: cstr = b"#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n\0"
    as *const u8 as *const libc::c_char;
static mut current: *mut Shader = 0 as *const Shader as *mut Shader;
static mut cache: *mut StrMap = 0 as *const StrMap as *mut StrMap;
unsafe extern "C" fn GetUniformIndex(
    mut this: *mut Shader,
    mut name: cstr,
) -> libc::c_int {
    if this.is_null() {
        Fatal(
            b"GetUniformIndex: No shader is bound\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut index: libc::c_int = __glewGetUniformLocation
        .expect("non-null function pointer")((*this).program, name);
    return index;
}
unsafe extern "C" fn CreateGLShader(mut src: cstr, mut type_0: GLenum) -> uint {
    let mut this: uint = __glewCreateShader
        .expect("non-null function pointer")(type_0);
    let mut srcs: [cstr; 2] = [versionString, src];
    __glewShaderSource
        .expect(
            "non-null function pointer",
        )(
        this,
        2 as libc::c_int,
        srcs.as_mut_ptr() as *const *const GLchar,
        0 as *const GLint,
    );
    __glewCompileShader.expect("non-null function pointer")(this);
    let mut status: libc::c_int = 0;
    __glewGetShaderiv
        .expect(
            "non-null function pointer",
        )(this, 0x8b81 as libc::c_int as GLenum, &mut status);
    if status == 0 as libc::c_int {
        let mut length: libc::c_int = 0;
        __glewGetShaderiv
            .expect(
                "non-null function pointer",
            )(this, 0x8b84 as libc::c_int as GLenum, &mut length);
        let mut infoLog: *mut libc::c_char = MemAllocZero(
            (length + 1 as libc::c_int) as libc::size_t,
        ) as *mut libc::c_char;
        __glewGetShaderInfoLog
            .expect(
                "non-null function pointer",
            )(this, length, 0 as *mut GLsizei, infoLog);
        Fatal(
            b"CreateGLShader: Failed to compile shader:\n%s\0" as *const u8
                as *const libc::c_char,
            infoLog,
        );
    }
    return this;
}
unsafe extern "C" fn CreateGLProgram(mut vs: uint, mut fs: uint) -> uint {
    let mut this: uint = __glewCreateProgram.expect("non-null function pointer")();
    __glewAttachShader.expect("non-null function pointer")(this, vs);
    __glewAttachShader.expect("non-null function pointer")(this, fs);
    __glewBindAttribLocation
        .expect(
            "non-null function pointer",
        )(
        this,
        0 as libc::c_int as GLuint,
        b"vertex_position\0" as *const u8 as *const libc::c_char,
    );
    __glewBindAttribLocation
        .expect(
            "non-null function pointer",
        )(
        this,
        1 as libc::c_int as GLuint,
        b"vertex_normal\0" as *const u8 as *const libc::c_char,
    );
    __glewBindAttribLocation
        .expect(
            "non-null function pointer",
        )(
        this,
        2 as libc::c_int as GLuint,
        b"vertex_uv\0" as *const u8 as *const libc::c_char,
    );
    __glewLinkProgram.expect("non-null function pointer")(this);
    let mut status: libc::c_int = 0;
    __glewGetProgramiv
        .expect(
            "non-null function pointer",
        )(this, 0x8b82 as libc::c_int as GLenum, &mut status);
    if status == 0 as libc::c_int {
        let mut length: libc::c_int = 0;
        __glewGetProgramiv
            .expect(
                "non-null function pointer",
            )(this, 0x8b84 as libc::c_int as GLenum, &mut length);
        let mut infoLog: *mut libc::c_char = MemAllocZero(
            (length + 1 as libc::c_int) as libc::size_t,
        ) as *mut libc::c_char;
        __glewGetProgramInfoLog
            .expect(
                "non-null function pointer",
            )(this, length, 0 as *mut GLsizei, infoLog);
        Fatal(
            b"CreateGLProgram: Failed to link program:\n%s\0" as *const u8
                as *const libc::c_char,
            infoLog,
        );
    }
    return this;
}
unsafe extern "C" fn GLSL_Load(mut name: cstr, mut this: *mut Shader) -> cstr {
    if cache.is_null() {
        cache = StrMap_Create(16 as libc::c_int as uint32);
    }
    let mut cached: *mut libc::c_void = StrMap_Get(cache, name);
    if !cached.is_null() {
        return cached as cstr;
    }
    let mut rawCode: cstr = Resource_LoadCstr(ResourceType_Shader, name);
    let mut code: cstr = StrReplace(
        rawCode,
        b"\r\n\0" as *const u8 as *const libc::c_char,
        b"\n\0" as *const u8 as *const libc::c_char,
    );
    StrFree(rawCode);
    code = GLSL_Preprocess(code, this);
    return code;
}
unsafe extern "C" fn GLSL_Preprocess(mut code: cstr, mut this: *mut Shader) -> cstr {
    let lenInclude: libc::c_int = StrLen(
        b"#include\0" as *const u8 as *const libc::c_char,
    ) as libc::c_int;
    let mut begin: cstr = 0 as *const libc::c_char;
    loop {
        begin = StrFind(code, b"#include\0" as *const u8 as *const libc::c_char);
        if begin.is_null() {
            break;
        }
        let mut end: cstr = StrFind(begin, b"\n\0" as *const u8 as *const libc::c_char);
        let mut name: cstr = StrSubStr(
            begin.offset(lenInclude as isize).offset(1),
            end,
        );
        let mut path: cstr = StrAdd(includePath, name);
        let mut prev: cstr = code;
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
        let mut end_0: cstr = StrFind(
            begin,
            b"\n\0" as *const u8 as *const libc::c_char,
        );
        let mut line: cstr = StrSubStr(begin, end_0);
        let mut varType: [libc::c_char; 32] = [
            0 as libc::c_int as libc::c_char,
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
            0 as libc::c_int as libc::c_char,
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
        if sscanf(
            line,
            b"#autovar %31s %31s\0" as *const u8 as *const libc::c_char,
            varType.as_mut_ptr(),
            varName.as_mut_ptr(),
        ) == 2 as libc::c_int
        {
            let mut var: ShaderVar = {
                let mut init = ShaderVar {
                    type_0: 0 as libc::c_int,
                    name: 0 as *const libc::c_char,
                    index: 0,
                };
                init
            };
            var.type_0 = ShaderVarType_FromStr(varType.as_mut_ptr() as cstr);
            if var.type_0 == 0 as libc::c_int {
                Fatal(
                    b"GLSL_Preprocess: Unknown shader variable type <%s> in directive:\n  %s\0"
                        as *const u8 as *const libc::c_char,
                    varType.as_mut_ptr(),
                    line,
                );
            }
            var.name = StrDup(varName.as_mut_ptr() as cstr);
            var.index = -(1 as libc::c_int);
            if ((*this).vars_capacity == (*this).vars_size) as libc::c_int
                as libc::c_long != 0
            {
                (*this)
                    .vars_capacity = if (*this).vars_capacity != 0 {
                    (*this).vars_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize: usize = ::core::mem::size_of::<ShaderVar>();
                let mut pData: *mut *mut libc::c_void = &mut (*this).vars_data
                    as *mut *mut ShaderVar as *mut *mut libc::c_void;
                *pData = MemRealloc(
                    (*this).vars_data as *mut libc::c_void,
                    ((*this).vars_capacity as usize).wrapping_mul(elemSize as usize),
                );
            }
            let fresh13 = (*this).vars_size;
            (*this).vars_size = (*this).vars_size + 1;
            *((*this).vars_data).offset(fresh13 as isize) = var;
        } else {
            Fatal(
                b"GLSL_Preprocess: Failed to parse directive:\n  %s\0" as *const u8
                    as *const libc::c_char,
                line,
            );
        }
        let mut prev_0: cstr = code;
        code = StrSub(code, begin, end_0, b"\0" as *const u8 as *const libc::c_char);
        StrFree(prev_0);
        StrFree(line);
    }
    return code;
}
unsafe extern "C" fn Shader_BindVariables(mut this: *mut Shader) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*this).vars_size {
        let mut var: *mut ShaderVar = ((*this).vars_data).offset(i as isize);
        (*var)
            .index = __glewGetUniformLocation
            .expect("non-null function pointer")((*this).program, (*var).name);
        if (*var).index < 0 as libc::c_int {
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
pub unsafe extern "C" fn Shader_Create(mut vs: cstr, mut fs: cstr) -> *mut Shader {
    let mut this: *mut Shader = MemAlloc(
        ::core::mem::size_of::<Shader>() as usize,
    ) as *mut Shader;
    (*this)._refCount = 1 as libc::c_int as uint32;
    (*this).vars_capacity = 0 as libc::c_int;
    (*this).vars_size = 0 as libc::c_int;
    (*this).vars_data = 0 as *mut ShaderVar;
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
    (*this).vs = CreateGLShader(vs, 0x8b31 as libc::c_int as GLenum);
    (*this).fs = CreateGLShader(fs, 0x8b30 as libc::c_int as GLenum);
    (*this).program = CreateGLProgram((*this).vs, (*this).fs);
    (*this).texIndex = 1 as libc::c_int as uint;
    (*this)
        .name = StrFormat(
        b"[anonymous shader @ %p]\0" as *const u8 as *const libc::c_char,
        this,
    );
    StrFree(vs);
    StrFree(fs);
    Shader_BindVariables(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Shader_Load(mut vName: cstr, mut fName: cstr) -> *mut Shader {
    let mut this: *mut Shader = MemAlloc(
        ::core::mem::size_of::<Shader>() as usize,
    ) as *mut Shader;
    (*this)._refCount = 1 as libc::c_int as uint32;
    (*this).vars_capacity = 0 as libc::c_int;
    (*this).vars_size = 0 as libc::c_int;
    (*this).vars_data = 0 as *mut ShaderVar;
    let mut vs: cstr = GLSL_Load(vName, this);
    let mut fs: cstr = GLSL_Load(fName, this);
    (*this).vs = CreateGLShader(vs, 0x8b31 as libc::c_int as GLenum);
    (*this).fs = CreateGLShader(fs, 0x8b30 as libc::c_int as GLenum);
    (*this).program = CreateGLProgram((*this).vs, (*this).fs);
    (*this).texIndex = 1 as libc::c_int as uint;
    (*this)
        .name = StrFormat(
        b"[vs: %s , fs: %s]\0" as *const u8 as *const libc::c_char,
        vName,
        fName,
    );
    Shader_BindVariables(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Shader_Acquire(mut this: *mut Shader) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_Free(mut this: *mut Shader) {
    if !this.is_null()
        && {
            (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
            (*this)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        __glewDeleteShader.expect("non-null function pointer")((*this).vs);
        __glewDeleteShader.expect("non-null function pointer")((*this).fs);
        __glewDeleteProgram.expect("non-null function pointer")((*this).program);
        MemFree((*this).vars_data as *const libc::c_void);
        StrFree((*this).name);
        MemFree(this as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ToShaderState(
    mut this: *mut Shader,
) -> *mut ShaderState {
    return ShaderState_Create(this);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_Start(mut this: *mut Shader) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Shader_Start\0"))
            .as_ptr(),
    );
    __glewUseProgram.expect("non-null function pointer")((*this).program);
    current = this;
    (*this).texIndex = 1 as libc::c_int as uint;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*this).vars_size {
        let mut var: *mut ShaderVar = ((*this).vars_data).offset(i as isize);
        if !((*var).index < 0 as libc::c_int) {
            let mut pValue: *mut libc::c_void = ShaderVar_Get(
                (*var).name,
                (*var).type_0,
            );
            if pValue.is_null() {
                Fatal(
                    b"Shader_Start: Shader variable stack does not contain variable <%s>\0"
                        as *const u8 as *const libc::c_char,
                    (*var).name,
                );
            }
            match (*var).type_0 {
                1 => {
                    let mut value: libc::c_float = *(pValue as *mut libc::c_float);
                    __glewUniform1f
                        .expect("non-null function pointer")((*var).index, value);
                }
                2 => {
                    let mut value_0 = *(pValue as *mut Vec2);
                    __glewUniform2f
                        .expect(
                            "non-null function pointer",
                        )((*var).index, value_0.x, value_0.y);
                }
                3 => {
                    let mut value_1: Vec3 = *(pValue as *mut Vec3);
                    __glewUniform3f
                        .expect(
                            "non-null function pointer",
                        )((*var).index, value_1.x, value_1.y, value_1.z);
                }
                4 => {
                    let mut value_2: Vec4f = *(pValue as *mut Vec4f);
                    __glewUniform4f
                        .expect(
                            "non-null function pointer",
                        )((*var).index, value_2.x, value_2.y, value_2.z, value_2.w);
                }
                5 => {
                    let mut value_3: libc::c_int = *(pValue as *mut libc::c_int);
                    __glewUniform1i
                        .expect("non-null function pointer")((*var).index, value_3);
                }
                6 => {
                    let mut value_4: IVec2 = *(pValue as *mut IVec2);
                    __glewUniform2i
                        .expect(
                            "non-null function pointer",
                        )((*var).index, value_4.x, value_4.y);
                }
                7 => {
                    let mut value_5: IVec3 = *(pValue as *mut IVec3);
                    __glewUniform3i
                        .expect(
                            "non-null function pointer",
                        )((*var).index, value_5.x, value_5.y, value_5.z);
                }
                8 => {
                    let mut value_6: IVec4 = *(pValue as *mut IVec4);
                    __glewUniform4i
                        .expect(
                            "non-null function pointer",
                        )((*var).index, value_6.x, value_6.y, value_6.z, value_6.w);
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
pub unsafe extern "C" fn Shader_Stop(mut s: *mut Shader) {
    __glewUseProgram.expect("non-null function pointer")(0 as libc::c_int as GLuint);
    current = 0 as *mut Shader;
}
unsafe extern "C" fn ShaderCache_FreeElem(mut s: cstr, mut data: *mut libc::c_void) {
    MemFree(data);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ClearCache() {
    if !cache.is_null() {
        StrMap_FreeEx(
            cache,
            Some(
                ShaderCache_FreeElem
                    as unsafe extern "C" fn(cstr, *mut libc::c_void) -> (),
            ),
        );
        cache = 0 as *mut StrMap;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shader_GetHandle(mut this: *mut Shader) -> uint {
    return (*this).program;
}
#[no_mangle]
pub unsafe extern "C" fn Shader_GetVariable(
    mut this: *mut Shader,
    mut name: cstr,
) -> libc::c_int {
    let mut index: libc::c_int = __glewGetUniformLocation
        .expect("non-null function pointer")((*this).program, name);
    if index == -(1 as libc::c_int) {
        Fatal(
            b"Shader_GetVariable: Shader <%s> has no variable <%s>\0" as *const u8
                as *const libc::c_char,
            (*this).name,
            name,
        );
    }
    return index;
}
#[no_mangle]
pub unsafe extern "C" fn Shader_HasVariable(
    mut this: *mut Shader,
    mut name: cstr,
) -> bool {
    return __glewGetUniformLocation
        .expect("non-null function pointer")((*this).program, name)
        > -(1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ResetTexIndex() {
    (*current).texIndex = 1 as libc::c_int as uint;
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat(mut name: cstr, mut value: libc::c_float) {
    __glewUniform1f
        .expect("non-null function pointer")(GetUniformIndex(current, name), value);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat(
    mut index: libc::c_int,
    mut value: libc::c_float,
) {
    __glewUniform1f.expect("non-null function pointer")(index, value);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat2(
    mut name: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
) {
    __glewUniform2f
        .expect("non-null function pointer")(GetUniformIndex(current, name), x, y);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat2(
    mut index: libc::c_int,
    mut x: libc::c_float,
    mut y: libc::c_float,
) {
    __glewUniform2f.expect("non-null function pointer")(index, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat3(
    mut name: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) {
    __glewUniform3f
        .expect("non-null function pointer")(GetUniformIndex(current, name), x, y, z);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat3(
    mut index: libc::c_int,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) {
    __glewUniform3f.expect("non-null function pointer")(index, x, y, z);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetFloat4(
    mut name: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) {
    __glewUniform4f
        .expect("non-null function pointer")(GetUniformIndex(current, name), x, y, z, w);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetFloat4(
    mut index: libc::c_int,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) {
    __glewUniform4f.expect("non-null function pointer")(index, x, y, z, w);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetInt(mut name: cstr, mut value: libc::c_int) {
    __glewUniform1i
        .expect("non-null function pointer")(GetUniformIndex(current, name), value);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetInt(mut index: libc::c_int, mut value: libc::c_int) {
    __glewUniform1i.expect("non-null function pointer")(index, value);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetMatrix(mut name: cstr, mut value: *mut Matrix) {
    __glewUniformMatrix4fv
        .expect(
            "non-null function pointer",
        )(
        GetUniformIndex(current, name),
        1 as libc::c_int,
        1 as libc::c_int as GLboolean,
        value as *mut libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetMatrixT(mut name: cstr, mut value: *mut Matrix) {
    __glewUniformMatrix4fv
        .expect(
            "non-null function pointer",
        )(
        GetUniformIndex(current, name),
        1 as libc::c_int,
        0 as libc::c_int as GLboolean,
        value as *mut libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetMatrix(
    mut index: libc::c_int,
    mut value: *mut Matrix,
) {
    __glewUniformMatrix4fv
        .expect(
            "non-null function pointer",
        )(
        index,
        1 as libc::c_int,
        1 as libc::c_int as GLboolean,
        value as *mut libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetMatrixT(
    mut index: libc::c_int,
    mut value: *mut Matrix,
) {
    __glewUniformMatrix4fv
        .expect(
            "non-null function pointer",
        )(
        index,
        1 as libc::c_int,
        0 as libc::c_int as GLboolean,
        value as *mut libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex1D(mut name: cstr, mut value: *mut Tex1D) {
    __glewUniform1i
        .expect(
            "non-null function pointer",
        )(GetUniformIndex(current, name), (*current).texIndex as GLint);
    let fresh14 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh14));
    glBindTexture(0xde0 as libc::c_int as GLenum, Tex1D_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex1D(
    mut index: libc::c_int,
    mut value: *mut Tex1D,
) {
    __glewUniform1i
        .expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh15 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh15));
    glBindTexture(0xde0 as libc::c_int as GLenum, Tex1D_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex2D(mut name: cstr, mut value: *mut Tex2D) {
    __glewUniform1i
        .expect(
            "non-null function pointer",
        )(GetUniformIndex(current, name), (*current).texIndex as GLint);
    let fresh16 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh16));
    glBindTexture(0xde1 as libc::c_int as GLenum, Tex2D_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex2D(
    mut index: libc::c_int,
    mut value: *mut Tex2D,
) {
    __glewUniform1i
        .expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh17 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh17));
    glBindTexture(0xde1 as libc::c_int as GLenum, Tex2D_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetTex3D(mut name: cstr, mut value: *mut Tex3D) {
    __glewUniform1i
        .expect(
            "non-null function pointer",
        )(GetUniformIndex(current, name), (*current).texIndex as GLint);
    let fresh18 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh18));
    glBindTexture(0x806f as libc::c_int as GLenum, Tex3D_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTex3D(
    mut index: libc::c_int,
    mut value: *mut Tex3D,
) {
    __glewUniform1i
        .expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh19 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh19));
    glBindTexture(0x806f as libc::c_int as GLenum, Tex3D_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_SetTexCube(mut name: cstr, mut value: *mut TexCube) {
    __glewUniform1i
        .expect(
            "non-null function pointer",
        )(GetUniformIndex(current, name), (*current).texIndex as GLint);
    let fresh20 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh20));
    glBindTexture(0x8513 as libc::c_int as GLenum, TexCube_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn Shader_ISetTexCube(
    mut index: libc::c_int,
    mut value: *mut TexCube,
) {
    __glewUniform1i
        .expect("non-null function pointer")(index, (*current).texIndex as GLint);
    let fresh21 = (*current).texIndex;
    (*current).texIndex = ((*current).texIndex).wrapping_add(1);
    __glewActiveTexture
        .expect(
            "non-null function pointer",
        )((0x84c0 as libc::c_int as libc::c_uint).wrapping_add(fresh21));
    glBindTexture(0x8513 as libc::c_int as GLenum, TexCube_GetHandle(value));
    __glewActiveTexture
        .expect("non-null function pointer")(0x84c0 as libc::c_int as GLenum);
}
