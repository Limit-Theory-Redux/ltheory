use internal::*;

use super::*;
use crate::common::*;
use crate::math::*;
use crate::*;

#[no_mangle]
pub unsafe extern "C" fn TexCube_GenIRMap(this: &mut TexCube, sampleCount: i32) -> *mut TexCube {
    let mut size: i32 = TexCube_GetSize(this);
    let format: TexFormat = TexCube_GetFormat(this);
    let result: *mut TexCube = TexCube_Create(size, format);
    let components: i32 = TexFormat_Components(format);
    let pf: PixelFormat = if components == 4 {
        PixelFormat_RGBA
    } else if components == 3 {
        PixelFormat_RGB
    } else if components == 2 {
        PixelFormat_RG
    } else {
        PixelFormat_Red
    };
    let df: DataFormat = DataFormat_Float;
    let buffer: *mut libc::c_void = MemAlloc(
        ((size * size) as usize)
            .wrapping_mul(std::mem::size_of::<f32>())
            .wrapping_mul(components as usize),
    );
    let mut i: i32 = 0;
    while i < 6 {
        TexCube_GetData(this, buffer, CubeFace_Get(i), 0, pf, df);
        TexCube_SetData(&mut *result, buffer, CubeFace_Get(i), 0, pf, df);
        i += 1;
    }
    TexCube_GenMipmap(&mut *result);
    MemFree(buffer);
    // TODO: Store the shader somewhere and use the Box correctly.
    static mut shader: *mut Shader = std::ptr::null_mut();
    if shader.is_null() {
        shader = Box::into_raw(Shader_Load(
            c_str!("vertex/identity"),
            c_str!("fragment/compute/irmap"),
        ));
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
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::new(0.0f32, 0.0f32, -1.0f32),
    ];
    let up: [Vec3; 6] = [
        Vec3::Y,
        Vec3::Y,
        Vec3::new(0.0f32, 0.0f32, -1.0f32),
        Vec3::Z,
        Vec3::Y,
        Vec3::Y,
    ];
    let mut rng: Box<Rng> = RNG_FromTime();
    let mut levels: i32 = 0;
    let mut i_0: i32 = size;
    while i_0 > 0 {
        levels += 1;
        i_0 /= 2;
    }
    Shader_Start(&mut *shader);
    let mut level: i32 = 0;
    while size > 1 {
        size /= 2;
        level += 1;
        let mut ggxWidth: f64 = level as f64 / levels as f64;
        ggxWidth *= ggxWidth;
        let sampleBuffer = MemNewArray!(Vec2, sampleCount);
        let sampleTex = Tex2D_Create(sampleCount, 1, TexFormat_RG16F);
        let mut i_1: i32 = 0;
        while i_1 < sampleCount {
            let e1: f64 = RNG_GetUniform(rng.as_mut());
            let e2: f64 = RNG_GetUniform(rng.as_mut());
            let pitch: f64 = f64::atan2(ggxWidth * f64::sqrt(e1), f64::sqrt(1.0f64 - e1));
            let yaw: f64 = std::f64::consts::TAU * e2;
            *sampleBuffer.offset(i_1 as isize) = Vec2::new(pitch as f32, yaw as f32);
            i_1 += 1;
        }
        Tex2D_SetData(
            &mut *sampleTex,
            sampleBuffer as *const _,
            PixelFormat_RG,
            DataFormat_Float,
        );
        let mut angle: f32 = level as f32 / (levels - 1) as f32;
        angle = angle * angle;
        Shader_ResetTexIndex();
        Shader_SetFloat(c_str!("angle"), angle);
        Shader_SetTexCube(c_str!("src"), this);
        Shader_SetTex2D(c_str!("sampleBuffer"), &mut *sampleTex);
        Shader_SetInt(c_str!("samples"), sampleCount);
        let mut i_2: i32 = 0;
        while i_2 < 6 {
            let thisFace: CubeFace = face[i_2 as usize];
            let thisLook: Vec3 = look[i_2 as usize];
            let thisUp: Vec3 = up[i_2 as usize];
            RenderTarget_Push(size, size);
            RenderTarget_BindTexCubeLevel(&mut *result, thisFace, level);
            Shader_SetFloat3(c_str!("cubeLook"), thisLook.x, thisLook.y, thisLook.z);
            Shader_SetFloat3(c_str!("cubeUp"), thisUp.x, thisUp.y, thisUp.z);
            Draw_Rect(-1.0f32, -1.0f32, 2.0f32, 2.0f32);
            RenderTarget_Pop();
            i_2 += 1;
        }
        MemFree(sampleBuffer as *const _);
        Tex2D_Free(sampleTex);
    }
    Shader_Stop(shader);
    TexCube_SetMagFilter(&mut *result, TexFilter_Linear);
    TexCube_SetMinFilter(&mut *result, TexFilter_LinearMipLinear);
    result
}
