use internal::*;

use super::*;
use crate::common::*;
use crate::math::*;
use crate::*;

#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeAO(this: &mut Mesh, radius: f32) {
    let indexCount: i32 = Mesh_GetIndexCount(this);
    let vertexCount: i32 = Mesh_GetVertexCount(this);
    let indexData: *mut i32 = Mesh_GetIndexData(this);
    let vertexData: *mut Vertex = Mesh_GetVertexData(this);
    let sDim: i32 = f64::ceil(f64::sqrt((indexCount / 3) as f64)) as i32;
    let vDim: i32 = f64::ceil(f64::sqrt(vertexCount as f64)) as i32;
    let surfels: i32 = sDim * sDim;
    let vertices: i32 = vDim * vDim;
    let bufSize: i32 = i32::max(surfels, vertices);
    let pointBuffer: *mut Vec4 = MemNewArray!(Vec4, bufSize);
    let normalBuffer: *mut Vec4 = MemNewArray!(Vec4, bufSize);
    MemZero(
        pointBuffer as *mut _,
        (std::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    MemZero(
        normalBuffer as *mut _,
        (std::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    let mut i: i32 = 0;
    while i < indexCount {
        let v1: *const Vertex = vertexData.offset(*indexData.offset((i + 0) as isize) as isize);
        let v2: *const Vertex = vertexData.offset(*indexData.offset((i + 1) as isize) as isize);
        let v3: *const Vertex = vertexData.offset(*indexData.offset((i + 2) as isize) as isize);
        let mut normal: Vec3 = Vec3::cross((*v3).p - (*v1).p, (*v2).p - (*v1).p);
        let length: f32 = normal.length();
        let area: f32 = 0.5f32 * length / std::f32::consts::PI;
        if f64::abs(length as f64) > 1e-6f64 {
            normal /= length;
        } else {
            normal = Vec3::X;
        }
        let center: Vec3 = ((*v1).p + (*v2).p + (*v3).p) / 3.0f32;
        *pointBuffer.offset((i / 3) as isize) = Vec4::new(center.x, center.y, center.z, area);
        *normalBuffer.offset((i / 3) as isize) = Vec4::new(normal.x, normal.y, normal.z, 0.0f32);
        i += 3;
    }
    let texSPoints: *mut Tex2D = Tex2D_Create(sDim, sDim, TexFormat_RGBA32F);
    let texSNormals: *mut Tex2D = Tex2D_Create(sDim, sDim, TexFormat_RGBA32F);
    Tex2D_SetData(
        &mut *texSPoints,
        pointBuffer as *const _,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    Tex2D_SetData(
        &mut *texSNormals,
        normalBuffer as *const _,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    MemZero(
        pointBuffer as *mut _,
        (std::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    MemZero(
        normalBuffer as *mut _,
        (std::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    let mut i_0: i32 = 0;
    while i_0 < vertexCount {
        let v: *const Vertex = vertexData.offset(i_0 as isize);
        *pointBuffer.offset(i_0 as isize) = Vec4::new((*v).p.x, (*v).p.y, (*v).p.z, 0.0f32);
        *normalBuffer.offset(i_0 as isize) = Vec4::new((*v).n.x, (*v).n.y, (*v).n.z, 0.0f32);
        i_0 += 1;
    }
    let texVPoints: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    let texVNormals: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    Tex2D_SetData(
        &mut *texVPoints,
        pointBuffer as *const _,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    Tex2D_SetData(
        &mut *texVNormals,
        normalBuffer as *const _,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    MemFree(pointBuffer as *const _);
    MemFree(normalBuffer as *const _);
    let texOutput: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_R32F);
    // TODO: Store shader properly
    static mut shader: *mut Shader = std::ptr::null_mut();
    if shader.is_null() {
        shader = Box::into_raw(Shader_Load(
            c_str!("vertex/identity"),
            c_str!("fragment/compute/occlusion"),
        ));
    }
    RenderState_PushAllDefaults();
    RenderTarget_PushTex2D(&mut *texOutput);
    Shader_Start(&mut *shader);
    Shader_SetInt(c_str!("sDim"), sDim);
    Shader_SetFloat(c_str!("radius"), radius);
    Shader_SetTex2D(c_str!("sPointBuffer"), &mut *texSPoints);
    Shader_SetTex2D(c_str!("sNormalBuffer"), &mut *texSNormals);
    Shader_SetTex2D(c_str!("vPointBuffer"), &mut *texVPoints);
    Shader_SetTex2D(c_str!("vNormalBuffer"), &mut *texVNormals);
    Draw_Rect(-1.0f32, -1.0f32, 2.0f32, 2.0f32);
    Shader_Stop(shader);
    RenderTarget_Pop();
    RenderState_PopAll();
    let result: *mut f32 = MemNewArray!(f32, (vDim * vDim));
    Tex2D_GetData(
        &mut *texOutput,
        result as *mut _,
        PixelFormat_Red,
        DataFormat_Float,
    );
    let mut i_1: i32 = 0;
    while i_1 < vertexCount {
        (*vertexData.offset(i_1 as isize)).uv.x = *result.offset(i_1 as isize);
        i_1 += 1;
    }
    MemFree(result as *const _);
    Tex2D_Free(&mut *texOutput);
    Tex2D_Free(&mut *texSPoints);
    Tex2D_Free(&mut *texSNormals);
    Tex2D_Free(&mut *texVPoints);
    Tex2D_Free(&mut *texVNormals);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeOcclusion(this: &mut Mesh, sdf: *mut Tex3D, radius: f32) {
    let vertexCount: i32 = Mesh_GetVertexCount(this);
    let vertexData: *mut Vertex = Mesh_GetVertexData(this);
    let vDim: i32 = f64::ceil(f64::sqrt(vertexCount as f64)) as i32;
    let texPoints: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    let texOutput: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_R32F);
    let pointBuffer: *mut Vec3 = MemNewArray!(Vec3, (vDim * vDim));
    let mut i: i32 = 0;
    while i < vertexCount {
        *pointBuffer.offset(i as isize) = (*vertexData.offset(i as isize)).p;
        i += 1;
    }
    Tex2D_SetData(
        &mut *texPoints,
        pointBuffer as *const _,
        PixelFormat_RGB,
        DataFormat_Float,
    );
    MemFree(pointBuffer as *const _);
    // TODO: Store shader properly.
    static mut shader: *mut Shader = std::ptr::null_mut();
    if shader.is_null() {
        shader = Box::into_raw(Shader_Load(
            c_str!("vertex/identity"),
            c_str!("fragment/compute/occlusion_sdf"),
        ));
    }
    RenderState_PushAllDefaults();
    RenderTarget_PushTex2D(&mut *texOutput);
    Shader_Start(&mut *shader);
    Shader_SetFloat(c_str!("radius"), radius);
    Shader_SetTex2D(c_str!("points"), &mut *texPoints);
    Shader_SetTex3D(c_str!("sdf"), &mut *sdf);
    Draw_Rect(-1.0f32, -1.0f32, 2.0f32, 2.0f32);
    Shader_Stop(shader);
    RenderTarget_Pop();
    RenderState_PopAll();
    let result: *mut f32 = MemNewArray!(f32, (vDim * vDim));
    Tex2D_GetData(
        &mut *texOutput,
        result as *mut _,
        PixelFormat_Red,
        DataFormat_Float,
    );
    let mut i_0: i32 = 0;
    while i_0 < vertexCount {
        (*vertexData.offset(i_0 as isize)).uv.x = *result.offset(i_0 as isize);
        i_0 += 1;
    }
    MemFree(result as *const _);
    Tex2D_Free(&mut *texPoints);
    Tex2D_Free(&mut *texOutput);
}
