use crate::internal::Memory::*;
use crate::Bytes::*;
use crate::Matrix::*;
use crate::Metric::*;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::Triangle::*;
use crate::SDF::*;
use glam::Vec2;
use glam::Vec3;
use libc;
use memoffset::{offset_of, span_of};

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn sqrt(_: f64) -> f64;
    fn glBegin(mode: GLenum);
    fn glDrawElements(mode: GLenum, count: GLsizei, type_0: GLenum, indices: *const libc::c_void);
    fn glEnd();
    fn glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat);
    static mut __glewBindBuffer: PFNGLBINDBUFFERPROC;
    static mut __glewBufferData: PFNGLBUFFERDATAPROC;
    static mut __glewDeleteBuffers: PFNGLDELETEBUFFERSPROC;
    static mut __glewGenBuffers: PFNGLGENBUFFERSPROC;
    static mut __glewDisableVertexAttribArray: PFNGLDISABLEVERTEXATTRIBARRAYPROC;
    static mut __glewEnableVertexAttribArray: PFNGLENABLEVERTEXATTRIBARRAYPROC;
    static mut __glewVertexAttribPointer: PFNGLVERTEXATTRIBPOINTERPROC;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Mesh {
    pub _refCount: u32,
    pub vbo: u32,
    pub ibo: u32,
    pub version: u64,
    pub versionBuffers: u64,
    pub versionInfo: u64,
    pub info: Computed,
    pub index_size: i32,
    pub index_capacity: i32,
    pub index_data: *mut i32,
    pub vertex_size: i32,
    pub vertex_capacity: i32,
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

pub type ResourceType = i32;
pub type GLu32 = u32;
pub type PFNGLDELETEBUFFERSPROC = Option<unsafe extern "C" fn(GLsizei, *const GLu32) -> ()>;
pub type GLsizei = i32;
pub type GLenum = u32;
pub type PFNGLBINDBUFFERPROC = Option<unsafe extern "C" fn(GLenum, GLu32) -> ()>;
pub type PFNGLDISABLEVERTEXATTRIBARRAYPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type GLboolean = libc::c_uchar;
pub type PFNGLVERTEXATTRIBPOINTERPROC = Option<
    unsafe extern "C" fn(GLu32, GLint, GLenum, GLboolean, GLsizei, *const libc::c_void) -> (),
>;
pub type GLint = i32;
pub type PFNGLENABLEVERTEXATTRIBARRAYPROC = Option<unsafe extern "C" fn(GLu32) -> ()>;
pub type GLsizeiptr = libc::ptrdiff_t;
pub type PFNGLBUFFERDATAPROC =
    Option<unsafe extern "C" fn(GLenum, GLsizeiptr, *const libc::c_void, GLenum) -> ()>;
pub type PFNGLGENBUFFERSPROC = Option<unsafe extern "C" fn(GLsizei, *mut GLu32) -> ()>;
pub type GLfloat = f32;

#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    sqrt(t as f64) as f32
}

#[inline]
unsafe extern "C" fn Maxf(mut a: f32, mut b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
unsafe extern "C" fn Max(mut a: f64, mut b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
unsafe extern "C" fn Minf(mut a: f32, mut b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
unsafe extern "C" fn Sqrt(mut t: f64) -> f64 {
    sqrt(t)
}

#[inline]
unsafe extern "C" fn Box3f_Center(mut this: Box3f) -> Vec3 {
    let mut center: Vec3 = Vec3 {
        x: (this.lower.x + this.upper.x) / 2.0f32,
        y: (this.lower.y + this.upper.y) / 2.0f32,
        z: (this.lower.z + this.upper.z) / 2.0f32,
    };
    center
}

#[inline]
unsafe extern "C" fn Box3f_Add(mut this: *mut Box3f, mut point: Vec3) {
    (*this).lower = Vec3::min((*this).lower, point);
    (*this).upper = Vec3::max((*this).upper, point);
}

#[inline]
unsafe extern "C" fn Vec2_Validate(mut v: Vec2) -> Error {
    let mut e: Error = 0_i32 as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    e
}

unsafe extern "C" fn Mesh_UpdateInfo(mut this: *mut Mesh) {
    if (*this).versionInfo == (*this).version {
        return;
    }
    (*this).info.bound.lower = Vec3::new(3.40282347e+38f32, 3.40282347e+38f32, 3.40282347e+38f32);
    (*this).info.bound.upper =
        Vec3::new(-3.40282347e+38f32, -3.40282347e+38f32, -3.40282347e+38f32);
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        Box3f_Add(&mut (*this).info.bound, (*v).p);
        v = v.offset(1);
    }
    let mut center: Vec3 = Box3f_Center((*this).info.bound);
    let mut r2: f64 = 0.0f64;
    let mut v_0: *mut Vertex = (*this).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
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
    let mut this: *mut Mesh = MemAlloc(::core::mem::size_of::<Mesh>()) as *mut Mesh;
    (*this)._refCount = 1_i32 as u32;
    (*this).vbo = 0_i32 as u32;
    (*this).ibo = 0_i32 as u32;
    (*this).version = 1_i32 as u64;
    (*this).versionBuffers = 0_i32 as u64;
    (*this).versionInfo = 0_i32 as u64;
    (*this).vertex_capacity = 0_i32;
    (*this).vertex_size = 0_i32;
    (*this).vertex_data = std::ptr::null_mut();
    (*this).index_capacity = 0_i32;
    (*this).index_size = 0_i32;
    (*this).index_data = std::ptr::null_mut();
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Clone(mut other: *mut Mesh) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Create();
    if ((*this).index_capacity < (*other).index_size) as libc::c_long != 0 {
        (*this).index_capacity = (*other).index_size;
        let mut elemSize: usize = ::core::mem::size_of::<i32>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).index_data as *mut *mut i32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).index_data as *mut libc::c_void,
            ((*this).index_capacity as usize).wrapping_mul(elemSize),
        );
    }
    if ((*this).vertex_capacity < (*other).vertex_size) as libc::c_long != 0 {
        (*this).vertex_capacity = (*other).vertex_size;
        let mut elemSize_0: usize = ::core::mem::size_of::<Vertex>();
        let mut pData_0: *mut *mut libc::c_void =
            &mut (*this).vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize_0),
        );
    }
    (*this).index_size = (*other).index_size;
    (*this).vertex_size = (*other).vertex_size;
    MemCpy(
        (*this).index_data as *mut libc::c_void,
        (*other).index_data as *const libc::c_void,
        (::core::mem::size_of::<i32>()).wrapping_mul((*other).index_size as usize),
    );
    MemCpy(
        (*this).vertex_data as *mut libc::c_void,
        (*other).vertex_data as *const libc::c_void,
        (::core::mem::size_of::<Vertex>()).wrapping_mul((*other).vertex_size as usize),
    );
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Load(mut name: *const libc::c_char) -> *mut Mesh {
    let mut bytes: *mut Bytes = Resource_LoadBytes(ResourceType_Mesh, name);
    let mut this: *mut Mesh = Mesh_FromBytes(bytes);
    Bytes_Free(bytes);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Acquire(mut this: *mut Mesh) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Free(mut this: *mut Mesh) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        MemFree((*this).vertex_data as *const libc::c_void);
        MemFree((*this).index_data as *const libc::c_void);
        if (*this).vbo != 0 {
            __glewDeleteBuffers.expect("non-null function pointer")(1_i32, &mut (*this).vbo);
            __glewDeleteBuffers.expect("non-null function pointer")(1_i32, &mut (*this).ibo);
        }
        MemFree(this as *const libc::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ToBytes(mut mesh: *mut Mesh) -> *mut Bytes {
    let mut vertexCount: i32 = Mesh_GetVertexCount(mesh);
    let mut indexCount: i32 = Mesh_GetIndexCount(mesh);
    let mut size: u32 = 2_usize
        .wrapping_mul(::core::mem::size_of::<i32>())
        .wrapping_add((vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()))
        .wrapping_add((indexCount as usize).wrapping_mul(::core::mem::size_of::<i32>()))
        as u32;
    let mut this: *mut Bytes = Bytes_Create(size);
    Bytes_WriteI32(this, vertexCount);
    Bytes_WriteI32(this, indexCount);
    Bytes_Write(
        this,
        (*mesh).vertex_data as *const libc::c_void,
        (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()) as u32,
    );
    Bytes_Write(
        this,
        (*mesh).index_data as *const libc::c_void,
        (indexCount as usize).wrapping_mul(::core::mem::size_of::<i32>()) as u32,
    );
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromBytes(mut buf: *mut Bytes) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Create();
    let mut vertexCount: i32 = Bytes_ReadI32(buf);
    let mut indexCount: i32 = Bytes_ReadI32(buf);
    Mesh_ReserveVertexData(this, vertexCount);
    Mesh_ReserveIndexData(this, indexCount);
    Bytes_Read(
        buf,
        (*this).vertex_data as *mut libc::c_void,
        (vertexCount as usize).wrapping_mul(::core::mem::size_of::<Vertex>()) as u32,
    );
    Bytes_Read(
        buf,
        (*this).index_data as *mut libc::c_void,
        (indexCount as usize).wrapping_mul(::core::mem::size_of::<i32>()) as u32,
    );
    (*this).vertex_size = vertexCount;
    (*this).index_size = indexCount;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromSDF(mut sdf: *mut SDF) -> *mut Mesh {
    SDF_ToMesh(sdf)
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddIndex(mut this: *mut Mesh, mut newIndex: i32) {
    if ((*this).index_capacity == (*this).index_size) as libc::c_long != 0 {
        (*this).index_capacity = if (*this).index_capacity != 0 {
            (*this).index_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = ::core::mem::size_of::<i32>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).index_data as *mut *mut i32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).index_data as *mut libc::c_void,
            ((*this).index_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh0 = (*this).index_size;
    (*this).index_size += 1;
    *((*this).index_data).offset(fresh0 as isize) = newIndex;
    (*this).version = ((*this).version).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddMesh(mut this: *mut Mesh, mut other: *mut Mesh) {
    let mut indexOffset: i32 = (*this).vertex_size;
    let mut i: i32 = 0_i32;
    while i < (*other).vertex_size {
        Mesh_AddVertexRaw(this, ((*other).vertex_data).offset(i as isize));
        i += 1;
    }
    let mut i_0: i32 = 0_i32;
    while i_0 < (*other).index_size {
        Mesh_AddIndex(
            this,
            *((*other).index_data).offset(i_0 as isize) + indexOffset,
        );
        i_0 += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddQuad(
    mut this: *mut Mesh,
    mut i1: i32,
    mut i2: i32,
    mut i3: i32,
    mut i4: i32,
) {
    Mesh_AddTri(this, i1, i2, i3);
    Mesh_AddTri(this, i1, i3, i4);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddTri(mut this: *mut Mesh, mut i1: i32, mut i2: i32, mut i3: i32) {
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
    if ((*this).vertex_capacity == (*this).vertex_size) as i32 as libc::c_long != 0 {
        (*this).vertex_capacity = if (*this).vertex_capacity != 0 {
            (*this).vertex_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh1 = (*this).vertex_size;
    (*this).vertex_size += 1;
    let mut newVertex: *mut Vertex = ((*this).vertex_data).offset(fresh1 as isize);
    (*newVertex).p = Vec3::new(px, py, pz);
    (*newVertex).n = Vec3::new(nx, ny, nz);
    (*newVertex).uv = Vec2::new(u, v);
    (*this).version = ((*this).version).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertexRaw(mut this: *mut Mesh, mut vertex: *const Vertex) {
    if ((*this).vertex_capacity == (*this).vertex_size) as i32 as libc::c_long != 0 {
        (*this).vertex_capacity = if (*this).vertex_capacity != 0 {
            (*this).vertex_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh2 = (*this).vertex_size;
    (*this).vertex_size += 1;
    *((*this).vertex_data).offset(fresh2 as isize) = *vertex;
    (*this).version = ((*this).version).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBind(mut this: *mut Mesh) {
    if (*this).vbo != 0 && (*this).version != (*this).versionBuffers {
        __glewDeleteBuffers.expect("non-null function pointer")(1_i32, &mut (*this).vbo);
        __glewDeleteBuffers.expect("non-null function pointer")(1_i32, &mut (*this).ibo);
        (*this).vbo = 0_i32 as u32;
        (*this).ibo = 0_i32 as u32;
    }
    if (*this).vbo == 0 {
        __glewGenBuffers.expect("non-null function pointer")(1_i32, &mut (*this).vbo);
        __glewGenBuffers.expect("non-null function pointer")(1_i32, &mut (*this).ibo);
        __glewBindBuffer.expect("non-null function pointer")(0x8892_i32 as GLenum, (*this).vbo);
        __glewBindBuffer.expect("non-null function pointer")(0x8893_i32 as GLenum, (*this).ibo);
        __glewBufferData.expect("non-null function pointer")(
            0x8892_i32 as GLenum,
            ((*this).vertex_size as usize).wrapping_mul(::core::mem::size_of::<Vertex>())
                as GLsizeiptr,
            (*this).vertex_data as *const libc::c_void,
            0x88e4_i32 as GLenum,
        );
        __glewBufferData.expect("non-null function pointer")(
            0x8893_i32 as GLenum,
            ((*this).index_size as usize).wrapping_mul(::core::mem::size_of::<i32>()) as GLsizeiptr,
            (*this).index_data as *const libc::c_void,
            0x88e4_i32 as GLenum,
        );
        (*this).versionBuffers = (*this).version;
    }
    __glewBindBuffer.expect("non-null function pointer")(0x8892_i32 as GLenum, (*this).vbo);
    __glewBindBuffer.expect("non-null function pointer")(0x8893_i32 as GLenum, (*this).ibo);
    __glewEnableVertexAttribArray.expect("non-null function pointer")(0_i32 as GLu32);
    __glewEnableVertexAttribArray.expect("non-null function pointer")(1_i32 as GLu32);
    __glewEnableVertexAttribArray.expect("non-null function pointer")(2_i32 as GLu32);
    __glewVertexAttribPointer.expect("non-null function pointer")(
        0_i32 as GLu32,
        3_i32,
        0x1406_i32 as GLenum,
        0_i32 as GLboolean,
        ::core::mem::size_of::<Vertex>() as GLsizei,
        offset_of!(Vertex, p) as *const libc::c_void,
    );
    __glewVertexAttribPointer.expect("non-null function pointer")(
        1_i32 as GLu32,
        3_i32,
        0x1406_i32 as GLenum,
        0_i32 as GLboolean,
        ::core::mem::size_of::<Vertex>() as GLsizei,
        offset_of!(Vertex, n) as *const libc::c_void,
    );
    __glewVertexAttribPointer.expect("non-null function pointer")(
        2_i32 as GLu32,
        2_i32,
        0x1406_i32 as GLenum,
        0_i32 as GLboolean,
        ::core::mem::size_of::<Vertex>() as GLsizei,
        offset_of!(Vertex, uv) as *const libc::c_void,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBound(mut this: *mut Mesh) {
    Metric_AddDraw(
        (*this).index_size / 3_i32,
        (*this).index_size / 3_i32,
        (*this).vertex_size,
    );
    glDrawElements(
        0x4_i32 as GLenum,
        (*this).index_size,
        0x1405_i32 as GLenum,
        std::ptr::null(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawUnbind(mut _this: *mut Mesh) {
    __glewDisableVertexAttribArray.expect("non-null function pointer")(0_i32 as GLu32);
    __glewDisableVertexAttribArray.expect("non-null function pointer")(1_i32 as GLu32);
    __glewDisableVertexAttribArray.expect("non-null function pointer")(2_i32 as GLu32);
    __glewBindBuffer.expect("non-null function pointer")(0x8892_i32 as GLenum, 0_i32 as GLu32);
    __glewBindBuffer.expect("non-null function pointer")(0x8893_i32 as GLenum, 0_i32 as GLu32);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Draw(mut this: *mut Mesh) {
    Mesh_DrawBind(this);
    Mesh_DrawBound(this);
    Mesh_DrawUnbind(this);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawNormals(mut this: *mut Mesh, mut scale: f32) {
    glBegin(0x1_i32 as GLenum);
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
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
pub unsafe extern "C" fn Mesh_GetIndexCount(mut this: *mut Mesh) -> i32 {
    (*this).index_size
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexData(mut this: *mut Mesh) -> *mut i32 {
    (*this).index_data
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetRadius(mut this: *mut Mesh) -> f32 {
    Mesh_UpdateInfo(this);
    (*this).info.radius
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVersion(mut this: *mut Mesh) -> u64 {
    (*this).version
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_IncVersion(mut this: *mut Mesh) {
    (*this).version = ((*this).version).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Validate(mut this: *mut Mesh) -> Error {
    let mut indexLen: i32 = Mesh_GetIndexCount(this);
    let mut indexData: *mut i32 = Mesh_GetIndexData(this);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(this);
    if indexLen % 3_i32 != 0_i32 {
        return (0x100000_i32 | 0x80_i32) as Error;
    }
    let mut i: i32 = 0_i32;
    while i < indexLen {
        let mut i0: i32 = *indexData.offset((i + 0_i32) as isize);
        let mut i1: i32 = *indexData.offset((i + 1_i32) as isize);
        let mut i2: i32 = *indexData.offset((i + 2_i32) as isize);
        let mut triangle: Triangle = Triangle {
            vertices: [Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            }; 3],
        };
        triangle.vertices[0] = (*vertexData.offset(i0 as isize)).p;
        triangle.vertices[1] = (*vertexData.offset(i1 as isize)).p;
        triangle.vertices[2] = (*vertexData.offset(i2 as isize)).p;
        let mut e: Error = Triangle_Validate(&mut triangle);
        if e != 0_i32 as u32 {
            return 0x400000_i32 as u32 | e;
        }
        i += 3_i32;
    }
    let mut v: *const Vertex = (*this).vertex_data;
    let mut __iterend: *const Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        let mut e_0: Error = 0;
        e_0 = Vec3_Validate((*v).p);
        if e_0 != 0_i32 as u32 {
            return 0x400000_i32 as u32 | e_0;
        }
        e_0 = Vec3_Validate((*v).n);
        if e_0 != 0_i32 as u32 {
            return 0x800000_i32 as u32 | e_0;
        }
        e_0 = Vec2_Validate((*v).uv);
        if e_0 != 0_i32 as u32 {
            return 0x1000000_i32 as u32 | e_0;
        }
        v = v.offset(1);
    }
    0_i32 as Error
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertex(mut this: *mut Mesh, mut index: i32) -> *mut Vertex {
    ((*this).vertex_data).offset(index as isize)
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexCount(mut this: *mut Mesh) -> i32 {
    (*this).vertex_size
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexData(mut this: *mut Mesh) -> *mut Vertex {
    (*this).vertex_data
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveIndexData(mut this: *mut Mesh, mut capacity: i32) {
    if ((*this).index_capacity < capacity) as libc::c_long != 0 {
        (*this).index_capacity = capacity;
        let mut elemSize: usize = ::core::mem::size_of::<i32>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).index_data as *mut *mut i32 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).index_data as *mut libc::c_void,
            ((*this).index_capacity as usize).wrapping_mul(elemSize),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveVertexData(mut this: *mut Mesh, mut capacity: i32) {
    if ((*this).vertex_capacity < capacity) as libc::c_long != 0 {
        (*this).vertex_capacity = capacity;
        let mut elemSize: usize = ::core::mem::size_of::<Vertex>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).vertex_data as *mut libc::c_void,
            ((*this).vertex_capacity as usize).wrapping_mul(elemSize),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Center(mut this: *mut Mesh) -> *mut Mesh {
    let mut c = Vec3::ZERO;
    Mesh_GetCenter(this, &mut c);
    Mesh_Translate(this, -c.x, -c.y, -c.z);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Invert(mut this: *mut Mesh) -> *mut Mesh {
    let mut i: i32 = 0_i32;
    while i < (*this).index_size {
        let mut swap_temp: [libc::c_uchar; 4] = [0; 4];
        libc::memcpy(
            swap_temp.as_mut_ptr() as *mut libc::c_void,
            &mut *((*this).index_data).offset((i + 2_i32) as isize) as *mut i32
                as *const libc::c_void,
            ::core::mem::size_of::<i32>(),
        );
        libc::memcpy(
            &mut *((*this).index_data).offset((i + 2_i32) as isize) as *mut i32
                as *mut libc::c_void,
            &mut *((*this).index_data).offset((i + 1_i32) as isize) as *mut i32
                as *const libc::c_void,
            ::core::mem::size_of::<i32>(),
        );
        libc::memcpy(
            &mut *((*this).index_data).offset((i + 1_i32) as isize) as *mut i32
                as *mut libc::c_void,
            swap_temp.as_mut_ptr() as *const libc::c_void,
            ::core::mem::size_of::<i32>(),
        );
        i += 3_i32;
    }
    (*this).version = ((*this).version).wrapping_add(1);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateX(mut this: *mut Mesh, mut rads: f32) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationX(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateY(mut this: *mut Mesh, mut rads: f32) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationY(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateZ(mut this: *mut Mesh, mut rads: f32) -> *mut Mesh {
    let mut matrix: *mut Matrix = Matrix_RotationZ(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
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
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Scale(
    mut this: *mut Mesh,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).p.x *= x;
        (*v).p.y *= y;
        (*v).p.z *= z;
        v = v.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ScaleUniform(mut this: *mut Mesh, mut s: f32) -> *mut Mesh {
    Mesh_Scale(this, s, s, s);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Translate(
    mut this: *mut Mesh,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) -> *mut Mesh {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).p.x += x;
        (*v).p.y += y;
        (*v).p.z += z;
        v = v.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Transform(mut this: *mut Mesh, mut matrix: *mut Matrix) -> *mut Mesh {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        Matrix_MulPoint(matrix, &mut (*v).p, (*v).p.x, (*v).p.y, (*v).p.z);
        v = v.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeNormals(mut this: *mut Mesh) {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).n.x = 0.0f32;
        (*v).n.y = 0.0f32;
        (*v).n.z = 0.0f32;
        v = v.offset(1);
    }
    let mut i: i32 = 0_i32;
    while i < (*this).index_size {
        let mut v1: *mut Vertex = ((*this).vertex_data)
            .offset(*((*this).index_data).offset((i + 0_i32) as isize) as isize);
        let mut v2: *mut Vertex = ((*this).vertex_data)
            .offset(*((*this).index_data).offset((i + 1_i32) as isize) as isize);
        let mut v3: *mut Vertex = ((*this).vertex_data)
            .offset(*((*this).index_data).offset((i + 2_i32) as isize) as isize);
        let mut e1: Vec3 = (*v2).p - (*v1).p;
        let mut e2: Vec3 = (*v3).p - (*v2).p;
        let mut en: Vec3 = Vec3::cross(e1, e2);
        (*v1).n += en;
        (*v2).n += en;
        (*v3).n += en;
        i += 3_i32;
    }
    let mut v_0: *mut Vertex = (*this).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v_0 < __iterend_0 {
        (*v_0).n = (*v_0).n.normalize();
        v_0 = v_0.offset(1);
    }
    (*this).version = ((*this).version).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_SplitNormals(mut this: *mut Mesh, mut minDot: f32) {
    let mut v: *mut Vertex = (*this).vertex_data;
    let mut __iterend: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v < __iterend {
        (*v).n = Vec3::new(0.0f32, 0.0f32, 0.0f32);
        v = v.offset(1);
    }
    let mut i: i32 = 0_i32;
    while i < (*this).index_size {
        let mut index: [*mut i32; 3] = [
            ((*this).index_data).offset(i as isize).offset(0),
            ((*this).index_data).offset(i as isize).offset(1),
            ((*this).index_data).offset(i as isize).offset(2),
        ];
        let mut v_0: [*mut Vertex; 3] = [
            ((*this).vertex_data).offset(*index[0] as isize),
            ((*this).vertex_data).offset(*index[1] as isize),
            ((*this).vertex_data).offset(*index[2] as isize),
        ];
        let mut face: Vec3 = Vec3::cross((*v_0[1]).p - (*v_0[0]).p, (*v_0[2]).p - (*v_0[0]).p);
        let mut j: i32 = 0_i32;
        while j < 3_i32 {
            let mut cn: *mut Vec3 = &mut (*((*this).vertex_data)
                .offset(**index.as_mut_ptr().offset(j as isize) as isize))
            .n;
            if (*cn).length_squared() > 0.0f32 {
                let mut cDot: f32 = Vec3::dot(face.normalize(), (*cn).normalize());
                if cDot < minDot {
                    if ((*this).vertex_capacity == (*this).vertex_size) as libc::c_long != 0 {
                        (*this).vertex_capacity = if (*this).vertex_capacity != 0 {
                            (*this).vertex_capacity * 2_i32
                        } else {
                            1_i32
                        };
                        let mut elemSize: usize = ::core::mem::size_of::<Vertex>();
                        let mut pData: *mut *mut libc::c_void =
                            &mut (*this).vertex_data as *mut *mut Vertex as *mut *mut libc::c_void;
                        *pData = MemRealloc(
                            (*this).vertex_data as *mut libc::c_void,
                            ((*this).vertex_capacity as usize).wrapping_mul(elemSize),
                        );
                    }
                    let fresh3 = (*this).vertex_size;
                    (*this).vertex_size += 1;
                    let mut nv: *mut Vertex = ((*this).vertex_data).offset(fresh3 as isize);
                    *nv = *((*this).vertex_data).offset(*index[j as usize] as isize);
                    (*nv).n = face;
                    *index[j as usize] = (*this).vertex_size - 1_i32;
                } else {
                    (*cn) += face;
                }
            } else {
                (*cn) += face;
            }
            j += 1;
        }
        i += 3_i32;
    }
    let mut v_1: *mut Vertex = (*this).vertex_data;
    let mut __iterend_0: *mut Vertex = ((*this).vertex_data).offset((*this).vertex_size as isize);
    while v_1 < __iterend_0 {
        (*v_1).n = (*v_1).n.normalize();
        v_1 = v_1.offset(1);
    }
}
