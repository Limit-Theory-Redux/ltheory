use crate::internal::Memory::*;
use crate::CubeFace::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFilter::*;
use crate::TexFormat::*;
use crate::Shader::*;
use crate::TexCube::*;
use crate::Tex2D::*;
use crate::RenderTarget::*;
use crate::Draw::*;
use crate::RNG::*;
use glam::Vec2;
use glam::Vec3;
use libc;

extern "C" {
    fn atan2(_: f64, _: f64) -> f64;
    fn sqrt(_: f64) -> f64;
}

pub type CubeFace = i32;
pub type DataFormat = i32;
pub type PixelFormat = i32;
pub type TexFilter = i32;
pub type TexFormat = i32;

#[inline]
unsafe extern "C" fn Sqrt(mut t: f64) -> f64 {
    sqrt(t)
}

#[inline]
unsafe extern "C" fn Atan2(mut y: f64, mut x: f64) -> f64 {
    atan2(y, x)
}

#[no_mangle]
pub unsafe extern "C" fn TexCube_GenIRMap(
    mut this: *mut TexCube,
    mut sampleCount: i32,
) -> *mut TexCube {
    let mut size: i32 = TexCube_GetSize(this);
    let mut format: TexFormat = TexCube_GetFormat(this);
    let mut result: *mut TexCube = TexCube_Create(size, format);
    let mut components: i32 = TexFormat_Components(format);
    let mut pf: PixelFormat = if components == 4_i32 {
        PixelFormat_RGBA
    } else if components == 3_i32 {
        PixelFormat_RGB
    } else if components == 2_i32 {
        PixelFormat_RG
    } else {
        PixelFormat_Red
    };
    let mut df: DataFormat = DataFormat_Float;
    let mut buffer: *mut libc::c_void = MemAlloc(
        ((size * size) as usize)
            .wrapping_mul(::core::mem::size_of::<f32>())
            .wrapping_mul(components as usize),
    );
    let mut i: i32 = 0_i32;
    while i < 6_i32 {
        TexCube_GetData(this, buffer, CubeFace_Get(i), 0_i32, pf, df);
        TexCube_SetData(result, buffer, CubeFace_Get(i), 0_i32, pf, df);
        i += 1;
    }
    TexCube_GenMipmap(result);
    MemFree(buffer);
    static mut shader: *mut Shader = std::ptr::null_mut();
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
        Vec3::new(1.0f32, 0.0f32, 0.0f32),
        Vec3::new(-1.0f32, 0.0f32, 0.0f32),
        Vec3::new(0.0f32, 1.0f32, 0.0f32),
        Vec3::new(0.0f32, -1.0f32, 0.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 0.0f32, -1.0f32),
    ];
    let up: [Vec3; 6] = [
        Vec3::new(0.0f32, 1.0f32, 0.0f32),
        Vec3::new(0.0f32, 1.0f32, 0.0f32),
        Vec3::new(0.0f32, 0.0f32, -1.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 1.0f32, 0.0f32),
        Vec3::new(0.0f32, 1.0f32, 0.0f32),
    ];
    let mut rng: *mut RNG = RNG_FromTime();
    let mut levels: i32 = 0_i32;
    let mut i_0: i32 = size;
    while i_0 > 0_i32 {
        levels += 1;
        i_0 /= 2_i32;
    }
    Shader_Start(shader);
    let mut level: i32 = 0_i32;
    while size > 1_i32 {
        size /= 2_i32;
        level += 1_i32;
        let mut ggxWidth: f64 = level as f64 / levels as f64;
        ggxWidth *= ggxWidth;
        let mut sampleBuffer: *mut Vec2 =
            MemAlloc((::core::mem::size_of::<Vec2>()).wrapping_mul(sampleCount as usize))
                as *mut Vec2;
        let mut sampleTex: *mut Tex2D = Tex2D_Create(sampleCount, 1_i32, TexFormat_RG16F);
        let mut i_1: i32 = 0_i32;
        while i_1 < sampleCount {
            let mut e1: f64 = RNG_GetUniform(rng);
            let mut e2: f64 = RNG_GetUniform(rng);
            let mut pitch: f64 = Atan2(ggxWidth * Sqrt(e1), Sqrt(1.0f64 - e1));
            let mut yaw: f64 = 6.28318531f32 as f64 * e2;
            *sampleBuffer.offset(i_1 as isize) = Vec2::new(pitch as f32, yaw as f32);
            i_1 += 1;
        }
        Tex2D_SetData(
            sampleTex,
            sampleBuffer as *const libc::c_void,
            PixelFormat_RG,
            DataFormat_Float,
        );
        let mut angle: f32 = level as f32 / (levels - 1_i32) as f32;
        angle = angle * angle;
        Shader_ResetTexIndex();
        Shader_SetFloat(b"angle\0" as *const u8 as *const libc::c_char, angle);
        Shader_SetTexCube(b"src\0" as *const u8 as *const libc::c_char, this);
        Shader_SetTex2D(
            b"sampleBuffer\0" as *const u8 as *const libc::c_char,
            sampleTex,
        );
        Shader_SetInt(
            b"samples\0" as *const u8 as *const libc::c_char,
            sampleCount,
        );
        let mut i_2: i32 = 0_i32;
        while i_2 < 6_i32 {
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
            Draw_Rect(-1.0f32, -1.0f32, 2.0f32, 2.0f32);
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
    result
}