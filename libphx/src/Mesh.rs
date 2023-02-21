use ::libc;
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
    fn sqrt(_: f64) -> f64;
    fn Matrix_Free(_: *mut Matrix);
    fn Matrix_RotationX(rads: f32) -> *mut Matrix;
    fn Matrix_RotationY(rads: f32) -> *mut Matrix;
    fn Matrix_RotationZ(rads: f32) -> *mut Matrix;
    fn Matrix_YawPitchRoll(
        yaw: f32,
        pitch: f32,
        roll: f32,
    ) -> *mut Matrix;
    fn Matrix_MulPoint(
        _: *const Matrix,
        out: *mut Vec3,
        x: f32,
        y: f32,
        z: f32,
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
    pub radius: f32,
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
pub type GLfloat = f32;



#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}
#[inline]
unsafe extern "C" fn Maxf(mut a: f32, mut b: f32) -> f32 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Max(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: f32, mut b: f32) -> f32 {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: f64) -> f64 {
    return sqrt(t);
}

#[inline]
unsafe extern "C" fn Box3f_Center(mut this: Box3f) -> Vec3 {
    let mut center: Vec3 =  Vec3 {
            x: (this.lower.x + this.upper.x) / 2.0f32,
            y: (this.lower.y + this.upper.y) / 2.0f32,
            z: (this.lower.z + this.upper.z) / 2.0f32,
        };
    return center;
}
#[inline]
unsafe extern "C" fn Box3f_Add(mut this: *mut Box3f, mut point: Vec3) {
    (*this).lower = Vec3::min((*this).lower, point);
    (*this).upper = Vec3::max((*this).upper, point);
}

#[inline]
unsafe extern "C" fn Vec2_Validate(mut v: Vec2) -> Error {
    let mut e: Error = 0 as libc::c_int as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    return e;
}
unsafe extern "C" fn Mesh_UpdateInfo(mut this: *mut Mesh) {
    if (*this).versionInfo == (*this).version {
        return;
    }
    (*this)
        .info
        .bound
        .lower = Vec3::new(3.40282347e+38f32, 3.40282347e+38f32, 3.40282347e+38f32);
    (*this)
        .info
        .bound
        .upper = Vec3::new(
        -3.40282347e+38f32,
        -3.40282347e+38f32,
        -3.40282347e+38f32,
    );
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v < __iterend {
        Box3f_Add(&mut (*this).info.bound, (*v).p);
        v = v.offset(1);
    }
    let mut center: Vec3 = Box3f_Center((*this).info.bound);
    let mut r2: f64 = 0.0f64;
    let mut v_0: *mut Vertex = (*this).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v_0 < __iterend_0 {
        let mut dx: f64 = ((*v_0).p.x - center.x) as f64;
        let mut dy: f64 = ((*v_0).p.y - center.y) as f64;
        let mut dz: f64 = ((*v_0).p.z - center.z) as f64;
        r2 = Max(r2, dx * dx + dy * dy + dz * dz);
        v_0 = v_0.offset(1);
    }
    (*this).info.radius = Sqrt(r2) as f32;
    (*this).versionInfo = (*this).version;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Create() -> *mut Mesh {
    let mut this: *mut Mesh = MemAlloc(::core::mem::size_of::<Mesh>())
        as *mut Mesh;
    (*this)._refCount = 1 as libc::c_int as uint32;
    (*this).vbo = 0 as libc::c_int as uint;
    (*this).ibo = 0 as libc::c_int as uint;
    (*this).version = 1 as libc::c_int as uint64;
    (*this).versionBuffers = 0 as libc::c_int as uint64;
    (*this).versionInfo = 0 as libc::c_int as uint64;
    (*this).vertex_capacity = 0 as libc::c_int;
    (*this).vertex_size = 0 as libc::c_int;
    (*this).vertex_data = 0 as *mut Vertex;
    (*this).index_capacity = 0 as libc::c_int;
    (*this).index_size = 0 as libc::c_int;
    (*this).index_data = 0 as *mut int32;
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Clone(mut other: *mut Mesh) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Create();
    if ((*this).index_capacity < (*other).index_size) as libc::c_long
        != 0
    {
        (*this).index_capacity = (*other).index_size;
        let mut elemSize: usize = ::core::mem::size_of::<int32>();
        let mut pData: *mut *mut libc::c_void = &mut (*this).index_data
            as *mut *mut int32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).index_data as *mut libc::c_void,
            ((*this).index_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    if ((*this).vertex_capacity < (*other).vertex_size) as libc::c_long
        != 0
    {
        (*this).vertex_capacity = (*other).vertex_size;
        let mut elemSize_0: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData_0: *mut *mut libc::c_void = &mut (*this).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize_0 as usize),
        );
    }
    (*this).index_size = (*other).index_size;
    (*this).vertex_size = (*other).vertex_size;
    MemCpy(
        (*this).index_data as *mut libc::c_void,
        (*other).index_data as *const libc::c_void,
        (::core::mem::size_of::<libc::c_int>())
            .wrapping_mul((*other).index_size as usize),
    );
    MemCpy(
        (*this).vertex_data as *mut libc::c_void,
        (*other).vertex_data as *const libc::c_void,
        (::core::mem::size_of::<Vertex>())
            .wrapping_mul((*other).vertex_size as usize),
    );
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Load(mut name: cstr) -> *mut Mesh {
    let mut bytes: *mut Bytes = Resource_LoadBytes(ResourceType_Mesh, name);
    let mut this: *mut Mesh = Mesh_FromBytes(bytes);
    Bytes_Free(bytes);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Acquire(mut this: *mut Mesh) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Free(mut this: *mut Mesh) {
    if !this.is_null()
        && {
            (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
            (*this)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        MemFree((*this).vertex_data as *const libc::c_void);
        MemFree((*this).index_data as *const libc::c_void);
        if (*this).vbo != 0 {
            __glewDeleteBuffers
                .expect(
                    "non-null function pointer",
                )(1 as libc::c_int, &mut (*this).vbo);
            __glewDeleteBuffers
                .expect(
                    "non-null function pointer",
                )(1 as libc::c_int, &mut (*this).ibo);
        }
        MemFree(this as *const libc::c_void);
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
    let mut this: *mut Bytes = Bytes_Create(size);
    Bytes_WriteI32(this, vertexCount);
    Bytes_WriteI32(this, indexCount);
    Bytes_Write(
        this,
        (*mesh).vertex_data as *const libc::c_void,
        (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()) as uint32,
    );
    Bytes_Write(
        this,
        (*mesh).index_data as *const libc::c_void,
        (indexCount as usize).wrapping_mul(::core::mem::size_of::<int32>()) as uint32,
    );
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_FromBytes(mut buf: *mut Bytes) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Create();
    let mut vertexCount: int32 = Bytes_ReadI32(buf);
    let mut indexCount: int32 = Bytes_ReadI32(buf);
    Mesh_ReserveVertexData(this, vertexCount);
    Mesh_ReserveIndexData(this, indexCount);
    Bytes_Read(
        buf,
        (*this).vertex_data as *mut libc::c_void,
        (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()) as uint32,
    );
    Bytes_Read(
        buf,
        (*this).index_data as *mut libc::c_void,
        (indexCount as usize).wrapping_mul(::core::mem::size_of::<int32>()) as uint32,
    );
    (*this).vertex_size = vertexCount;
    (*this).index_size = indexCount;
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_FromSDF(mut sdf: *mut SDF) -> *mut Mesh {
    return SDF_ToMesh(sdf);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddIndex(
    mut this: *mut Mesh,
    mut newIndex: libc::c_int,
) {
    if ((*this).index_capacity == (*this).index_size) as libc::c_long
        != 0
    {
        (*this)
            .index_capacity = if (*this).index_capacity != 0 {
            (*this).index_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<int32>();
        let mut pData: *mut *mut libc::c_void = &mut (*this).index_data
            as *mut *mut int32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).index_data as *mut libc::c_void,
            ((*this).index_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh0 = (*this).index_size;
    (*this).index_size = (*this).index_size + 1;
    *((*this).index_data).offset(fresh0 as isize) = newIndex;
    (*this).version = ((*this).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddMesh(mut this: *mut Mesh, mut other: *mut Mesh) {
    let mut indexOffset: libc::c_int = (*this).vertex_size;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*other).vertex_size {
        Mesh_AddVertexRaw(this, ((*other).vertex_data).offset(i as isize));
        i += 1;
    }
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < (*other).index_size {
        Mesh_AddIndex(this, *((*other).index_data).offset(i_0 as isize) + indexOffset);
        i_0 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddQuad(
    mut this: *mut Mesh,
    mut i1: libc::c_int,
    mut i2: libc::c_int,
    mut i3: libc::c_int,
    mut i4: libc::c_int,
) {
    Mesh_AddTri(this, i1, i2, i3);
    Mesh_AddTri(this, i1, i3, i4);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddTri(
    mut this: *mut Mesh,
    mut i1: libc::c_int,
    mut i2: libc::c_int,
    mut i3: libc::c_int,
) {
    Mesh_AddIndex(this, i1);
    Mesh_AddIndex(this, i2);
    Mesh_AddIndex(this, i3);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertex(
    mut this: *mut Mesh,
    mut px: f32,
    mut py: f32,
    mut pz: f32,
    mut nx: f32,
    mut ny: f32,
    mut nz: f32,
    mut u: f32,
    mut v: f32,
) {
    if ((*this).vertex_capacity == (*this).vertex_size) as libc::c_int
        as libc::c_long != 0
    {
        (*this)
            .vertex_capacity = if (*this).vertex_capacity != 0 {
            (*this).vertex_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut (*this).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh1 = (*this).vertex_size;
    (*this).vertex_size = (*this).vertex_size + 1;
    let mut newVertex: *mut Vertex = ((*this).vertex_data).offset(fresh1 as isize);
    (*newVertex).p = Vec3::new(px, py, pz);
    (*newVertex).n = Vec3::new(nx, ny, nz);
    (*newVertex).uv = Vec2::new(u, v);
    (*this).version = ((*this).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertexRaw(
    mut this: *mut Mesh,
    mut vertex: *const Vertex,
) {
    if ((*this).vertex_capacity == (*this).vertex_size) as libc::c_int
        as libc::c_long != 0
    {
        (*this)
            .vertex_capacity = if (*this).vertex_capacity != 0 {
            (*this).vertex_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut (*this).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh2 = (*this).vertex_size;
    (*this).vertex_size = (*this).vertex_size + 1;
    *((*this).vertex_data).offset(fresh2 as isize) = *vertex;
    (*this).version = ((*this).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBind(mut this: *mut Mesh) {
    if (*this).vbo != 0 && (*this).version != (*this).versionBuffers {
        __glewDeleteBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*this).vbo);
        __glewDeleteBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*this).ibo);
        (*this).vbo = 0 as libc::c_int as uint;
        (*this).ibo = 0 as libc::c_int as uint;
    }
    if (*this).vbo == 0 {
        __glewGenBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*this).vbo);
        __glewGenBuffers
            .expect("non-null function pointer")(1 as libc::c_int, &mut (*this).ibo);
        __glewBindBuffer
            .expect(
                "non-null function pointer",
            )(0x8892 as libc::c_int as GLenum, (*this).vbo);
        __glewBindBuffer
            .expect(
                "non-null function pointer",
            )(0x8893 as libc::c_int as GLenum, (*this).ibo);
        __glewBufferData
            .expect(
                "non-null function pointer",
            )(
            0x8892 as libc::c_int as GLenum,
            ((*this).vertex_size as usize).wrapping_mul(::core::mem::size_of::<Vertex>())
                as GLsizeiptr,
            (*this).vertex_data as *const libc::c_void,
            0x88e4 as libc::c_int as GLenum,
        );
        __glewBufferData
            .expect(
                "non-null function pointer",
            )(
            0x8893 as libc::c_int as GLenum,
            ((*this).index_size as usize).wrapping_mul(::core::mem::size_of::<libc::c_int>())
                as GLsizeiptr,
            (*this).index_data as *const libc::c_void,
            0x88e4 as libc::c_int as GLenum,
        );
        (*this).versionBuffers = (*this).version;
    }
    __glewBindBuffer
        .expect(
            "non-null function pointer",
        )(0x8892 as libc::c_int as GLenum, (*this).vbo);
    __glewBindBuffer
        .expect(
            "non-null function pointer",
        )(0x8893 as libc::c_int as GLenum, (*this).ibo);
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
pub unsafe extern "C" fn Mesh_DrawBound(mut this: *mut Mesh) {
    Metric_AddDraw(
        (*this).index_size / 3 as libc::c_int,
        (*this).index_size / 3 as libc::c_int,
        (*this).vertex_size,
    );
    glDrawElements(
        0x4 as libc::c_int as GLenum,
        (*this).index_size,
        0x1405 as libc::c_int as GLenum,
        0 as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawUnbind(mut this: *mut Mesh) {
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
pub unsafe extern "C" fn Mesh_Draw(mut this: *mut Mesh) {
    Mesh_DrawBind(this);
    Mesh_DrawBound(this);
    Mesh_DrawUnbind(this);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawNormals(
    mut this: *mut Mesh,
    mut scale: f32,
) {
    glBegin(0x1 as libc::c_int as GLenum);
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
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
pub unsafe extern "C" fn Mesh_GetBound(mut this: *mut Mesh, mut out: *mut Box3f) {
    Mesh_UpdateInfo(this);
    *out = (*this).info.bound;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetCenter(mut this: *mut Mesh, mut out: *mut Vec3) {
    Mesh_UpdateInfo(this);
    *out = Box3f_Center((*this).info.bound);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexCount(mut this: *mut Mesh) -> libc::c_int {
    return (*this).index_size;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexData(mut this: *mut Mesh) -> *mut libc::c_int {
    return (*this).index_data;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetRadius(mut this: *mut Mesh) -> f32 {
    Mesh_UpdateInfo(this);
    return (*this).info.radius;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVersion(mut this: *mut Mesh) -> uint64 {
    return (*this).version;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_IncVersion(mut this: *mut Mesh) {
    (*this).version = ((*this).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Validate(mut this: *mut Mesh) -> Error {
    let mut indexLen: int32 = Mesh_GetIndexCount(this);
    let mut indexData: *mut int32 = Mesh_GetIndexData(this);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(this);
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
    let mut v: *const Vertex = (*this).vertex_data;
    let mut __iterend: *const Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
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
    mut this: *mut Mesh,
    mut index: libc::c_int,
) -> *mut Vertex {
    return ((*this).vertex_data).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexCount(mut this: *mut Mesh) -> libc::c_int {
    return (*this).vertex_size;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexData(mut this: *mut Mesh) -> *mut Vertex {
    return (*this).vertex_data;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveIndexData(
    mut this: *mut Mesh,
    mut capacity: libc::c_int,
) {
    if ((*this).index_capacity < capacity) as libc::c_long != 0 {
        (*this).index_capacity = capacity;
        let mut elemSize: usize = ::core::mem::size_of::<int32>();
        let mut pData: *mut *mut libc::c_void = &mut (*this).index_data
            as *mut *mut int32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).index_data as *mut libc::c_void,
            ((*this).index_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveVertexData(
    mut this: *mut Mesh,
    mut capacity: libc::c_int,
) {
    if ((*this).vertex_capacity < capacity) as libc::c_long != 0 {
        (*this).vertex_capacity = capacity;
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>() as usize;
        let mut pData: *mut *mut libc::c_void = &mut (*this).vertex_data
            as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Center(mut this: *mut Mesh) -> *mut Mesh {
    let mut c = Vec3::ZERO;
    Mesh_GetCenter(this, &mut c);
    Mesh_Translate(this, -c.x, -c.y, -c.z);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Invert(mut this: *mut Mesh) -> *mut Mesh {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*this).index_size {
        let mut swap_temp: [libc::c_uchar; 4] = [0; 4];
        libc::memcpy(
            swap_temp.as_mut_ptr() as *mut libc::c_void,
            &mut *((*this).index_data).offset((i + 2 as libc::c_int) as isize)
                as *mut int32 as *const libc::c_void,
            ::core::mem::size_of::<int32>() as usize,
        );
        libc::memcpy(
            &mut *((*this).index_data).offset((i + 2 as libc::c_int) as isize)
                as *mut int32 as *mut libc::c_void,
            &mut *((*this).index_data).offset((i + 1 as libc::c_int) as isize)
                as *mut int32 as *const libc::c_void,
            ::core::mem::size_of::<int32>() as usize,
        );
        libc::memcpy(
            &mut *((*this).index_data).offset((i + 1 as libc::c_int) as isize)
                as *mut int32 as *mut libc::c_void,
            swap_temp.as_mut_ptr() as *const libc::c_void,
            ::core::mem::size_of::<int32>() as usize,
        );
        i += 3 as libc::c_int;
    }
    (*this).version = ((*this).version).wrapping_add(1);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateX(
    mut this: *mut Mesh,
    mut rads: f32,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationX(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateY(
    mut this: *mut Mesh,
    mut rads: f32,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationY(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateZ(
    mut this: *mut Mesh,
    mut rads: f32,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationZ(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateYPR(
    mut this: *mut Mesh,
    mut yaw: f32,
    mut pitch: f32,
    mut roll: f32,
) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_YawPitchRoll(yaw, pitch, roll);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Scale(
    mut this: *mut Mesh,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).p.x *= x;
        (*v).p.y *= y;
        (*v).p.z *= z;
        v = v.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ScaleUniform(
    mut this: *mut Mesh,
    mut s: f32,
) -> *mut Mesh {
    Mesh_Scale(this, s, s, s);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Translate(
    mut this: *mut Mesh,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).p.x += x;
        (*v).p.y += y;
        (*v).p.z += z;
        v = v.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Transform(
    mut this: *mut Mesh,
    mut matrix: *mut Matrix,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v < __iterend {
        Matrix_MulPoint(matrix, &mut (*v).p, (*v).p.x, (*v).p.y, (*v).p.z);
        v = v.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeNormals(mut this: *mut Mesh) {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).n.x = 0.0f32;
        (*v).n.y = 0.0f32;
        (*v).n.z = 0.0f32;
        v = v.offset(1);
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*this).index_size {
        let mut v1: *mut Vertex = ((*this).vertex_data)
            .offset(
                *((*this).index_data).offset((i + 0 as libc::c_int) as isize) as isize,
            );
        let mut v2: *mut Vertex = ((*this).vertex_data)
            .offset(
                *((*this).index_data).offset((i + 1 as libc::c_int) as isize) as isize,
            );
        let mut v3: *mut Vertex = ((*this).vertex_data)
            .offset(
                *((*this).index_data).offset((i + 2 as libc::c_int) as isize) as isize,
            );
        let mut e1: Vec3 = (*v2).p - (*v1).p;
        let mut e2: Vec3 = (*v3).p - (*v2).p;
        let mut en: Vec3 = Vec3::cross(e1, e2);
        (*v1).n = (*v1).n + en;
        (*v2).n = (*v2).n + en;
        (*v3).n = (*v3).n + en;
        i += 3 as libc::c_int;
    }
    let mut v_0: *mut Vertex = (*this).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v_0 < __iterend_0 {
        (*v_0).n = (*v_0).n.normalize();
        v_0 = v_0.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_SplitNormals(
    mut this: *mut Mesh,
    mut minDot: f32,
) {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v)
            .n = Vec3::new(
            0.0f32,
            0.0f32,
            0.0f32,
        );
        v = v.offset(1);
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*this).index_size {
        let mut index: [*mut int32; 3] = [
            ((*this).index_data).offset(i as isize).offset(0),
            ((*this).index_data).offset(i as isize).offset(1),
            ((*this).index_data).offset(i as isize).offset(2),
        ];
        let mut v_0: [*mut Vertex; 3] = [
            ((*this).vertex_data).offset(*index[0] as isize),
            ((*this).vertex_data).offset(*index[1] as isize),
            ((*this).vertex_data).offset(*index[2] as isize),
        ];
        let mut face: Vec3 = Vec3::cross(
            (*v_0[1]).p - (*v_0[0]).p,
            (*v_0[2]).p - (*v_0[0]).p,
        );
        let mut j: libc::c_int = 0 as libc::c_int;
        while j < 3 as libc::c_int {
            let mut cn: *mut Vec3 = &mut (*((*this).vertex_data)
                .offset(**index.as_mut_ptr().offset(j as isize) as isize))
                .n;
            if (*cn).length_squared() > 0.0f32 {
                let mut cDot: f32 = Vec3::dot(
                    face.normalize(),
                    (*cn).normalize(),
                );
                if cDot < minDot {
                    if ((*this).vertex_capacity == (*this).vertex_size)
                        as libc::c_long != 0
                    {
                        (*this)
                            .vertex_capacity = if (*this).vertex_capacity != 0 {
                            (*this).vertex_capacity * 2 as libc::c_int
                        } else {
                            1 as libc::c_int
                        };
                        let mut elemSize: usize = ::core::mem::size_of::<Vertex>();
                        let mut pData: *mut *mut libc::c_void = &mut (*this)
                            .vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
                        *pData = MemRealloc(
                            (*this).vertex_data as *mut libc::c_void,
                            ((*this).vertex_capacity as usize)
                                .wrapping_mul(elemSize),
                        );
                    }
                    let fresh3 = (*this).vertex_size;
                    (*this).vertex_size = (*this).vertex_size + 1;
                    let mut nv: *mut Vertex = ((*this).vertex_data)
                        .offset(fresh3 as isize);
                    *nv = *((*this).vertex_data).offset(*index[j as usize] as isize);
                    (*nv).n = face;
                    *index[j as usize] = (*this).vertex_size - 1 as libc::c_int;
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
    let mut v_1: *mut Vertex = (*this).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*this).vertex_data)
        .offset((*this).vertex_size as isize);
    while v_1 < __iterend_0 {
        (*v_1).n = (*v_1).n.normalize();
        v_1 = v_1.offset(1);
    }
}
