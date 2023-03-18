use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::Draw::*;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Math::Vec4;
use crate::Mesh::*;
use crate::PixelFormat::*;
use crate::RenderState::*;
use crate::RenderTarget::*;
use crate::Shader::*;
use crate::Tex2D::*;
use crate::Tex3D::*;
use crate::TexFormat::*;
use libc;

pub type DataFormat = i32;
pub type PixelFormat = i32;
pub type TexFormat = i32;

#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeAO(mut this: *mut Mesh, mut radius: f32) {
    let mut indexCount: i32 = Mesh_GetIndexCount(this);
    let mut vertexCount: i32 = Mesh_GetVertexCount(this);
    let mut indexData: *mut i32 = Mesh_GetIndexData(this);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(this);
    let mut sDim: i32 = f64::ceil(f64::sqrt((indexCount / 3_i32) as f64)) as i32;
    let mut vDim: i32 = f64::ceil(f64::sqrt(vertexCount as f64)) as i32;
    let mut surfels: i32 = sDim * sDim;
    let mut vertices: i32 = vDim * vDim;
    let mut bufSize: i32 = f64::max(surfels as f64, vertices as f64) as i32;
    let mut pointBuffer: *mut Vec4 =
        MemAlloc((::core::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize)) as *mut Vec4;
    let mut normalBuffer: *mut Vec4 =
        MemAlloc((::core::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize)) as *mut Vec4;
    MemZero(
        pointBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    MemZero(
        normalBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    let mut i: i32 = 0_i32;
    while i < indexCount {
        let mut v1: *const Vertex =
            vertexData.offset(*indexData.offset((i + 0_i32) as isize) as isize);
        let mut v2: *const Vertex =
            vertexData.offset(*indexData.offset((i + 1_i32) as isize) as isize);
        let mut v3: *const Vertex =
            vertexData.offset(*indexData.offset((i + 2_i32) as isize) as isize);
        let mut normal: Vec3 = Vec3::cross((*v3).p - (*v1).p, (*v2).p - (*v1).p);
        let mut length: f32 = normal.length();
        let mut area: f32 = 0.5f32 * length / std::f32::consts::PI;
        if f64::abs(length as f64) > 1e-6f64 {
            normal /= length;
        } else {
            normal = Vec3::new(1.0f32, 0.0f32, 0.0f32);
        }
        let mut center: Vec3 = ((*v1).p + (*v2).p + (*v3).p) / 3.0f32;
        *pointBuffer.offset((i / 3_i32) as isize) = Vec4::new(center.x, center.y, center.z, area);
        *normalBuffer.offset((i / 3_i32) as isize) =
            Vec4::new(normal.x, normal.y, normal.z, 0.0f32);
        i += 3_i32;
    }
    let mut texSPoints: *mut Tex2D = Tex2D_Create(sDim, sDim, TexFormat_RGBA32F);
    let mut texSNormals: *mut Tex2D = Tex2D_Create(sDim, sDim, TexFormat_RGBA32F);
    Tex2D_SetData(
        texSPoints,
        pointBuffer as *const libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    Tex2D_SetData(
        texSNormals,
        normalBuffer as *const libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    MemZero(
        pointBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    MemZero(
        normalBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4>()).wrapping_mul(bufSize as usize),
    );
    let mut i_0: i32 = 0_i32;
    while i_0 < vertexCount {
        let mut v: *const Vertex = vertexData.offset(i_0 as isize);
        *pointBuffer.offset(i_0 as isize) = Vec4::new((*v).p.x, (*v).p.y, (*v).p.z, 0.0f32);
        *normalBuffer.offset(i_0 as isize) = Vec4::new((*v).n.x, (*v).n.y, (*v).n.z, 0.0f32);
        i_0 += 1;
    }
    let mut texVPoints: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    let mut texVNormals: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    Tex2D_SetData(
        texVPoints,
        pointBuffer as *const libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    Tex2D_SetData(
        texVNormals,
        normalBuffer as *const libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    MemFree(pointBuffer as *const libc::c_void);
    MemFree(normalBuffer as *const libc::c_void);
    let mut texOutput: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_R32F);
    static mut shader: *mut Shader = std::ptr::null_mut();
    if shader.is_null() {
        shader = Shader_Load(
            b"vertex/identity\0" as *const u8 as *const libc::c_char,
            b"fragment/compute/occlusion\0" as *const u8 as *const libc::c_char,
        );
    }
    RenderState_PushAllDefaults();
    RenderTarget_PushTex2D(texOutput);
    Shader_Start(shader);
    Shader_SetInt(b"sDim\0" as *const u8 as *const libc::c_char, sDim);
    Shader_SetFloat(b"radius\0" as *const u8 as *const libc::c_char, radius);
    Shader_SetTex2D(
        b"sPointBuffer\0" as *const u8 as *const libc::c_char,
        texSPoints,
    );
    Shader_SetTex2D(
        b"sNormalBuffer\0" as *const u8 as *const libc::c_char,
        texSNormals,
    );
    Shader_SetTex2D(
        b"vPointBuffer\0" as *const u8 as *const libc::c_char,
        texVPoints,
    );
    Shader_SetTex2D(
        b"vNormalBuffer\0" as *const u8 as *const libc::c_char,
        texVNormals,
    );
    Draw_Rect(-1.0f32, -1.0f32, 2.0f32, 2.0f32);
    Shader_Stop(shader);
    RenderTarget_Pop();
    RenderState_PopAll();
    let mut result: *mut f32 =
        MemAlloc((::core::mem::size_of::<f32>()).wrapping_mul((vDim * vDim) as usize)) as *mut f32;
    Tex2D_GetData(
        texOutput,
        result as *mut libc::c_void,
        PixelFormat_Red,
        DataFormat_Float,
    );
    let mut i_1: i32 = 0_i32;
    while i_1 < vertexCount {
        (*vertexData.offset(i_1 as isize)).uv.x = *result.offset(i_1 as isize);
        i_1 += 1;
    }
    MemFree(result as *const libc::c_void);
    Tex2D_Free(texOutput);
    Tex2D_Free(texSPoints);
    Tex2D_Free(texSNormals);
    Tex2D_Free(texVPoints);
    Tex2D_Free(texVNormals);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeOcclusion(
    mut this: *mut Mesh,
    mut sdf: *mut Tex3D,
    mut radius: f32,
) {
    let mut vertexCount: i32 = Mesh_GetVertexCount(this);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(this);
    let mut vDim: i32 = f64::ceil(f64::sqrt(vertexCount as f64)) as i32;
    let mut texPoints: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    let mut texOutput: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_R32F);
    let mut pointBuffer: *mut Vec3 =
        MemAlloc((::core::mem::size_of::<Vec3>()).wrapping_mul((vDim * vDim) as usize))
            as *mut Vec3;
    let mut i: i32 = 0_i32;
    while i < vertexCount {
        *pointBuffer.offset(i as isize) = (*vertexData.offset(i as isize)).p;
        i += 1;
    }
    Tex2D_SetData(
        texPoints,
        pointBuffer as *const libc::c_void,
        PixelFormat_RGB,
        DataFormat_Float,
    );
    MemFree(pointBuffer as *const libc::c_void);
    static mut shader: *mut Shader = std::ptr::null_mut();
    if shader.is_null() {
        shader = Shader_Load(
            b"vertex/identity\0" as *const u8 as *const libc::c_char,
            b"fragment/compute/occlusion_sdf\0" as *const u8 as *const libc::c_char,
        );
    }
    RenderState_PushAllDefaults();
    RenderTarget_PushTex2D(texOutput);
    Shader_Start(shader);
    Shader_SetFloat(b"radius\0" as *const u8 as *const libc::c_char, radius);
    Shader_SetTex2D(b"points\0" as *const u8 as *const libc::c_char, texPoints);
    Shader_SetTex3D(b"sdf\0" as *const u8 as *const libc::c_char, sdf);
    Draw_Rect(-1.0f32, -1.0f32, 2.0f32, 2.0f32);
    Shader_Stop(shader);
    RenderTarget_Pop();
    RenderState_PopAll();
    let mut result: *mut f32 =
        MemAlloc((::core::mem::size_of::<f32>()).wrapping_mul((vDim * vDim) as usize)) as *mut f32;
    Tex2D_GetData(
        texOutput,
        result as *mut libc::c_void,
        PixelFormat_Red,
        DataFormat_Float,
    );
    let mut i_0: i32 = 0_i32;
    while i_0 < vertexCount {
        (*vertexData.offset(i_0 as isize)).uv.x = *result.offset(i_0 as isize);
        i_0 += 1;
    }
    MemFree(result as *const libc::c_void);
    Tex2D_Free(texPoints);
    Tex2D_Free(texOutput);
}
