use ::libc;
extern "C" {
    pub type Bytes;
    pub type ShaderState;
    fn Fatal(_: cstr, _: ...);
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Bytes_Rewind(_: *mut Bytes);
    fn ClipRect_Push(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn ClipRect_Pop();
    fn DataFormat_GetSize(_: DataFormat) -> libc::c_int;
    fn floor(_: libc::c_double) -> libc::c_double;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn Draw_Rect(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn Draw_Flush();
    fn Draw_Clear(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn GLMatrix_Clear();
    fn GLMatrix_ModeP();
    fn GLMatrix_ModeWV();
    fn GLMatrix_Pop();
    fn GLMatrix_Push();
    fn glBindTexture(target: GLenum, texture: GLuint);
    fn glDeleteTextures(n: GLsizei, textures: *const GLuint);
    fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    fn glGetTexImage(
        target: GLenum,
        level: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *mut libc::c_void,
    );
    fn glTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_0: GLenum,
        pixels: *const libc::c_void,
    );
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    static mut __glewGenerateMipmap: PFNGLGENERATEMIPMAPPROC;
    fn PixelFormat_Components(_: PixelFormat) -> libc::c_int;
    fn RenderState_PushAllDefaults();
    fn RenderState_PopAll();
    fn RenderTarget_Push(sx: libc::c_int, sy: libc::c_int);
    fn RenderTarget_Pop();
    fn RenderTarget_BindTexCube(_: *mut TexCube, _: CubeFace);
    fn Shader_SetFloat(_: cstr, _: libc::c_float);
    fn Shader_SetFloat3(_: cstr, _: libc::c_float, _: libc::c_float, _: libc::c_float);
    fn ShaderState_Start(_: *mut ShaderState);
    fn ShaderState_Stop(_: *mut ShaderState);
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> libc::c_double;
    fn TexFormat_IsDepth(_: TexFormat) -> bool;
    fn TexFormat_IsValid(_: TexFormat) -> bool;
    fn Tex2D_LoadRaw(
        path: cstr,
        sx: *mut libc::c_int,
        sy: *mut libc::c_int,
        components: *mut libc::c_int,
    ) -> *mut uchar;
    fn Tex2D_Save_Png(
        path: cstr,
        sx: libc::c_int,
        sy: libc::c_int,
        components: libc::c_int,
        data: *mut uchar,
    ) -> bool;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type uint = libc::c_uint;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TexCube {
    pub _refCount: uint32,
    pub handle: uint,
    pub size: libc::c_int,
    pub format: TexFormat,
}
pub type TexFormat = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
pub type CubeFace = int32;
pub type DataFormat = int32;
pub type PixelFormat = int32;
pub type TexFilter = int32;
pub type TimeStamp = uint64;
pub type GLenum = libc::c_uint;
pub type GLuint = libc::c_uint;
pub type GLint = libc::c_int;
pub type GLsizei = libc::c_int;
pub type PFNGLGENERATEMIPMAPPROC = Option::<unsafe extern "C" fn(GLenum) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Face {
    pub face: CubeFace,
    pub look: Vec3f,
    pub up: Vec3f,
}
#[no_mangle]
pub static mut CubeFace_PX: CubeFace = 0;
#[no_mangle]
pub static mut CubeFace_NX: CubeFace = 0;
#[no_mangle]
pub static mut CubeFace_PY: CubeFace = 0;
#[no_mangle]
pub static mut CubeFace_NY: CubeFace = 0;
#[no_mangle]
pub static mut CubeFace_PZ: CubeFace = 0;
#[no_mangle]
pub static mut CubeFace_NZ: CubeFace = 0;
#[no_mangle]
pub static mut DataFormat_U8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_Float: DataFormat = 0;
#[inline]
unsafe extern "C" fn Floor(mut t: libc::c_double) -> libc::c_double {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn MemAlloc(mut size: size_t) -> *mut libc::c_void {
    return malloc(size);
}
#[inline]
unsafe extern "C" fn MemFree(mut ptr: *const libc::c_void) {
    free(ptr as *mut libc::c_void);
}
#[no_mangle]
pub static mut PixelFormat_Red: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RG: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGB: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGR: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGBA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGRA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_Depth_Component: PixelFormat = 0;
#[inline]
unsafe extern "C" fn StrAlloc(mut len: size_t) -> *mut libc::c_char {
    return malloc(len) as *mut libc::c_char;
}
#[inline]
unsafe extern "C" fn StrAdd3(mut a: cstr, mut b: cstr, mut c: cstr) -> cstr {
    let mut buf: *mut libc::c_char = StrAlloc(
        (StrLen(a))
            .wrapping_add(StrLen(b))
            .wrapping_add(StrLen(c))
            .wrapping_add(1 as libc::c_int as libc::c_ulong),
    );
    let mut cur: *mut libc::c_char = buf;
    while *a != 0 {
        let fresh0 = a;
        a = a.offset(1);
        let fresh1 = cur;
        cur = cur.offset(1);
        *fresh1 = *fresh0;
    }
    while *b != 0 {
        let fresh2 = b;
        b = b.offset(1);
        let fresh3 = cur;
        cur = cur.offset(1);
        *fresh3 = *fresh2;
    }
    while *c != 0 {
        let fresh4 = c;
        c = c.offset(1);
        let fresh5 = cur;
        cur = cur.offset(1);
        *fresh5 = *fresh4;
    }
    *cur = 0 as libc::c_int as libc::c_char;
    return buf as cstr;
}
#[inline]
unsafe extern "C" fn StrLen(mut s: cstr) -> size_t {
    if s.is_null() {
        return 0 as libc::c_int as size_t;
    }
    let mut begin: cstr = s;
    while *s != 0 {
        s = s.offset(1);
    }
    return s.offset_from(begin) as libc::c_long as size_t;
}
#[no_mangle]
pub static mut TexFormat_R8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGB8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth24: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth32F: TexFormat = 0;
static mut kFaces: [Face; 6] = [
    {
        let mut init = Face {
            face: 0x8515 as libc::c_int,
            look: {
                let mut init = Vec3f {
                    x: 1 as libc::c_int as libc::c_float,
                    y: 0 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
            up: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 1 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
        };
        init
    },
    {
        let mut init = Face {
            face: 0x8516 as libc::c_int,
            look: {
                let mut init = Vec3f {
                    x: -(1 as libc::c_int) as libc::c_float,
                    y: 0 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
            up: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 1 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
        };
        init
    },
    {
        let mut init = Face {
            face: 0x8517 as libc::c_int,
            look: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 1 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
            up: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 0 as libc::c_int as libc::c_float,
                    z: -(1 as libc::c_int) as libc::c_float,
                };
                init
            },
        };
        init
    },
    {
        let mut init = Face {
            face: 0x8518 as libc::c_int,
            look: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: -(1 as libc::c_int) as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
            up: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 0 as libc::c_int as libc::c_float,
                    z: 1 as libc::c_int as libc::c_float,
                };
                init
            },
        };
        init
    },
    {
        let mut init = Face {
            face: 0x8519 as libc::c_int,
            look: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 0 as libc::c_int as libc::c_float,
                    z: 1 as libc::c_int as libc::c_float,
                };
                init
            },
            up: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 1 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
        };
        init
    },
    {
        let mut init = Face {
            face: 0x851a as libc::c_int,
            look: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 0 as libc::c_int as libc::c_float,
                    z: -(1 as libc::c_int) as libc::c_float,
                };
                init
            },
            up: {
                let mut init = Vec3f {
                    x: 0 as libc::c_int as libc::c_float,
                    y: 1 as libc::c_int as libc::c_float,
                    z: 0 as libc::c_int as libc::c_float,
                };
                init
            },
        };
        init
    },
];
static mut kFaceExt: [cstr; 6] = [
    b"px\0" as *const u8 as *const libc::c_char,
    b"py\0" as *const u8 as *const libc::c_char,
    b"pz\0" as *const u8 as *const libc::c_char,
    b"nx\0" as *const u8 as *const libc::c_char,
    b"ny\0" as *const u8 as *const libc::c_char,
    b"nz\0" as *const u8 as *const libc::c_char,
];
#[inline]
unsafe extern "C" fn TexCube_InitParameters() {
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        0x2600 as libc::c_int,
    );
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2802 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2803 as libc::c_int as GLenum,
        0x812f as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Create(
    mut size: libc::c_int,
    mut format: TexFormat,
) -> *mut TexCube {
    if !TexFormat_IsValid(format) {
        Fatal(
            b"TexCube_Create: Invalid texture format requested\0" as *const u8
                as *const libc::c_char,
        );
    }
    if TexFormat_IsDepth(format) {
        Fatal(
            b"TexCube_Create: Cannot create cubemap with depth format\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut self_0: *mut TexCube = MemAlloc(
        ::core::mem::size_of::<TexCube>() as libc::c_ulong,
    ) as *mut TexCube;
    (*self_0)._refCount = 1 as libc::c_int as uint32;
    glGenTextures(1 as libc::c_int, &mut (*self_0).handle);
    (*self_0).size = size;
    (*self_0).format = format;
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    glTexImage2D(
        0x8515 as libc::c_int as GLenum,
        0 as libc::c_int,
        format,
        size,
        size,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1400 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    glTexImage2D(
        0x8517 as libc::c_int as GLenum,
        0 as libc::c_int,
        format,
        size,
        size,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1400 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    glTexImage2D(
        0x8519 as libc::c_int as GLenum,
        0 as libc::c_int,
        format,
        size,
        size,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1400 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    glTexImage2D(
        0x8516 as libc::c_int as GLenum,
        0 as libc::c_int,
        format,
        size,
        size,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1400 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    glTexImage2D(
        0x8518 as libc::c_int as GLenum,
        0 as libc::c_int,
        format,
        size,
        size,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1400 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    glTexImage2D(
        0x851a as libc::c_int as GLenum,
        0 as libc::c_int,
        format,
        size,
        size,
        0 as libc::c_int,
        0x1903 as libc::c_int as GLenum,
        0x1400 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
    TexCube_InitParameters();
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Acquire(mut self_0: *mut TexCube) {
    (*self_0)._refCount = ((*self_0)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Clear(
    mut self_0: *mut TexCube,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        let mut face: Face = kFaces[i as usize];
        RenderTarget_Push((*self_0).size, (*self_0).size);
        RenderTarget_BindTexCube(self_0, face.face);
        Draw_Clear(r, g, b, a);
        RenderTarget_Pop();
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Free(mut self_0: *mut TexCube) {
    if !self_0.is_null()
        && {
            (*self_0)._refCount = ((*self_0)._refCount).wrapping_sub(1);
            (*self_0)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        glDeleteTextures(1 as libc::c_int, &mut (*self_0).handle);
        MemFree(self_0 as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Load(mut path: cstr) -> *mut TexCube {
    let mut self_0: *mut TexCube = MemAlloc(
        ::core::mem::size_of::<TexCube>() as libc::c_ulong,
    ) as *mut TexCube;
    glGenTextures(1 as libc::c_int, &mut (*self_0).handle);
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    let mut components: libc::c_int = 0 as libc::c_int;
    let mut dataLayout: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        let mut facePath: cstr = StrAdd3(
            path,
            kFaceExt[i as usize],
            b".jpg\0" as *const u8 as *const libc::c_char,
        );
        let mut sx: libc::c_int = 0;
        let mut sy: libc::c_int = 0;
        let mut lcomponents: libc::c_int = 0;
        let mut data: *mut uchar = Tex2D_LoadRaw(
            facePath,
            &mut sx,
            &mut sy,
            &mut lcomponents,
        );
        if data.is_null() {
            Fatal(
                b"TexCube_Load failed to load cubemap face from '%s'\0" as *const u8
                    as *const libc::c_char,
                facePath,
            );
        }
        if sx != sy {
            Fatal(
                b"TexCube_Load loaded cubemap face is not square\0" as *const u8
                    as *const libc::c_char,
            );
        }
        if i != 0 {
            if sx != (*self_0).size || sy != (*self_0).size {
                Fatal(
                    b"TexCube_Load loaded cubemap faces have different resolutions\0"
                        as *const u8 as *const libc::c_char,
                );
            }
            if lcomponents != components {
                Fatal(
                    b"TexCube_Load loaded cubemap faces have different number of components\0"
                        as *const u8 as *const libc::c_char,
                );
            }
        } else {
            components = lcomponents;
            (*self_0).size = sx;
            (*self_0)
                .format = if components == 4 as libc::c_int {
                TexFormat_RGBA8
            } else if components == 3 as libc::c_int {
                TexFormat_RGB8
            } else if components == 2 as libc::c_int {
                TexFormat_RG8
            } else {
                TexFormat_R8
            };
            dataLayout = if components == 4 as libc::c_int {
                0x1908 as libc::c_int
            } else if components == 3 as libc::c_int {
                0x1907 as libc::c_int
            } else if components == 2 as libc::c_int {
                0x8227 as libc::c_int
            } else {
                0x1903 as libc::c_int
            };
        }
        glTexImage2D(
            kFaces[i as usize].face as GLenum,
            0 as libc::c_int,
            (*self_0).format,
            (*self_0).size,
            (*self_0).size,
            0 as libc::c_int,
            dataLayout as GLenum,
            0x1401 as libc::c_int as GLenum,
            data as *const libc::c_void,
        );
        MemFree(facePath as *const libc::c_void);
        MemFree(data as *const libc::c_void);
        i += 1;
    }
    TexCube_InitParameters();
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetData(
    mut self_0: *mut TexCube,
    mut data: *mut libc::c_void,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    glGetTexImage(face as GLenum, level, pf as GLenum, df as GLenum, data);
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetDataBytes(
    mut self_0: *mut TexCube,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: libc::c_int = (*self_0).size * (*self_0).size;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as uint32);
    TexCube_GetData(self_0, Bytes_GetData(data), face, level, pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetFormat(mut self_0: *mut TexCube) -> TexFormat {
    return (*self_0).format;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetHandle(mut self_0: *mut TexCube) -> uint {
    return (*self_0).handle;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetSize(mut self_0: *mut TexCube) -> libc::c_int {
    return (*self_0).size;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Generate(
    mut self_0: *mut TexCube,
    mut state: *mut ShaderState,
) {
    GLMatrix_ModeP();
    GLMatrix_Push();
    GLMatrix_Clear();
    GLMatrix_ModeWV();
    GLMatrix_Push();
    GLMatrix_Clear();
    RenderState_PushAllDefaults();
    ShaderState_Start(state);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        let mut face: Face = kFaces[i as usize];
        let mut size: libc::c_int = (*self_0).size;
        let mut fSize: libc::c_float = (*self_0).size as libc::c_float;
        RenderTarget_Push(size, size);
        RenderTarget_BindTexCube(self_0, face.face);
        Draw_Clear(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
        );
        Shader_SetFloat3(
            b"cubeLook\0" as *const u8 as *const libc::c_char,
            face.look.x,
            face.look.y,
            face.look.z,
        );
        Shader_SetFloat3(
            b"cubeUp\0" as *const u8 as *const libc::c_char,
            face.up.x,
            face.up.y,
            face.up.z,
        );
        Shader_SetFloat(b"cubeSize\0" as *const u8 as *const libc::c_char, fSize);
        let mut j: libc::c_int = 1 as libc::c_int;
        let mut jobSize: libc::c_int = 1 as libc::c_int;
        while j <= size {
            let mut time: TimeStamp = TimeStamp_Get();
            ClipRect_Push(
                0 as libc::c_int as libc::c_float,
                (j - 1 as libc::c_int) as libc::c_float,
                size as libc::c_float,
                jobSize as libc::c_float,
            );
            Draw_Rect(
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                fSize,
                fSize,
            );
            Draw_Flush();
            ClipRect_Pop();
            j += jobSize;
            let mut elapsed: libc::c_double = TimeStamp_GetElapsed(time);
            jobSize = Max(
                1 as libc::c_int as libc::c_double,
                Floor(0.25f64 * jobSize as libc::c_double / elapsed + 0.5f64)
                    as libc::c_int as libc::c_double,
            ) as libc::c_int;
            jobSize = Min(
                jobSize as libc::c_double,
                (size - j + 1 as libc::c_int) as libc::c_double,
            ) as libc::c_int;
        }
        RenderTarget_Pop();
        i += 1;
    }
    ShaderState_Stop(state);
    RenderState_PopAll();
    GLMatrix_ModeP();
    GLMatrix_Pop();
    GLMatrix_ModeWV();
    GLMatrix_Pop();
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GenMipmap(mut self_0: *mut TexCube) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    __glewGenerateMipmap
        .expect("non-null function pointer")(0x8513 as libc::c_int as GLenum);
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetData(
    mut self_0: *mut TexCube,
    mut data: *const libc::c_void,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    glTexImage2D(
        face as GLenum,
        level,
        (*self_0).format,
        (*self_0).size,
        (*self_0).size,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetDataBytes(
    mut self_0: *mut TexCube,
    mut data: *mut Bytes,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    TexCube_SetData(self_0, Bytes_GetData(data), face, level, pf, df);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetMagFilter(
    mut self_0: *mut TexCube,
    mut filter: TexFilter,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetMinFilter(
    mut self_0: *mut TexCube,
    mut filter: TexFilter,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Save(mut self_0: *mut TexCube, mut path: cstr) {
    TexCube_SaveLevel(self_0, path, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SaveLevel(
    mut self_0: *mut TexCube,
    mut path: cstr,
    mut level: libc::c_int,
) {
    let mut size: libc::c_int = (*self_0).size >> level;
    glBindTexture(0x8513 as libc::c_int as GLenum, (*self_0).handle);
    let mut buffer: *mut uchar = MemAlloc(
        (::core::mem::size_of::<uchar>() as libc::c_ulong)
            .wrapping_mul((4 as libc::c_int * size * size) as libc::c_ulong),
    ) as *mut uchar;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        let mut face: CubeFace = kFaces[i as usize].face;
        let mut facePath: cstr = StrAdd3(
            path,
            kFaceExt[i as usize],
            b".png\0" as *const u8 as *const libc::c_char,
        );
        glGetTexImage(
            face as GLenum,
            level,
            0x1908 as libc::c_int as GLenum,
            0x1401 as libc::c_int as GLenum,
            buffer as *mut libc::c_void,
        );
        Tex2D_Save_Png(facePath, size, size, 4 as libc::c_int, buffer);
        MemFree(facePath as *const libc::c_void);
        i += 1;
    }
    MemFree(buffer as *const libc::c_void);
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
