use internal::*;

use super::*;
use crate::common::*;
use crate::math::*;
use crate::system::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TexCube {
    pub _refCount: u32,
    pub handle: u32,
    pub size: i32,
    pub format: TexFormat,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Face {
    pub face: CubeFace,
    pub look: Vec3,
    pub up: Vec3,
}

static mut kFaces: [Face; 6] = [
    Face {
        face: CubeFace_PX,
        look: Vec3::X,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace_NX,
        look: Vec3::NEG_X,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace_PY,
        look: Vec3::Y,
        up: Vec3::new(0.0f32, 0.0f32, -1.0f32),
    },
    Face {
        face: CubeFace_NY,
        look: Vec3::NEG_Y,
        up: Vec3::Z,
    },
    Face {
        face: CubeFace_PZ,
        look: Vec3::Z,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace_NZ,
        look: Vec3::new(0.0f32, 0.0f32, -1.0f32),
        up: Vec3::Y,
    },
];

const K_FACE_EXT: [&str; 6] = ["px", "py", "pz", "nx", "ny", "nz"];

#[inline]
extern "C" fn TexCube_InitParameters() {
    gl_tex_parameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32,
    );
    gl_tex_parameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST as i32,
    );
    gl_tex_parameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl_tex_parameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as i32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Create(size: i32, format: TexFormat) -> *mut TexCube {
    if !TexFormat_IsValid(format) {
        panic!("TexCube_Create: Invalid texture format requested");
    }
    if TexFormat_IsDepth(format) {
        panic!("TexCube_Create: Cannot create cubemap with depth format");
    }

    let this = MemNew!(TexCube);
    (*this)._refCount = 1;

    gl_gen_textures(1, &mut (*this).handle);

    (*this).size = size;
    (*this).format = format;

    gl_bind_texture(gl::TEXTURE_CUBE_MAP, (*this).handle);
    gl_tex_image2d(
        gl::TEXTURE_CUBE_MAP_POSITIVE_X,
        0,
        format,
        size,
        size,
        0,
        gl::RED,
        gl::BYTE,
        std::ptr::null(),
    );
    gl_tex_image2d(
        gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
        0,
        format,
        size,
        size,
        0,
        gl::RED,
        gl::BYTE,
        std::ptr::null(),
    );
    gl_tex_image2d(
        gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
        0,
        format,
        size,
        size,
        0,
        gl::RED,
        gl::BYTE,
        std::ptr::null(),
    );
    gl_tex_image2d(
        gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
        0,
        format,
        size,
        size,
        0,
        gl::RED,
        gl::BYTE,
        std::ptr::null(),
    );
    gl_tex_image2d(
        gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
        0,
        format,
        size,
        size,
        0,
        gl::RED,
        gl::BYTE,
        std::ptr::null(),
    );
    gl_tex_image2d(
        gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        0,
        format,
        size,
        size,
        0,
        gl::RED,
        gl::BYTE,
        std::ptr::null(),
    );
    TexCube_InitParameters();
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);

    this
}

#[no_mangle]
pub extern "C" fn TexCube_Acquire(this: &mut TexCube) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Clear(this: &mut TexCube, r: f32, g: f32, b: f32, a: f32) {
    for i in 0..6 {
        let face: Face = kFaces[i as usize];

        RenderTarget_Push((*this).size, (*this).size);
        RenderTarget_BindTexCube(this, face.face);
        Draw_Clear(r, g, b, a);
        RenderTarget_Pop();
    }
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Free(this: *mut TexCube) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        gl_delete_textures(1, &mut (*this).handle);
        MemFree(this as *const _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Load(path: *const libc::c_char) -> *mut TexCube {
    let this = MemNew!(TexCube);
    gl_gen_textures(1, &mut (*this).handle);
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, (*this).handle);

    let mut components: i32 = 0;
    let mut dataLayout: i32 = 0;

    for i in 0..6 {
        let face_path = format!("{}{}.jpg", path.as_str(), K_FACE_EXT[i as usize]);
        let mut sx: i32 = 0;
        let mut sy: i32 = 0;
        let mut lcomponents: i32 = 0;
        let data: *mut libc::c_uchar =
            tex2d_load_raw(&face_path, &mut sx, &mut sy, &mut lcomponents);

        if data.is_null() {
            panic!("TexCube_Load failed to load cubemap face from '{face_path}'",);
        }

        if sx != sy {
            panic!("TexCube_Load loaded cubemap face is not square");
        }

        if i != 0 {
            if sx != (*this).size || sy != (*this).size {
                panic!("TexCube_Load loaded cubemap faces have different resolutions");
            }

            if lcomponents != components {
                panic!("TexCube_Load loaded cubemap faces have different number of components");
            }
        } else {
            components = lcomponents;
            (*this).size = sx;
            (*this).format = if components == 4 {
                TexFormat_RGBA8
            } else if components == 3 {
                TexFormat_RGB8
            } else if components == 2 {
                TexFormat_RG8
            } else {
                TexFormat_R8
            };

            dataLayout = if components == 4 {
                gl::RGBA
            } else if components == 3 {
                gl::RGB
            } else if components == 2 {
                gl::RG
            } else {
                gl::RED
            } as i32;
        }

        gl_tex_image2d(
            kFaces[i as usize].face as gl::types::GLenum,
            0,
            (*this).format,
            (*this).size,
            (*this).size,
            0,
            dataLayout as gl::types::GLenum,
            gl::UNSIGNED_BYTE,
            data as *const _,
        );

        MemFree(data as *const _);
    }

    TexCube_InitParameters();

    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);

    this
}

#[no_mangle]
pub extern "C" fn TexCube_GetData(
    this: &mut TexCube,
    data: *mut libc::c_void,
    face: CubeFace,
    level: i32,
    pf: PixelFormat,
    df: DataFormat,
) {
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, this.handle);
    gl_get_tex_image(
        face as gl::types::GLenum,
        level,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    );
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GetDataBytes(
    this: &mut TexCube,
    face: CubeFace,
    level: i32,
    pf: PixelFormat,
    df: DataFormat,
) -> *mut Bytes {
    let mut size: i32 = this.size * this.size;
    size *= DataFormat_GetSize(df);
    size *= PixelFormat_Components(pf);

    let data: *mut Bytes = Bytes_Create(size as u32);
    TexCube_GetData(this, Bytes_GetData(&mut *data), face, level, pf, df);
    Bytes_Rewind(&mut *data);

    data
}

#[no_mangle]
pub extern "C" fn TexCube_GetFormat(this: &mut TexCube) -> TexFormat {
    this.format
}

#[no_mangle]
pub extern "C" fn TexCube_GetHandle(this: &mut TexCube) -> u32 {
    this.handle
}

#[no_mangle]
pub extern "C" fn TexCube_GetSize(this: &mut TexCube) -> i32 {
    this.size
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Generate(this: &mut TexCube, state: &mut ShaderState) {
    GLMatrix_ModeP();
    GLMatrix_Push();
    GLMatrix_Clear();
    GLMatrix_ModeWV();
    GLMatrix_Push();
    GLMatrix_Clear();
    RenderState_PushAllDefaults();
    ShaderState_Start(state);

    for i in 0..6 {
        let face: Face = kFaces[i as usize];
        let size: i32 = this.size;
        let fSize: f32 = this.size as f32;

        RenderTarget_Push(size, size);
        RenderTarget_BindTexCube(this, face.face);
        Draw_Clear(0.0f32, 0.0f32, 0.0f32, 1.0f32);
        Shader_SetFloat3(c_str!("cubeLook"), face.look.x, face.look.y, face.look.z);
        Shader_SetFloat3(c_str!("cubeUp"), face.up.x, face.up.y, face.up.z);
        Shader_SetFloat(c_str!("cubeSize"), fSize);

        let mut j: i32 = 1;
        let mut jobSize: i32 = 1;
        while j <= size {
            let time: TimeStamp = TimeStamp::now();
            ClipRect_Push(0.0f32, (j - 1) as f32, size as f32, jobSize as f32);
            Draw_Rect(0.0f32, 0.0f32, fSize, fSize);
            Draw_Flush();
            ClipRect_Pop();

            j += jobSize;
            let elapsed = time.get_elapsed();

            jobSize = f64::max(
                1.0,
                f64::floor(0.25f64 * jobSize as f64 / elapsed + 0.5f64) as i32 as f64,
            ) as i32;
            jobSize = i32::min(jobSize, size - j + 1);
        }

        RenderTarget_Pop();
    }

    ShaderState_Stop(state);
    RenderState_PopAll();
    GLMatrix_ModeP();
    GLMatrix_Pop();
    GLMatrix_ModeWV();
    GLMatrix_Pop();
}

#[no_mangle]
pub extern "C" fn TexCube_GenMipmap(this: &mut TexCube) {
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, this.handle);
    gl_generate_mipmap(gl::TEXTURE_CUBE_MAP);
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);
}

#[no_mangle]
pub extern "C" fn TexCube_SetData(
    this: &mut TexCube,
    data: *const libc::c_void,
    face: CubeFace,
    level: i32,
    pf: PixelFormat,
    df: DataFormat,
) {
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, this.handle);
    gl_tex_image2d(
        face as gl::types::GLenum,
        level,
        this.format,
        this.size,
        this.size,
        0,
        pf as gl::types::GLenum,
        df as gl::types::GLenum,
        data,
    );
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SetDataBytes(
    this: &mut TexCube,
    data: &mut Bytes,
    face: CubeFace,
    level: i32,
    pf: PixelFormat,
    df: DataFormat,
) {
    TexCube_SetData(this, Bytes_GetData(data), face, level, pf, df);
}

#[no_mangle]
pub extern "C" fn TexCube_SetMagFilter(this: &mut TexCube, filter: TexFilter) {
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, this.handle);
    gl_tex_parameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, filter);
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);
}

#[no_mangle]
pub extern "C" fn TexCube_SetMinFilter(this: &mut TexCube, filter: TexFilter) {
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, this.handle);
    gl_tex_parameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, filter);
    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_Save(this: &mut TexCube, path: *const libc::c_char) {
    TexCube_SaveLevel(this, path, 0);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_SaveLevel(
    this: &mut TexCube,
    path: *const libc::c_char,
    level: i32,
) {
    let size: i32 = this.size >> level;

    gl_bind_texture(gl::TEXTURE_CUBE_MAP, this.handle);

    let buffer: *mut libc::c_uchar =
        MemAlloc((std::mem::size_of::<libc::c_uchar>()).wrapping_mul((4 * size * size) as usize))
            as *mut libc::c_uchar;

    for i in 0..6 {
        let face: CubeFace = kFaces[i as usize].face;
        let face_path = format!("{}{}.png", path.as_str(), K_FACE_EXT[i as usize]);

        gl_get_tex_image(
            face as gl::types::GLenum,
            level,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            buffer as *mut _,
        );

        tex2d_save_png(&face_path, size, size, 4, buffer);
    }

    MemFree(buffer as *const _);

    gl_bind_texture(gl::TEXTURE_CUBE_MAP, 0);
}
