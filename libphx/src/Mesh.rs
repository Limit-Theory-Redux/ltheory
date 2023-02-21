use ::libc;
use libc::c_int;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::ResourceType::*;
use memoffset::{offset_of, span_of};
use glam::Vec2;
use crate::Bytes::*;

extern "C" {
    pub type SDF;
    pub type Matrix;
    fn Fatal(_: cstr, _: ...);
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Matrix_Free(_: *mut Matrix);
    fn Matrix_RotationX(rads: libc::c_float) -> *mut Matrix;
    fn Matrix_RotationY(rads: libc::c_float) -> *mut Matrix;
    fn Matrix_RotationZ(rads: libc::c_float) -> *mut Matrix;
    fn Matrix_YawPitchRoll(
        yaw: libc::c_float,
        pitch: libc::c_float,
        roll: libc::c_float,
    ) -> *mut Matrix;
    fn Matrix_MulPoint(
        _: *const Matrix,
        out: *mut Vec3,
        x: libc::c_float,
        y: libc::c_float,
        z: libc::c_float,
    );
    fn Metric_AddDraw(polys: int32, tris: int32, verts: int32);
    fn glBegin(mode: GLenum);
    fn glDrawElements(
        mode: GLenum,
        count: GLsizei,
        type_0: GLenum,
        indices: *const libc::c_void,
    );
    fn glEnd();
    fn glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat);
    static mut __glewBindBuffer: PFNGLBINDBUFFERPROC;
    static mut __glewBufferData: PFNGLBUFFERDATAPROC;
    static mut __glewDeleteBuffers: PFNGLDELETEBUFFERSPROC;
    static mut __glewGenBuffers: PFNGLGENBUFFERSPROC;
    static mut __glewDisableVertexAttribArray: PFNGLDISABLEVERTEXATTRIBARRAYPROC;
    static mut __glewEnableVertexAttribArray: PFNGLENABLEVERTEXATTRIBARRAYPROC;
    static mut __glewVertexAttribPointer: PFNGLVERTEXATTRIBPOINTERPROC;
    fn Resource_LoadBytes(_: ResourceType, name: cstr) -> *mut Bytes;
    fn SDF_ToMesh(_: *mut SDF) -> *mut Mesh;
    fn Triangle_Validate(_: *const Triangle) -> Error;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_ptrdiff_t = libc::c_long;
pub type uint = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Mesh {
    pub _refCount: uint32,
    pub vbo: uint,
    pub ibo: uint,
    pub version: uint64,
    pub versionBuffers: uint64,
    pub versionInfo: uint64,
    pub info: Computed,
    pub index_size: int32,
    pub index_capacity: int32,
    pub index_data: *mut int32,
    pub vertex_size: int32,
    pub vertex_capacity: int32,
    pub vertex_data: *mut Vertex,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Computed {
    pub bound: Box3f,
    pub radius: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3f {
    pub lower: Vec3,
    pub upper: Vec3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}
pub type ResourceType = int32;
pub type GLuint = libc::c_uint;
pub type PFNGLDELETEBUFFERSPROC = Option::<
    unsafe extern "C" fn(GLsizei, *const GLuint) -> (),
>;
pub type GLsizei = libc::c_int;
pub type GLenum = libc::c_uint;
pub type PFNGLBINDBUFFERPROC = Option::<unsafe extern "C" fn(GLenum, GLuint) -> ()>;
pub type PFNGLDISABLEVERTEXATTRIBARRAYPROC = Option::<
    unsafe extern "C" fn(GLuint) -> (),
>;
pub type GLboolean = libc::c_uchar;
pub type PFNGLVERTEXATTRIBPOINTERPROC = Option::<
    unsafe extern "C" fn(
        GLuint,
        GLint,
        GLenum,
        GLboolean,
        GLsizei,
        *const libc::c_void,
    ) -> (),
>;
pub type GLint = libc::c_int;
pub type PFNGLENABLEVERTEXATTRIBARRAYPROC = Option::<unsafe extern "C" fn(GLuint) -> ()>;
pub type GLsizeiptr = ptrdiff_t;
pub type ptrdiff_t = __darwin_ptrdiff_t;
pub type PFNGLBUFFERDATAPROC = Option::<
    unsafe extern "C" fn(GLenum, GLsizeiptr, *const libc::c_void, GLenum) -> (),
>;
pub type PFNGLGENBUFFERSPROC = Option::<
    unsafe extern "C" fn(GLsizei, *mut GLuint) -> (),
>;
pub type GLfloat = libc::c_float;



#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Maxf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}

#[inline]
unsafe extern "C" fn Box3f_Center(mut self_0: Box3f) -> Vec3 {
    let mut center: Vec3 = {
        let mut init = Vec3 {
            x: (self_0.lower.x + self_0.upper.x) / 2 as libc::c_int as libc::c_float,
            y: (self_0.lower.y + self_0.upper.y) / 2 as libc::c_int as libc::c_float,
            z: (self_0.lower.z + self_0.upper.z) / 2 as libc::c_int as libc::c_float,
        };
        init
    };
    return center;
}
#[inline]
unsafe extern "C" fn Box3f_Add(mut self_0: *mut Box3f, mut point: Vec3) {
    (*self_0).lower = Vec3::min((*self_0).lower, point);
    (*self_0).upper = Vec3::max((*self_0).upper, point);
}

#[inline]
unsafe extern "C" fn Vec2_Validate(mut v: Vec2) -> Error {
    let mut e: Error = 0 as libc::c_int as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    return e;
}
unsafe extern "C" fn Mesh_UpdateInfo(mut self_0: *mut Mesh) {
    if (*self_0).versionInfo == (*self_0).version {
        return;
    }
    (*self_0)
        .info
        .bound
        .lower = Vec3::new(3.40282347e+38f32, 3.40282347e+38f32, 3.40282347e+38f32);
    (*self_0)
        .info
        .bound
        .upper = Vec3::new(
        -3.40282347e+38f32,
        -3.40282347e+38f32,
        -3.40282347e+38f32,
    );
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        Box3f_Add(&mut (*self_0).info.bound, (*v).p);
        v = v.offset(1);
    }
    let mut center: Vec3 = Box3f_Center((*self_0).info.bound);
    let mut r2: libc::c_double = 0.0f64;
    let mut v_0: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v_0 < __iterend_0 {
        let mut dx: libc::c_double = ((*v_0).p.x - center.x) as libc::c_double;
        let mut dy: libc::c_double = ((*v_0).p.y - center.y) as libc::c_double;
        let mut dz: libc::c_double = ((*v_0).p.z - center.z) as libc::c_double;
        r2 = Max(r2, dx * dx + dy * dy + dz * dz);
        v_0 = v_0.offset(1);
    }
    (*self_0).info.radius = Sqrt(r2) as libc::c_float;
    (*self_0).versionInfo = (*self_0).version;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Create() -> *mut Mesh {
    let mut self_0: *mut Mesh = MemAlloc(::core::mem::size_of::<Mesh>())
        as *mut Mesh;
    (*self_0)._refCount = 1 as libc::c_int as uint32;
    (*self_0).vbo = 0 as libc::c_int as uint;
    (*self_0).ibo = 0 as libc::c_int as uint;
    (*self_0).version = 1 as libc::c_int as uint64;
    (*self_0).versionBuffers = 0 as libc::c_int as uint64;
    (*self_0).versionInfo = 0 as libc::c_int as uint64;
    (*self_0).vertex_capacity = 0 as libc::c_int;
    (*self_0).vertex_size = 0 as libc::c_int;
    (*self_0).vertex_data = 0 as *mut Vertex;
    (*self_0).index_capacity = 0 as libc::c_int;
    (*self_0).index_size = 0 as libc::c_int;
    (*self_0).index_data = 0 as *mut int32;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Clone(mut other: *mut Mesh) -> *mut Mesh {
    let mut self_0: *mut Mesh = Mesh_Create();
    if ((*self_0).index_capacity < (*other).index_size) as libc::c_int as libc::c_long
        != 0
    {
        (*self_0).index_capacity = (*other).index_size;
        let mut elemSize: usize = ::core::mem::size_of::<int32>();
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).index_data
            as *mut *mut int32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).index_data as *mut libc::c_void,
            ((*self_0).index_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    if ((*self_0).vertex_capacity < (*other).vertex_size) as libc::c_int as libc::c_long
        != 0
    {
        (*self_0).vertex_capacity = (*other).vertex_size;
        let mut elemSize_0: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData_0: *mut *mut libc::c_void = &mut (*self_0).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            (*self_0).vertex_data as *mut libc::c_void,
            ((*self_0).vertex_capacity as usize).wrapping_mul(elemSize_0 as usize),
        );
    }
    (*self_0).index_size = (*other).index_size;
    (*self_0).vertex_size = (*other).vertex_size;
    MemCpy(
        (*self_0).index_data as *mut libc::c_void,
        (*other).index_data as *const libc::c_void,
        (::core::mem::size_of::<libc::c_int>())
            .wrapping_mul((*other).index_size as usize),
    );
    MemCpy(
        (*self_0).vertex_data as *mut libc::c_void,
        (*other).vertex_data as *const libc::c_void,
        (::core::mem::size_of::<Vertex>())
            .wrapping_mul((*other).vertex_size as usize),
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Load(mut name: cstr) -> *mut Mesh {
    let mut bytes: *mut Bytes = Resource_LoadBytes(ResourceType_Mesh, name);
    let mut self_0: *mut Mesh = Mesh_FromBytes(bytes);
    Bytes_Free(bytes);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Acquire(mut self_0: *mut Mesh) {
    (*self_0)._refCount = ((*self_0)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Free(mut self_0: *mut Mesh) {
    if !self_0.is_null()
        && {
            (*self_0)._refCount = ((*self_0)._refCount).wrapping_sub(1);
            (*self_0)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        MemFree((*self_0).vertex_data as *const libc::c_void);
        MemFree((*self_0).index_data as *const libc::c_void);
        if (*self_0).vbo != 0 {
            __glewDeleteBuffers
                .expect(
                    "non-null function pointer",
                )(1 as libc::c_int, &mut (*self_0).vbo);
            __glewDeleteBuffers
                .expect(
                    "non-null function pointer",
                )(1 as libc::c_int, &mut (*self_0).ibo);
        }
        MemFree(self_0 as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ToBytes(mut mesh: *mut Mesh) -> *mut Bytes {
    let mut vertexCount: int32 = Mesh_GetVertexCount(mesh);
    let mut indexCount: int32 = Mesh_GetIndexCount(mesh);
    let mut size: uint32 = (2 as usize).wrapping_mul(::core::mem::size_of::<int32>())
        .wrapping_add(
            (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()),
        )
        .wrapping_add(
            (indexCount as usize).wrapping_mul(::core::mem::size_of::<int32>()),
        ) as uint32;
    let mut self_0: *mut Bytes = Bytes_Create(size);
    Bytes_WriteI32(self_0, vertexCount);
    Bytes_WriteI32(self_0, indexCount);
    Bytes_Write(
        self_0,
        (*mesh).vertex_data as *const libc::c_void,
        (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()) as uint32,
    );
    Bytes_Write(
        self_0,
        (*mesh).index_data as *const libc::c_void,
        (indexCount as usize).wrapping_mul(::core::mem::size_of::<int32>()) as uint32,
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_FromBytes(mut buf: *mut Bytes) -> *mut Mesh {
    let mut self_0: *mut Mesh = Mesh_Create();
    let mut vertexCount: int32 = Bytes_ReadI32(buf);
    let mut indexCount: int32 = Bytes_ReadI32(buf);
    Mesh_ReserveVertexData(self_0, vertexCount);
    Mesh_ReserveIndexData(self_0, indexCount);
    Bytes_Read(
        buf,
        (*self_0).vertex_data as *mut libc::c_void,
        (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()) as uint32,
    );
    Bytes_Read(
        buf,
        (*self_0).index_data as *mut libc::c_void,
        (indexCount as usize).wrapping_mul(::core::mem::size_of::<int32>()) as uint32,
    );
    (*self_0).vertex_size = vertexCount;
    (*self_0).index_size = indexCount;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_FromSDF(mut sdf: *mut SDF) -> *mut Mesh {
    return SDF_ToMesh(sdf);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddIndex(
    mut self_0: *mut Mesh,
    mut newIndex: libc::c_int,
) {
    if ((*self_0).index_capacity == (*self_0).index_size) as libc::c_int as libc::c_long
        != 0
    {
        (*self_0)
            .index_capacity = if (*self_0).index_capacity != 0 {
            (*self_0).index_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<int32>();
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).index_data
            as *mut *mut int32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).index_data as *mut libc::c_void,
            ((*self_0).index_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh0 = (*self_0).index_size;
    (*self_0).index_size = (*self_0).index_size + 1;
    *((*self_0).index_data).offset(fresh0 as isize) = newIndex;
    (*self_0).version = ((*self_0).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddMesh(mut self_0: *mut Mesh, mut other: *mut Mesh) {
    let mut indexOffset: libc::c_int = (*self_0).vertex_size;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*other).vertex_size {
        Mesh_AddVertexRaw(self_0, ((*other).vertex_data).offset(i as isize));
        i += 1;
    }
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < (*other).index_size {
        Mesh_AddIndex(self_0, *((*other).index_data).offset(i_0 as isize) + indexOffset);
        i_0 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddQuad(
    mut self_0: *mut Mesh,
    mut i1: libc::c_int,
    mut i2: libc::c_int,
    mut i3: libc::c_int,
    mut i4: libc::c_int,
) {
    Mesh_AddTri(self_0, i1, i2, i3);
    Mesh_AddTri(self_0, i1, i3, i4);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddTri(
    mut self_0: *mut Mesh,
    mut i1: libc::c_int,
    mut i2: libc::c_int,
    mut i3: libc::c_int,
) {
    Mesh_AddIndex(self_0, i1);
    Mesh_AddIndex(self_0, i2);
    Mesh_AddIndex(self_0, i3);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertex(
    mut self_0: *mut Mesh,
    mut px: libc::c_float,
    mut py: libc::c_float,
    mut pz: libc::c_float,
    mut nx: libc::c_float,
    mut ny: libc::c_float,
    mut nz: libc::c_float,
    mut u: libc::c_float,
    mut v: libc::c_float,
) {
    if ((*self_0).vertex_capacity == (*self_0).vertex_size) as libc::c_int
        as libc::c_long != 0
    {
        (*self_0)
            .vertex_capacity = if (*self_0).vertex_capacity != 0 {
            (*self_0).vertex_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).vertex_data as *mut libc::c_void,
            ((*self_0).vertex_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh1 = (*self_0).vertex_size;
    (*self_0).vertex_size = (*self_0).vertex_size + 1;
    let mut newVertex: *mut Vertex = ((*self_0).vertex_data).offset(fresh1 as isize);
    (*newVertex).p = Vec3::new(px, py, pz);
    (*newVertex).n = Vec3::new(nx, ny, nz);
    (*newVertex).uv = Vec2::new(u, v);
    (*self_0).version = ((*self_0).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertexRaw(
    mut self_0: *mut Mesh,
    mut vertex: *const Vertex,
) {
    if ((*self_0).vertex_capacity == (*self_0).vertex_size) as libc::c_int
        as libc::c_long != 0
    {
        (*self_0)
            .vertex_capacity = if (*self_0).vertex_capacity != 0 {
            (*self_0).vertex_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).vertex_data as *mut libc::c_void,
            ((*self_0).vertex_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh2 = (*self_0).vertex_size;
    (*self_0).vertex_size = (*self_0).vertex_size + 1;
    *((*self_0).vertex_data).offset(fresh2 as isize) = *vertex;
    (*self_0).version = ((*self_0).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBind(mut self_0: *mut Mesh) {
    if (*self_0).vbo != 0 && (*self_0).version != (*self_0).versionBuffers {
        __glewDeleteBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*self_0).vbo);
        __glewDeleteBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*self_0).ibo);
        (*self_0).vbo = 0 as libc::c_int as uint;
        (*self_0).ibo = 0 as libc::c_int as uint;
    }
    if (*self_0).vbo == 0 {
        __glewGenBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*self_0).vbo);
        __glewGenBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*self_0).ibo);
        __glewBindBuffer
            .expect(
                "non-null function pointer",
            )(0x8892 as libc::c_int as GLenum, (*self_0).vbo);
        __glewBindBuffer
            .expect(
                "non-null function pointer",
            )(0x8893 as libc::c_int as GLenum, (*self_0).ibo);
        __glewBufferData
            .expect(
                "non-null function pointer",
            )(
            0x8892 as libc::c_int as GLenum,
            ((*self_0).vertex_size as usize).wrapping_mul(::core::mem::size_of::<Vertex>())
                as GLsizeiptr,
            (*self_0).vertex_data as *const libc::c_void,
            0x88e4 as libc::c_int as GLenum,
        );
        __glewBufferData
            .expect(
                "non-null function pointer",
            )(
            0x8893 as libc::c_int as GLenum,
            ((*self_0).index_size as usize).wrapping_mul(::core::mem::size_of::<libc::c_int>())
                as GLsizeiptr,
            (*self_0).index_data as *const libc::c_void,
            0x88e4 as libc::c_int as GLenum,
        );
        (*self_0).versionBuffers = (*self_0).version;
    }
    __glewBindBuffer
        .expect(
            "non-null function pointer",
        )(0x8892 as libc::c_int as GLenum, (*self_0).vbo);
    __glewBindBuffer
        .expect(
            "non-null function pointer",
        )(0x8893 as libc::c_int as GLenum, (*self_0).ibo);
    __glewEnableVertexAttribArray
        .expect("non-null function pointer")(0 as libc::c_int as GLuint);
    __glewEnableVertexAttribArray
        .expect("non-null function pointer")(1 as libc::c_int as GLuint);
    __glewEnableVertexAttribArray
        .expect("non-null function pointer")(2 as libc::c_int as GLuint);
    __glewVertexAttribPointer
        .expect(
            "non-null function pointer",
        )(
        0 as libc::c_int as GLuint,
        3 as libc::c_int,
        0x1406 as libc::c_int as GLenum,
        0 as libc::c_int as GLboolean,
        ::core::mem::size_of::<Vertex>() as usize as GLsizei,
        offset_of!(Vertex, p) as *const libc::c_void,
    );
    __glewVertexAttribPointer
        .expect(
            "non-null function pointer",
        )(
        1 as libc::c_int as GLuint,
        3 as libc::c_int,
        0x1406 as libc::c_int as GLenum,
        0 as libc::c_int as GLboolean,
        ::core::mem::size_of::<Vertex>() as usize as GLsizei,
        offset_of!(Vertex, n) as *const libc::c_void,
    );
    __glewVertexAttribPointer
        .expect(
            "non-null function pointer",
        )(
        2 as libc::c_int as GLuint,
        2 as libc::c_int,
        0x1406 as libc::c_int as GLenum,
        0 as libc::c_int as GLboolean,
        ::core::mem::size_of::<Vertex>() as usize as GLsizei,
        offset_of!(Vertex, uv) as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBound(mut self_0: *mut Mesh) {
    Metric_AddDraw(
        (*self_0).index_size / 3 as libc::c_int,
        (*self_0).index_size / 3 as libc::c_int,
        (*self_0).vertex_size,
    );
    glDrawElements(
        0x4 as libc::c_int as GLenum,
        (*self_0).index_size,
        0x1405 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawUnbind(mut self_0: *mut Mesh) {
    __glewDisableVertexAttribArray
        .expect("non-null function pointer")(0 as libc::c_int as GLuint);
    __glewDisableVertexAttribArray
        .expect("non-null function pointer")(1 as libc::c_int as GLuint);
    __glewDisableVertexAttribArray
        .expect("non-null function pointer")(2 as libc::c_int as GLuint);
    __glewBindBuffer
        .expect(
            "non-null function pointer",
        )(0x8892 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
    __glewBindBuffer
        .expect(
            "non-null function pointer",
        )(0x8893 as libc::c_int as GLenum, 0 as libc::c_int as GLuint);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Draw(mut self_0: *mut Mesh) {
    Mesh_DrawBind(self_0);
    Mesh_DrawBound(self_0);
    Mesh_DrawUnbind(self_0);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawNormals(
    mut self_0: *mut Mesh,
    mut scale: libc::c_float,
) {
    glBegin(0x1 as libc::c_int as GLenum);
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        glVertex3f((*v).p.x, (*v).p.y, (*v).p.z);
        glVertex3f(
            (*v).p.x + scale * (*v).n.x,
            (*v).p.y + scale * (*v).n.y,
            (*v).p.z + scale * (*v).n.z,
        );
        v = v.offset(1);
    }
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetBound(mut self_0: *mut Mesh, mut out: *mut Box3f) {
    Mesh_UpdateInfo(self_0);
    *out = (*self_0).info.bound;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetCenter(mut self_0: *mut Mesh, mut out: *mut Vec3) {
    Mesh_UpdateInfo(self_0);
    *out = Box3f_Center((*self_0).info.bound);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexCount(mut self_0: *mut Mesh) -> libc::c_int {
    return (*self_0).index_size;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexData(mut self_0: *mut Mesh) -> *mut libc::c_int {
    return (*self_0).index_data;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetRadius(mut self_0: *mut Mesh) -> libc::c_float {
    Mesh_UpdateInfo(self_0);
    return (*self_0).info.radius;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVersion(mut self_0: *mut Mesh) -> uint64 {
    return (*self_0).version;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_IncVersion(mut self_0: *mut Mesh) {
    (*self_0).version = ((*self_0).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Validate(mut self_0: *mut Mesh) -> Error {
    let mut indexLen: int32 = Mesh_GetIndexCount(self_0);
    let mut indexData: *mut int32 = Mesh_GetIndexData(self_0);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(self_0);
    if indexLen % 3 as libc::c_int != 0 as libc::c_int {
        return (0x100000 as libc::c_int | 0x80 as libc::c_int) as Error;
    }
    let mut i: int32 = 0 as libc::c_int;
    while i < indexLen {
        let mut i0: int32 = *indexData.offset((i + 0 as libc::c_int) as isize);
        let mut i1: int32 = *indexData.offset((i + 1 as libc::c_int) as isize);
        let mut i2: int32 = *indexData.offset((i + 2 as libc::c_int) as isize);
        let mut triangle: Triangle = Triangle {
            vertices: [Vec3 { x: 0., y: 0., z: 0. }; 3],
        };
        triangle
            .vertices[0] = (*vertexData.offset(i0 as isize)).p;
        triangle
            .vertices[1] = (*vertexData.offset(i1 as isize)).p;
        triangle
            .vertices[2] = (*vertexData.offset(i2 as isize)).p;
        let mut e: Error = Triangle_Validate(&mut triangle);
        if e != 0 as libc::c_int as libc::c_uint {
            return 0x400000 as libc::c_int as libc::c_uint | e;
        }
        i += 3 as libc::c_int;
    }
    let mut v: *const Vertex = (*self_0).vertex_data;
    let mut __iterend: *const Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        let mut e_0: Error = 0;
        e_0 = Vec3_Validate((*v).p);
        if e_0 != 0 as libc::c_int as libc::c_uint {
            return 0x400000 as libc::c_int as libc::c_uint | e_0;
        }
        e_0 = Vec3_Validate((*v).n);
        if e_0 != 0 as libc::c_int as libc::c_uint {
            return 0x800000 as libc::c_int as libc::c_uint | e_0;
        }
        e_0 = Vec2_Validate((*v).uv);
        if e_0 != 0 as libc::c_int as libc::c_uint {
            return 0x1000000 as libc::c_int as libc::c_uint | e_0;
        }
        v = v.offset(1);
    }
    return 0 as libc::c_int as Error;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertex(
    mut self_0: *mut Mesh,
    mut index: libc::c_int,
) -> *mut Vertex {
    return ((*self_0).vertex_data).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexCount(mut self_0: *mut Mesh) -> libc::c_int {
    return (*self_0).vertex_size;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexData(mut self_0: *mut Mesh) -> *mut Vertex {
    return (*self_0).vertex_data;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveIndexData(
    mut self_0: *mut Mesh,
    mut capacity: libc::c_int,
) {
    if ((*self_0).index_capacity < capacity) as libc::c_int as libc::c_long != 0 {
        (*self_0).index_capacity = capacity;
        let mut elemSize: usize = ::core::mem::size_of::<int32>();
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).index_data
            as *mut *mut int32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).index_data as *mut libc::c_void,
            ((*self_0).index_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveVertexData(
    mut self_0: *mut Mesh,
    mut capacity: libc::c_int,
) {
    if ((*self_0).vertex_capacity < capacity) as libc::c_int as libc::c_long != 0 {
        (*self_0).vertex_capacity = capacity;
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).vertex_data as *mut libc::c_void,
            ((*self_0).vertex_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Center(mut self_0: *mut Mesh) -> *mut Mesh {
    let mut c = Vec3::ZERO;
    Mesh_GetCenter(self_0, &mut c);
    Mesh_Translate(self_0, -c.x, -c.y, -c.z);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Invert(mut self_0: *mut Mesh) -> *mut Mesh {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*self_0).index_size {
        let mut swap_temp: [libc::c_uchar; 4] = [0; 4];
        libc::memcpy(
            swap_temp.as_mut_ptr() as *mut libc::c_void,
            &mut *((*self_0).index_data).offset((i + 2 as libc::c_int) as isize)
                as *mut int32 as *const libc::c_void,
            ::core::mem::size_of::<int32>() as usize,
        );
        libc::memcpy(
            &mut *((*self_0).index_data).offset((i + 2 as libc::c_int) as isize)
                as *mut int32 as *mut libc::c_void,
            &mut *((*self_0).index_data).offset((i + 1 as libc::c_int) as isize)
                as *mut int32 as *const libc::c_void,
            ::core::mem::size_of::<int32>() as usize,
        );
        libc::memcpy(
            &mut *((*self_0).index_data).offset((i + 1 as libc::c_int) as isize)
                as *mut int32 as *mut libc::c_void,
            swap_temp.as_mut_ptr() as *const libc::c_void,
            ::core::mem::size_of::<int32>() as usize,
        );
        i += 3 as libc::c_int;
    }
    (*self_0).version = ((*self_0).version).wrapping_add(1);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateX(
    mut self_0: *mut Mesh,
    mut rads: libc::c_float,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationX(rads);
    Mesh_Transform(self_0, matrix);
    Matrix_Free(matrix);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateY(
    mut self_0: *mut Mesh,
    mut rads: libc::c_float,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationY(rads);
    Mesh_Transform(self_0, matrix);
    Matrix_Free(matrix);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateZ(
    mut self_0: *mut Mesh,
    mut rads: libc::c_float,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationZ(rads);
    Mesh_Transform(self_0, matrix);
    Matrix_Free(matrix);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateYPR(
    mut self_0: *mut Mesh,
    mut yaw: libc::c_float,
    mut pitch: libc::c_float,
    mut roll: libc::c_float,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_YawPitchRoll(yaw, pitch, roll);
    Mesh_Transform(self_0, matrix);
    Matrix_Free(matrix);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Scale(
    mut self_0: *mut Mesh,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        (*v).p.x *= x;
        (*v).p.y *= y;
        (*v).p.z *= z;
        v = v.offset(1);
    }
    (*self_0).version = ((*self_0).version).wrapping_add(1);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ScaleUniform(
    mut self_0: *mut Mesh,
    mut s: libc::c_float,
) -> *mut Mesh {
    Mesh_Scale(self_0, s, s, s);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Translate(
    mut self_0: *mut Mesh,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        (*v).p.x += x;
        (*v).p.y += y;
        (*v).p.z += z;
        v = v.offset(1);
    }
    (*self_0).version = ((*self_0).version).wrapping_add(1);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Transform(
    mut self_0: *mut Mesh,
    mut matrix: *mut Matrix,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        Matrix_MulPoint(matrix, &mut (*v).p, (*v).p.x, (*v).p.y, (*v).p.z);
        v = v.offset(1);
    }
    (*self_0).version = ((*self_0).version).wrapping_add(1);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeNormals(mut self_0: *mut Mesh) {
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        (*v).n.x = 0 as libc::c_int as libc::c_float;
        (*v).n.y = 0 as libc::c_int as libc::c_float;
        (*v).n.z = 0 as libc::c_int as libc::c_float;
        v = v.offset(1);
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*self_0).index_size {
        let mut v1: *mut Vertex = ((*self_0).vertex_data)
            .offset(
                *((*self_0).index_data).offset((i + 0 as libc::c_int) as isize) as isize,
            );
        let mut v2: *mut Vertex = ((*self_0).vertex_data)
            .offset(
                *((*self_0).index_data).offset((i + 1 as libc::c_int) as isize) as isize,
            );
        let mut v3: *mut Vertex = ((*self_0).vertex_data)
            .offset(
                *((*self_0).index_data).offset((i + 2 as libc::c_int) as isize) as isize,
            );
        let mut e1: Vec3 = (*v2).p - (*v1).p;
        let mut e2: Vec3 = (*v3).p - (*v2).p;
        let mut en: Vec3 = Vec3::cross(e1, e2);
        (*v1).n = (*v1).n + en;
        (*v2).n = (*v2).n + en;
        (*v3).n = (*v3).n + en;
        i += 3 as libc::c_int;
    }
    let mut v_0: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v_0 < __iterend_0 {
        (*v_0).n = (*v_0).n.normalize();
        v_0 = v_0.offset(1);
    }
    (*self_0).version = ((*self_0).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_SplitNormals(
    mut self_0: *mut Mesh,
    mut minDot: libc::c_float,
) {
    let mut v: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v < __iterend {
        (*v)
            .n = Vec3::new(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        v = v.offset(1);
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*self_0).index_size {
        let mut index: [*mut int32; 3] = [
            ((*self_0).index_data).offset(i as isize).offset(0),
            ((*self_0).index_data).offset(i as isize).offset(1),
            ((*self_0).index_data).offset(i as isize).offset(2),
        ];
        let mut v_0: [*mut Vertex; 3] = [
            ((*self_0).vertex_data).offset(*index[0] as isize),
            ((*self_0).vertex_data).offset(*index[1] as isize),
            ((*self_0).vertex_data).offset(*index[2] as isize),
        ];
        let mut face: Vec3 = Vec3::cross(
            (*v_0[1]).p - (*v_0[0]).p,
            (*v_0[2]).p - (*v_0[0]).p,
        );
        let mut j: libc::c_int = 0 as libc::c_int;
        while j < 3 as libc::c_int {
            let mut cn: *mut Vec3 = &mut (*((*self_0).vertex_data)
                .offset(**index.as_mut_ptr().offset(j as isize) as isize))
                .n;
            if (*cn).length_squared() > 0.0f32 {
                let mut cDot: libc::c_float = Vec3::dot(
                    face.normalize(),
                    (*cn).normalize(),
                );
                if cDot < minDot {
                    if ((*self_0).vertex_capacity == (*self_0).vertex_size)
                        as libc::c_int as libc::c_long != 0
                    {
                        (*self_0)
                            .vertex_capacity = if (*self_0).vertex_capacity != 0 {
                            (*self_0).vertex_capacity * 2 as libc::c_int
                        } else {
                            1 as libc::c_int
                        };
                        let mut elemSize: usize = ::core::mem::size_of::<Vertex>();
                        let mut pData: *mut *mut libc::c_void = &mut (*self_0)
                            .vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
                        *pData = MemRealloc(
                            (*self_0).vertex_data as *mut libc::c_void,
                            ((*self_0).vertex_capacity as usize)
                                .wrapping_mul(elemSize),
                        );
                    }
                    let fresh3 = (*self_0).vertex_size;
                    (*self_0).vertex_size = (*self_0).vertex_size + 1;
                    let mut nv: *mut Vertex = ((*self_0).vertex_data)
                        .offset(fresh3 as isize);
                    *nv = *((*self_0).vertex_data).offset(*index[j as usize] as isize);
                    (*nv).n = face;
                    *index[j as usize] = (*self_0).vertex_size - 1 as libc::c_int;
                } else {
                    (*cn) += face;
                }
            } else {
                (*cn) += face;
            }
            j += 1;
        }
        i += 3 as libc::c_int;
    }
    let mut v_1: *mut Vertex = (*self_0).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*self_0).vertex_data)
        .offset((*self_0).vertex_size as isize);
    while v_1 < __iterend_0 {
        (*v_1).n = (*v_1).n.normalize();
        v_1 = v_1.offset(1);
    }
}
