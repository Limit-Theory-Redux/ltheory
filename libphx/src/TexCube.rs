use crate::internal::Memory::*;
use crate::CubeFace::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
use crate::RenderState::*;
use crate::RenderTarget::*;
use crate::GLMatrix::*;
use crate::Tex2D_Load::*;
use crate::Tex2D_Save::*;
use crate::Shader::*;
use crate::Bytes::*;
use crate::ClipRect::*;
use crate::ShaderState::*;
use crate::Draw::*;
use crate::Tex2D::*;
use crate::TimeStamp::*;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn floor(_: f64) -> f64;
    fn glBindTexture(target: GLenum, texture: GLu32);
    fn glDeleteTextures(n: GLsizei, textures: *const GLu32);
    fn glGenTextures(n: GLsizei, textures: *mut GLu32);
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
}

pub type uchar = libc::c_uchar;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TexCube {
    pub _refCount: u32,
    pub handle: u32,
    pub size: i32,
    pub format: TexFormat,
}
pub type TexFormat = i32;
pub type CubeFace = i32;
pub type DataFormat = i32;
pub type PixelFormat = i32;
pub type TexFilter = i32;
pub type TimeStamp = u64;
pub type GLenum = u32;
pub type GLu32 = u32;
pub type GLint = i32;
pub type GLsizei = i32;
pub type PFNGLGENERATEMIPMAPPROC = Option<unsafe extern "C" fn(GLenum) -> ()>;

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
unsafe extern "C" fn Max(mut a: f64, mut b: f64) -> f64 {
    return if a > b { a } else { b };
}

#[inline]
unsafe extern "C" fn Min(mut a: f64, mut b: f64) -> f64 {
    return if a < b { a } else { b };
}

static mut kFaces: [Face; 6] = [
    Face {
        face: 0x8515_i32,
        look: Vec3::new(1.0f32, 0.0f32, 0.0f32),
        up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
    },
    Face {
        face: 0x8516_i32,
        look: Vec3::new(-1.0f32, 0.0f32, 0.0f32),
        up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
    },
    Face {
        face: 0x8517_i32,
        look: Vec3::new(0.0f32, 1.0f32, 0.0f32),
        up: Vec3::new(0.0f32, 0.0f32, -1.0f32),
    },
    Face {
        face: 0x8518_i32,
        look: Vec3::new(0.0f32, -1.0f32, 0.0f32),
        up: Vec3::new(0.0f32, 0.0f32, 1.0f32),
    },
    Face {
        face: 0x8519_i32,
        look: Vec3::new(0.0f32, 0.0f32, 1.0f32),
        up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
    },
    Face {
        face: 0x851a_i32,
        look: Vec3::new(0.0f32, 0.0f32, -1.0f32),
        up: Vec3::new(0.0f32, 1.0f32, 0.0f32),
    },
];
static mut kFaceExt: [*const libc::c_char; 6] = [
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
        0x8513_i32 as GLenum,
        0x2800_i32 as GLenum,
        0x2600_i32,
    );
    glTexParameteri(
        0x8513_i32 as GLenum,
        0x2801_i32 as GLenum,
        0x2600_i32,
    );
    glTexParameteri(
        0x8513_i32 as GLenum,
        0x2802_i32 as GLenum,
        0x812f_i32,
    );
    glTexParameteri(
        0x8513_i32 as GLenum,
        0x2803_i32 as GLenum,
        0x812f_i32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Create(mut size: i32, mut format: TexFormat) -> *mut TexCube {
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
    let mut this: *mut TexCube =
        MemAlloc(::core::mem::size_of::<TexCube>()) as *mut TexCube;
    (*this)._refCount = 1_i32 as u32;
    glGenTextures(1_i32, &mut (*this).handle);
    (*this).size = size;
    (*this).format = format;
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    glTexImage2D(
        0x8515_i32 as GLenum,
        0_i32,
        format,
        size,
        size,
        0_i32,
        0x1903_i32 as GLenum,
        0x1400_i32 as GLenum,
        std::ptr::null(),
    );
    glTexImage2D(
        0x8517_i32 as GLenum,
        0_i32,
        format,
        size,
        size,
        0_i32,
        0x1903_i32 as GLenum,
        0x1400_i32 as GLenum,
        std::ptr::null(),
    );
    glTexImage2D(
        0x8519_i32 as GLenum,
        0_i32,
        format,
        size,
        size,
        0_i32,
        0x1903_i32 as GLenum,
        0x1400_i32 as GLenum,
        std::ptr::null(),
    );
    glTexImage2D(
        0x8516_i32 as GLenum,
        0_i32,
        format,
        size,
        size,
        0_i32,
        0x1903_i32 as GLenum,
        0x1400_i32 as GLenum,
        std::ptr::null(),
    );
    glTexImage2D(
        0x8518_i32 as GLenum,
        0_i32,
        format,
        size,
        size,
        0_i32,
        0x1903_i32 as GLenum,
        0x1400_i32 as GLenum,
        std::ptr::null(),
    );
    glTexImage2D(
        0x851a_i32 as GLenum,
        0_i32,
        format,
        size,
        size,
        0_i32,
        0x1903_i32 as GLenum,
        0x1400_i32 as GLenum,
        std::ptr::null(),
    );
    TexCube_InitParameters();
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
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
    let mut i: i32 = 0_i32;
    while i < 6_i32 {
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
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        glDeleteTextures(1_i32, &mut (*this).handle);
        MemFree(this as *const libc::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Load(mut path: *const libc::c_char) -> *mut TexCube {
    let mut this: *mut TexCube =
        MemAlloc(::core::mem::size_of::<TexCube>()) as *mut TexCube;
    glGenTextures(1_i32, &mut (*this).handle);
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    let mut components: i32 = 0_i32;
    let mut dataLayout: i32 = 0_i32;
    let mut i: i32 = 0_i32;
    while i < 6_i32 {
        let mut facePath: *const libc::c_char = StrAdd3(
            path,
            kFaceExt[i as usize],
            b".jpg\0" as *const u8 as *const libc::c_char,
        );
        let mut sx: i32 = 0;
        let mut sy: i32 = 0;
        let mut lcomponents: i32 = 0;
        let mut data: *mut uchar = Tex2D_LoadRaw(facePath, &mut sx, &mut sy, &mut lcomponents);
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
                    b"TexCube_Load loaded cubemap faces have different resolutions\0" as *const u8
                        as *const libc::c_char,
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
            (*this).format = if components == 4_i32 {
                TexFormat_RGBA8
            } else if components == 3_i32 {
                TexFormat_RGB8
            } else if components == 2_i32 {
                TexFormat_RG8
            } else {
                TexFormat_R8
            };
            dataLayout = if components == 4_i32 {
                0x1908_i32
            } else if components == 3_i32 {
                0x1907_i32
            } else if components == 2_i32 {
                0x8227_i32
            } else {
                0x1903_i32
            };
        }
        glTexImage2D(
            kFaces[i as usize].face as GLenum,
            0_i32,
            (*this).format,
            (*this).size,
            (*this).size,
            0_i32,
            dataLayout as GLenum,
            0x1401_i32 as GLenum,
            data as *const libc::c_void,
        );
        MemFree(facePath as *const libc::c_void);
        MemFree(data as *const libc::c_void);
        i += 1;
    }
    TexCube_InitParameters();
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GetData(
    mut this: *mut TexCube,
    mut data: *mut libc::c_void,
    mut face: CubeFace,
    mut level: i32,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    glGetTexImage(face as GLenum, level, pf as GLenum, df as GLenum, data);
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GetDataBytes(
    mut this: *mut TexCube,
    mut face: CubeFace,
    mut level: i32,
    mut pf: PixelFormat,
    mut df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = (*this).size * (*this).size;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);
    let mut data: *mut Bytes = Bytes_Create(size as u32);
    TexCube_GetData(this, Bytes_GetData(data), face, level, pf, df);
    Bytes_Rewind(data);
    return data;
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GetFormat(mut this: *mut TexCube) -> TexFormat {
    return (*this).format;
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GetHandle(mut this: *mut TexCube) -> u32 {
    return (*this).handle;
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GetSize(mut this: *mut TexCube) -> i32 {
    return (*this).size;
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Generate(mut this: *mut TexCube, mut state: *mut ShaderState) {
    GLMatrix_ModeP();
    GLMatrix_Push();
    GLMatrix_Clear();
    GLMatrix_ModeWV();
    GLMatrix_Push();
    GLMatrix_Clear();
    RenderState_PushAllDefaults();
    ShaderState_Start(state);
    let mut i: i32 = 0_i32;
    while i < 6_i32 {
        let mut face: Face = kFaces[i as usize];
        let mut size: i32 = (*this).size;
        let mut fSize: f32 = (*this).size as f32;
        RenderTarget_Push(size, size);
        RenderTarget_BindTexCube(this, face.face);
        Draw_Clear(0.0f32, 0.0f32, 0.0f32, 1.0f32);
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
        let mut j: i32 = 1_i32;
        let mut jobSize: i32 = 1_i32;
        while j <= size {
            let mut time: TimeStamp = TimeStamp_Get();
            ClipRect_Push(0.0f32, (j - 1_i32) as f32, size as f32, jobSize as f32);
            Draw_Rect(0.0f32, 0.0f32, fSize, fSize);
            Draw_Flush();
            ClipRect_Pop();
            j += jobSize;
            let mut elapsed: f64 = TimeStamp_GetElapsed(time);
            jobSize = Max(
                1_f64,
                Floor(0.25f64 * jobSize as f64 / elapsed + 0.5f64) as i32 as f64,
            ) as i32;
            jobSize = Min(jobSize as f64, (size - j + 1_i32) as f64) as i32;
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
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    __glewGenerateMipmap.expect("non-null function pointer")(0x8513_i32 as GLenum);
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SetData(
    mut this: *mut TexCube,
    mut data: *const libc::c_void,
    mut face: CubeFace,
    mut level: i32,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    glTexImage2D(
        face as GLenum,
        level,
        (*this).format,
        (*this).size,
        (*this).size,
        0_i32,
        pf as GLenum,
        df as GLenum,
        data,
    );
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SetDataBytes(
    mut this: *mut TexCube,
    mut data: *mut Bytes,
    mut face: CubeFace,
    mut level: i32,
    mut pf: PixelFormat,
    mut df: DataFormat,
) {
    TexCube_SetData(this, Bytes_GetData(data), face, level, pf, df);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SetMagFilter(mut this: *mut TexCube, mut filter: TexFilter) {
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    glTexParameteri(0x8513_i32 as GLenum, 0x2800_i32 as GLenum, filter);
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SetMinFilter(mut this: *mut TexCube, mut filter: TexFilter) {
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    glTexParameteri(0x8513_i32 as GLenum, 0x2801_i32 as GLenum, filter);
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Save(mut this: *mut TexCube, mut path: *const libc::c_char) {
    TexCube_SaveLevel(this, path, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SaveLevel(mut this: *mut TexCube, mut path: *const libc::c_char, mut level: i32) {
    let mut size: i32 = (*this).size >> level;
    glBindTexture(0x8513_i32 as GLenum, (*this).handle);
    let mut buffer: *mut uchar =
        MemAlloc((::core::mem::size_of::<uchar>()).wrapping_mul((4_i32 * size * size) as usize))
            as *mut uchar;
    let mut i: i32 = 0_i32;
    while i < 6_i32 {
        let mut face: CubeFace = kFaces[i as usize].face;
        let mut facePath: *const libc::c_char = StrAdd3(
            path,
            kFaceExt[i as usize],
            b".png\0" as *const u8 as *const libc::c_char,
        );
        glGetTexImage(
            face as GLenum,
            level,
            0x1908_i32 as GLenum,
            0x1401_i32 as GLenum,
            buffer as *mut libc::c_void,
        );
        Tex2D_Save_Png(facePath, size, size, 4_i32, buffer);
        MemFree(facePath as *const libc::c_void);
        i += 1;
    }
    MemFree(buffer as *const libc::c_void);
    glBindTexture(0x8513_i32 as GLenum, 0_i32 as GLu32);
}
