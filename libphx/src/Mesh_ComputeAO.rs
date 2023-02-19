use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
use glam::Vec2;

extern "C" {
    pub type Mesh;
    pub type Shader;
    pub type Tex2D;
    pub type Tex3D;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn ceil(_: libc::c_double) -> libc::c_double;
    fn Draw_Rect(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn Mesh_GetIndexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetIndexData(_: *mut Mesh) -> *mut libc::c_int;
    fn Mesh_GetVertexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
    fn RenderState_PushAllDefaults();
    fn RenderState_PopAll();
    fn RenderTarget_Pop();
    fn RenderTarget_PushTex2D(_: *mut Tex2D);
    fn Shader_Load(vertName: cstr, fragName: cstr) -> *mut Shader;
    fn Shader_Start(_: *mut Shader);
    fn Shader_Stop(_: *mut Shader);
    fn Shader_SetFloat(_: cstr, _: libc::c_float);
    fn Shader_SetInt(_: cstr, _: libc::c_int);
    fn Shader_SetTex2D(_: cstr, _: *mut Tex2D);
    fn Shader_SetTex3D(_: cstr, _: *mut Tex3D);
    fn Tex2D_Create(sx: libc::c_int, sy: libc::c_int, _: TexFormat) -> *mut Tex2D;
    fn Tex2D_Free(_: *mut Tex2D);
    fn Tex2D_GetData(_: *mut Tex2D, _: *mut libc::c_void, _: PixelFormat, _: DataFormat);
    fn Tex2D_SetData(
        _: *mut Tex2D,
        _: *const libc::c_void,
        _: PixelFormat,
        _: DataFormat,
    );
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
}
pub type DataFormat = int32;
pub type PixelFormat = int32;
pub type TexFormat = int32;

#[inline]
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Ceil(mut t: libc::c_double) -> libc::c_double {
    return ceil(t);
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}

#[inline]
unsafe extern "C" fn Vec4f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) -> Vec4f {
    let mut self_0: Vec4f = {
        let mut init = Vec4f { x: x, y: y, z: z, w: w };
        init
    };
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeAO(
    mut self_0: *mut Mesh,
    mut radius: libc::c_float,
) {
    let mut indexCount: libc::c_int = Mesh_GetIndexCount(self_0);
    let mut vertexCount: libc::c_int = Mesh_GetVertexCount(self_0);
    let mut indexData: *mut libc::c_int = Mesh_GetIndexData(self_0);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(self_0);
    let mut sDim: libc::c_int = Ceil(
        Sqrt((indexCount / 3 as libc::c_int) as libc::c_double),
    ) as libc::c_int;
    let mut vDim: libc::c_int = Ceil(Sqrt(vertexCount as libc::c_double)) as libc::c_int;
    let mut surfels: libc::c_int = sDim * sDim;
    let mut vertices: libc::c_int = vDim * vDim;
    let mut bufSize: libc::c_int = Max(
        surfels as libc::c_double,
        vertices as libc::c_double,
    ) as libc::c_int;
    let mut pointBuffer: *mut Vec4f = MemAlloc(
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(bufSize as usize),
    ) as *mut Vec4f;
    let mut normalBuffer: *mut Vec4f = MemAlloc(
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(bufSize as usize),
    ) as *mut Vec4f;
    MemZero(
        pointBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(bufSize as usize),
    );
    MemZero(
        normalBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(bufSize as usize),
    );
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < indexCount {
        let mut v1: *const Vertex = vertexData
            .offset(*indexData.offset((i + 0 as libc::c_int) as isize) as isize);
        let mut v2: *const Vertex = vertexData
            .offset(*indexData.offset((i + 1 as libc::c_int) as isize) as isize);
        let mut v3: *const Vertex = vertexData
            .offset(*indexData.offset((i + 2 as libc::c_int) as isize) as isize);
        let mut normal: Vec3 = Vec3::cross(
            (*v3).p - (*v1).p,
            (*v2).p - (*v1).p,
        );
        let mut length: libc::c_float = normal.length();
        let mut area: libc::c_float = 0.5f32 * length / 3.14159265f32;
        if Abs(length as libc::c_double) > 1e-6f64 {
            normal /= length;
        } else {
            normal = Vec3::new(
                1.0f32,
                0.0f32,
                0.0f32,
            );
        }
        let mut center: Vec3 = ((*v1).p + (*v2).p + (*v3).p) / 3.0f32;
        *pointBuffer
            .offset(
                (i / 3 as libc::c_int) as isize,
            ) = Vec4f_Create(center.x, center.y, center.z, area);
        *normalBuffer
            .offset(
                (i / 3 as libc::c_int) as isize,
            ) = Vec4f_Create(
            normal.x,
            normal.y,
            normal.z,
            0 as libc::c_int as libc::c_float,
        );
        i += 3 as libc::c_int;
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
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(bufSize as usize),
    );
    MemZero(
        normalBuffer as *mut libc::c_void,
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(bufSize as usize),
    );
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < vertexCount {
        let mut v: *const Vertex = vertexData.offset(i_0 as isize);
        *pointBuffer
            .offset(
                i_0 as isize,
            ) = Vec4f_Create(
            (*v).p.x,
            (*v).p.y,
            (*v).p.z,
            0 as libc::c_int as libc::c_float,
        );
        *normalBuffer
            .offset(
                i_0 as isize,
            ) = Vec4f_Create(
            (*v).n.x,
            (*v).n.y,
            (*v).n.z,
            0 as libc::c_int as libc::c_float,
        );
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
    static mut shader: *mut Shader = 0 as *const Shader as *mut Shader;
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
    Shader_SetTex2D(b"sPointBuffer\0" as *const u8 as *const libc::c_char, texSPoints);
    Shader_SetTex2D(b"sNormalBuffer\0" as *const u8 as *const libc::c_char, texSNormals);
    Shader_SetTex2D(b"vPointBuffer\0" as *const u8 as *const libc::c_char, texVPoints);
    Shader_SetTex2D(b"vNormalBuffer\0" as *const u8 as *const libc::c_char, texVNormals);
    Draw_Rect(
        -(1 as libc::c_int) as libc::c_float,
        -(1 as libc::c_int) as libc::c_float,
        2 as libc::c_int as libc::c_float,
        2 as libc::c_int as libc::c_float,
    );
    Shader_Stop(shader);
    RenderTarget_Pop();
    RenderState_PopAll();
    let mut result: *mut libc::c_float = MemAlloc(
        (::core::mem::size_of::<libc::c_float>())
            .wrapping_mul((vDim * vDim) as usize),
    ) as *mut libc::c_float;
    Tex2D_GetData(
        texOutput,
        result as *mut libc::c_void,
        PixelFormat_Red,
        DataFormat_Float,
    );
    let mut i_1: libc::c_int = 0 as libc::c_int;
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
    mut self_0: *mut Mesh,
    mut sdf: *mut Tex3D,
    mut radius: libc::c_float,
) {
    let mut vertexCount: libc::c_int = Mesh_GetVertexCount(self_0);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(self_0);
    let mut vDim: libc::c_int = Ceil(Sqrt(vertexCount as libc::c_double)) as libc::c_int;
    let mut texPoints: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_RGBA32F);
    let mut texOutput: *mut Tex2D = Tex2D_Create(vDim, vDim, TexFormat_R32F);
    let mut pointBuffer: *mut Vec3 = MemAlloc(
        (::core::mem::size_of::<Vec3>())
            .wrapping_mul((vDim * vDim) as usize),
    ) as *mut Vec3;
    let mut i: libc::c_int = 0 as libc::c_int;
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
    static mut shader: *mut Shader = 0 as *const Shader as *mut Shader;
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
    Draw_Rect(
        -(1 as libc::c_int) as libc::c_float,
        -(1 as libc::c_int) as libc::c_float,
        2 as libc::c_int as libc::c_float,
        2 as libc::c_int as libc::c_float,
    );
    Shader_Stop(shader);
    RenderTarget_Pop();
    RenderState_PopAll();
    let mut result: *mut libc::c_float = MemAlloc(
        (::core::mem::size_of::<libc::c_float>())
            .wrapping_mul((vDim * vDim) as usize),
    ) as *mut libc::c_float;
    Tex2D_GetData(
        texOutput,
        result as *mut libc::c_void,
        PixelFormat_Red,
        DataFormat_Float,
    );
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < vertexCount {
        (*vertexData.offset(i_0 as isize)).uv.x = *result.offset(i_0 as isize);
        i_0 += 1;
    }
    MemFree(result as *const libc::c_void);
    Tex2D_Free(texPoints);
    Tex2D_Free(texOutput);
}
