use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::CubeFace::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
extern "C" {
    pub type Bytes;
    pub type ShaderState;
    fn Fatal(_: cstr, _: ...);
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Bytes_Rewind(_: *mut Bytes);
    fn ClipRect_Push(
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    );
    fn ClipRect_Pop();
    fn DataFormat_GetSize(_: DataFormat) -> libc::c_int;
    fn floor(_: f64) -> f64;
    fn Draw_Rect(
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    );
    fn Draw_Flush();
    fn Draw_Clear(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
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
    fn Shader_SetFloat(_: cstr, _: f32);
    fn Shader_SetFloat3(_: cstr, _: f32, _: f32, _: f32);
    fn ShaderState_Start(_: *mut ShaderState);
    fn ShaderState_Stop(_: *mut ShaderState);
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> f64;
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
    pub look: Vec3,
    pub up: Vec3,
}
#[inline]
unsafe extern "C" fn Floor(mut t: f64) -> f64 {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a < b { a } else { b };
}

static mut kFaces: [Face; 6] = [
     Face {
            face: 0x8515 as libc::c_int,
            look: Vec3::new(1.0f32, 0.0f32, 0.0f32),
            up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
        },
     Face {
            face: 0x8516 as libc::c_int,
            look: Vec3::new(-1.0f32, 0.0f32, 0.0f32),
            up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
        },
     Face {
            face: 0x8517 as libc::c_int,
            look: Vec3::new(0.0f32, 1.0f32, 0.0f32),
            up: Vec3::new(0.0f32, 0.0f32, -1.0f32),
        },
     Face {
            face: 0x8518 as libc::c_int,
            look: Vec3::new(0.0f32, -1.0f32, 0.0f32),
            up: Vec3::new(0.0f32, 0.0f32, 1.0f32),
        },
     Face {
            face: 0x8519 as libc::c_int,
            look: Vec3::new(0.0f32, 0.0f32, 1.0f32),
            up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
        },
     Face {
            face: 0x851a as libc::c_int,
            look: Vec3::new(0.0f32, 0.0f32, -1.0f32),
            up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
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
    let mut this: *mut TexCube = MemAlloc(
        ::core::mem::size_of::<TexCube>() as usize,
    ) as *mut TexCube;
    (*this)._refCount = 1 as libc::c_int as uint32;
    glGenTextures(1 as libc::c_int, &mut (*this).handle);
    (*this).size = size;
    (*this).format = format;
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
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
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Acquire(mut this: *mut TexCube) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Clear(
    mut this: *mut TexCube,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        let mut face: Face = kFaces[i as usize];
        RenderTarget_Push((*this).size, (*this).size);
        RenderTarget_BindTexCube(this, face.face);
        Draw_Clear(r, g, b, a);
        RenderTarget_Pop();
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Free(mut this: *mut TexCube) {
    if !this.is_null()
        && {
            (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
            (*this)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        glDeleteTextures(1 as libc::c_int, &mut (*this).handle);
        MemFree(this as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Load(mut path: cstr) -> *mut TexCube {
    let mut this: *mut TexCube = MemAlloc(
        ::core::mem::size_of::<TexCube>() as usize,
    ) as *mut TexCube;
    glGenTextures(1 as libc::c_int, &mut (*this).handle);
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
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
            if sx != (*this).size || sy != (*this).size {
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
            (*this).size = sx;
            (*this)
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
            (*this).format,
            (*this).size,
            (*this).size,
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
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetData(
    mut this: *mut TexCube,
    mut data: *mut libc::c_void,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
    glGetTexImage(face as GLenum, level, pf as GLenum, df as GLenum, data);
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetDataBytes(
    mut this: *mut TexCube,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: libc::c_int = (*this).size * (*this).size;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as uint32);
    TexCube_GetData(this, Bytes_GetData(data), face, level, pf, df);
    Bytes_Rewind(data);
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetFormat(mut this: *mut TexCube) -> TexFormat {
    return (*this).format;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetHandle(mut this: *mut TexCube) -> uint {
    return (*this).handle;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_GetSize(mut this: *mut TexCube) -> libc::c_int {
    return (*this).size;
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Generate(
    mut this: *mut TexCube,
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
        let mut size: libc::c_int = (*this).size;
        let mut fSize: f32 = (*this).size as f32;
        RenderTarget_Push(size, size);
        RenderTarget_BindTexCube(this, face.face);
        Draw_Clear(
            0.0f32,
            0.0f32,
            0.0f32,
            1.0f32,
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
                0.0f32,
                (j - 1 as libc::c_int) as f32,
                size as f32,
                jobSize as f32,
            );
            Draw_Rect(
                0.0f32,
                0.0f32,
                fSize,
                fSize,
            );
            Draw_Flush();
            ClipRect_Pop();
            j += jobSize;
            let mut elapsed: f64 = TimeStamp_GetElapsed(time);
            jobSize = Max(
                1 as libc::c_int as f64,
                Floor(0.25f64 * jobSize as f64 / elapsed + 0.5f64)
                    as libc::c_int as f64,
            ) as libc::c_int;
            jobSize = Min(
                jobSize as f64,
                (size - j + 1 as libc::c_int) as f64,
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
pub unsafe extern "C" fn TexCube_GenMipmap(mut this: *mut TexCube) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
    __glewGenerateMipmap
        .expect("non-null function pointer")(0x8513 as libc::c_int as GLenum);
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetData(
    mut this: *mut TexCube,
    mut data: *const libc::c_void,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
    glTexImage2D(
        face as GLenum,
        level,
        (*this).format,
        (*this).size,
        (*this).size,
        0 as libc::c_int,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetDataBytes(
    mut this: *mut TexCube,
    mut data: *mut Bytes,
    mut face: CubeFace,
    mut level: libc::c_int,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    TexCube_SetData(this, Bytes_GetData(data), face, level, pf, df);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetMagFilter(
    mut this: *mut TexCube,
    mut filter: TexFilter,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2800 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SetMinFilter(
    mut this: *mut TexCube,
    mut filter: TexFilter,
) {
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
    glTexParameteri(
        0x8513 as libc::c_int as GLenum,
        0x2801 as libc::c_int as GLenum,
        filter,
    );
    glBindTexture(0x8513 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_Save(mut this: *mut TexCube, mut path: cstr) {
    TexCube_SaveLevel(this, path, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn TexCube_SaveLevel(
    mut this: *mut TexCube,
    mut path: cstr,
    mut level: libc::c_int,
) {
    let mut size: libc::c_int = (*this).size >> level;
    glBindTexture(0x8513 as libc::c_int as GLenum, (*this).handle);
    let mut buffer: *mut uchar = MemAlloc(
        (::core::mem::size_of::<uchar>())
            .wrapping_mul((4 as libc::c_int * size * size) as usize),
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
