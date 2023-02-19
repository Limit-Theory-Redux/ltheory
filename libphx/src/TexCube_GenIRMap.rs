use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::CubeFace::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
use crate::TexFilter::*;
use glam::Vec2;

extern "C" {
    pub type RNG;
    pub type Shader;
    pub type Tex2D;
    pub type TexCube;
    fn CubeFace_Get(index: libc::c_int) -> CubeFace;
    fn atan2(_: libc::c_double, _: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Draw_Rect(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn RenderTarget_Push(sx: libc::c_int, sy: libc::c_int);
    fn RenderTarget_Pop();
    fn RenderTarget_BindTexCubeLevel(_: *mut TexCube, _: CubeFace, level: libc::c_int);
    fn RNG_FromTime() -> *mut RNG;
    fn RNG_Free(_: *mut RNG);
    fn RNG_GetUniform(_: *mut RNG) -> libc::c_double;
    fn Shader_Load(vertName: cstr, fragName: cstr) -> *mut Shader;
    fn Shader_Start(_: *mut Shader);
    fn Shader_Stop(_: *mut Shader);
    fn Shader_ResetTexIndex();
    fn Shader_SetFloat(_: cstr, _: libc::c_float);
    fn Shader_SetFloat3(_: cstr, _: libc::c_float, _: libc::c_float, _: libc::c_float);
    fn Shader_SetInt(_: cstr, _: libc::c_int);
    fn Shader_SetTex2D(_: cstr, _: *mut Tex2D);
    fn Shader_SetTexCube(_: cstr, _: *mut TexCube);
    fn Tex2D_Create(sx: libc::c_int, sy: libc::c_int, _: TexFormat) -> *mut Tex2D;
    fn Tex2D_Free(_: *mut Tex2D);
    fn Tex2D_SetData(
        _: *mut Tex2D,
        _: *const libc::c_void,
        _: PixelFormat,
        _: DataFormat,
    );
    fn TexCube_Create(size: libc::c_int, _: TexFormat) -> *mut TexCube;
    fn TexCube_GenMipmap(_: *mut TexCube);
    fn TexCube_GetData(
        _: *mut TexCube,
        _: *mut libc::c_void,
        _: CubeFace,
        level: libc::c_int,
        _: PixelFormat,
        _: DataFormat,
    );
    fn TexCube_GetFormat(_: *mut TexCube) -> TexFormat;
    fn TexCube_GetSize(_: *mut TexCube) -> libc::c_int;
    fn TexCube_SetData(
        _: *mut TexCube,
        _: *const libc::c_void,
        _: CubeFace,
        level: libc::c_int,
        _: PixelFormat,
        _: DataFormat,
    );
    fn TexCube_SetMagFilter(_: *mut TexCube, _: TexFilter);
    fn TexCube_SetMinFilter(_: *mut TexCube, _: TexFilter);
    fn TexFormat_Components(_: TexFormat) -> libc::c_int;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;

pub type CubeFace = int32;
pub type DataFormat = int32;
pub type PixelFormat = int32;
pub type TexFilter = int32;
pub type TexFormat = int32;
#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}
#[inline]
unsafe extern "C" fn Atan2(
    mut y: libc::c_double,
    mut x: libc::c_double,
) -> libc::c_double {
    return atan2(y, x);
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GenIRMap(
    mut self_0: *mut TexCube,
    mut sampleCount: libc::c_int,
) -> *mut TexCube {
    let mut size: libc::c_int = TexCube_GetSize(self_0);
    let mut format: TexFormat = TexCube_GetFormat(self_0);
    let mut result: *mut TexCube = TexCube_Create(size, format);
    let mut components: libc::c_int = TexFormat_Components(format);
    let mut pf: PixelFormat = if components == 4 as libc::c_int {
        PixelFormat_RGBA
    } else if components == 3 as libc::c_int {
        PixelFormat_RGB
    } else if components == 2 as libc::c_int {
        PixelFormat_RG
    } else {
        PixelFormat_Red
    };
    let mut df: DataFormat = DataFormat_Float;
    let mut buffer: *mut libc::c_void = MemAlloc(
        ((size * size) as usize).wrapping_mul(::core::mem::size_of::<libc::c_float>())
            .wrapping_mul(components as usize),
    );
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        TexCube_GetData(self_0, buffer, CubeFace_Get(i), 0 as libc::c_int, pf, df);
        TexCube_SetData(result, buffer, CubeFace_Get(i), 0 as libc::c_int, pf, df);
        i += 1;
    }
    TexCube_GenMipmap(result);
    MemFree(buffer);
    static mut shader: *mut Shader = 0 as *const Shader as *mut Shader;
    if shader.is_null() {
        shader = Shader_Load(
            b"vertex/identity\0" as *const u8 as *const libc::c_char,
            b"fragment/compute/irmap\0" as *const u8 as *const libc::c_char,
        );
    }
    let face: [CubeFace; 6] = [
        CubeFace_PX,
        CubeFace_NX,
        CubeFace_PY,
        CubeFace_NY,
        CubeFace_PZ,
        CubeFace_NZ,
    ];
    let look: [Vec3; 6] = [
        {
            let mut init = Vec3 {
                x: 1 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: -(1 as libc::c_int) as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
    ];
    let up: [Vec3; 6] = [
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
    ];
    let mut rng: *mut RNG = RNG_FromTime();
    let mut levels: libc::c_int = 0 as libc::c_int;
    let mut i_0: libc::c_int = size;
    while i_0 > 0 as libc::c_int {
        levels += 1;
        i_0 /= 2 as libc::c_int;
    }
    Shader_Start(shader);
    let mut level: libc::c_int = 0 as libc::c_int;
    while size > 1 as libc::c_int {
        size /= 2 as libc::c_int;
        level += 1 as libc::c_int;
        let mut ggxWidth: libc::c_double = level as libc::c_double
            / levels as libc::c_double;
        ggxWidth *= ggxWidth;
        let mut sampleBuffer: *mut Vec2 = MemAlloc(
            (::core::mem::size_of::<Vec2>())
                .wrapping_mul(sampleCount as usize),
        ) as *mut Vec2;
        let mut sampleTex: *mut Tex2D = Tex2D_Create(
            sampleCount,
            1 as libc::c_int,
            TexFormat_RG16F,
        );
        let mut i_1: libc::c_int = 0 as libc::c_int;
        while i_1 < sampleCount {
            let mut e1: libc::c_double = RNG_GetUniform(rng);
            let mut e2: libc::c_double = RNG_GetUniform(rng);
            let mut pitch: libc::c_double = Atan2(
                ggxWidth * Sqrt(e1),
                Sqrt(1.0f64 - e1),
            );
            let mut yaw: libc::c_double = 6.28318531f32 as libc::c_double * e2;
            *sampleBuffer
                .offset(
                    i_1 as isize,
                ) = Vec2::new(pitch as libc::c_float, yaw as libc::c_float);
            i_1 += 1;
        }
        Tex2D_SetData(
            sampleTex,
            sampleBuffer as *const libc::c_void,
            PixelFormat_RG,
            DataFormat_Float,
        );
        let mut angle: libc::c_float = level as libc::c_float
            / (levels - 1 as libc::c_int) as libc::c_float;
        angle = angle * angle;
        Shader_ResetTexIndex();
        Shader_SetFloat(b"angle\0" as *const u8 as *const libc::c_char, angle);
        Shader_SetTexCube(b"src\0" as *const u8 as *const libc::c_char, self_0);
        Shader_SetTex2D(
            b"sampleBuffer\0" as *const u8 as *const libc::c_char,
            sampleTex,
        );
        Shader_SetInt(b"samples\0" as *const u8 as *const libc::c_char, sampleCount);
        let mut i_2: libc::c_int = 0 as libc::c_int;
        while i_2 < 6 as libc::c_int {
            let mut thisFace: CubeFace = face[i_2 as usize];
            let mut thisLook: Vec3 = look[i_2 as usize];
            let mut thisUp: Vec3 = up[i_2 as usize];
            RenderTarget_Push(size, size);
            RenderTarget_BindTexCubeLevel(result, thisFace, level);
            Shader_SetFloat3(
                b"cubeLook\0" as *const u8 as *const libc::c_char,
                thisLook.x,
                thisLook.y,
                thisLook.z,
            );
            Shader_SetFloat3(
                b"cubeUp\0" as *const u8 as *const libc::c_char,
                thisUp.x,
                thisUp.y,
                thisUp.z,
            );
            Draw_Rect(
                -(1 as libc::c_int) as libc::c_float,
                -(1 as libc::c_int) as libc::c_float,
                2 as libc::c_int as libc::c_float,
                2 as libc::c_int as libc::c_float,
            );
            RenderTarget_Pop();
            i_2 += 1;
        }
        MemFree(sampleBuffer as *const libc::c_void);
        Tex2D_Free(sampleTex);
    }
    RNG_Free(rng);
    Shader_Stop(shader);
    TexCube_SetMagFilter(result, TexFilter_Linear);
    TexCube_SetMinFilter(result, TexFilter_LinearMipLinear);
    return result;
}
