use crate::internal::Memory::*;
use crate::Bytes::*;
use crate::Common::*;
use crate::Math::*;
use crate::Matrix::*;
use crate::Metric::*;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::Triangle::*;
use crate::GL::gl;
use crate::SDF::*;
use libc;
use memoffset::{offset_of, span_of};

#[derive(Clone)]
#[repr(C)]
pub struct Mesh {
    pub _refCount: u32,
    pub vbo: u32,
    pub ibo: u32,
    pub version: u64,
    pub versionBuffers: u64,
    pub versionInfo: u64,
    pub info: Computed,
    pub index: Vec<i32>,
    pub vertex: Vec<Vertex>,
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Computed {
    pub bound: Box3,
    pub radius: f32,
}

#[inline]
unsafe extern "C" fn Vec2_Validate(v: Vec2) -> Error {
    let mut e: Error = 0 as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    e
}

unsafe extern "C" fn Mesh_UpdateInfo(this: *mut Mesh) {
    if (*this).versionInfo == (*this).version {
        return;
    }

    (*this).info.bound.lower = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    (*this).info.bound.upper = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for v in (*this).vertex.iter() {
        (*this).info.bound.add((*v).p);
    }

    let center: Vec3 = (*this).info.bound.center();
    let mut r2: f64 = 0.0f64;
    for v in (*this).vertex.iter() {
        let dx: f64 = ((*v).p.x - center.x) as f64;
        let dy: f64 = ((*v).p.y - center.y) as f64;
        let dz: f64 = ((*v).p.z - center.z) as f64;
        r2 = f64::max(r2, dx * dx + dy * dy + dz * dz);
    }
    (*this).info.radius = f64::sqrt(r2) as f32;
    (*this).versionInfo = (*this).version;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Create() -> *mut Mesh {
    let this = MemNew!(Mesh);
    (*this)._refCount = 1;
    (*this).vbo = 0;
    (*this).ibo = 0;
    (*this).version = 1;
    (*this).versionBuffers = 0;
    (*this).versionInfo = 0;
    (*this).vertex = Vec::new();
    (*this).index = Vec::new();
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Clone(other: *mut Mesh) -> *mut Mesh {
    let this: *mut Mesh = Mesh_Create();
    (*this).index = (*other).index.clone();
    (*this).vertex = (*other).vertex.clone();
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Load(name: *const libc::c_char) -> *mut Mesh {
    let bytes: *mut Bytes = Resource_LoadBytes(ResourceType_Mesh, name);
    let this: *mut Mesh = Mesh_FromBytes(bytes);
    Bytes_Free(bytes);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Acquire(this: *mut Mesh) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Free(this: *mut Mesh) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        if (*this).vbo != 0 {
            gl::DeleteBuffers(1, &mut (*this).vbo);
            gl::DeleteBuffers(1, &mut (*this).ibo);
        }
        MemDelete!(this);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ToBytes(mesh: *mut Mesh) -> *mut Bytes {
    let vertexCount: i32 = Mesh_GetVertexCount(mesh);
    let indexCount: i32 = Mesh_GetIndexCount(mesh);
    let size: u32 = 2_usize
        .wrapping_mul(std::mem::size_of::<i32>())
        .wrapping_add((vertexCount as usize).wrapping_mul(std::mem::size_of::<Vertex>()))
        .wrapping_add((indexCount as usize).wrapping_mul(std::mem::size_of::<i32>()))
        as u32;
    let this: *mut Bytes = Bytes_Create(size);
    Bytes_WriteI32(this, vertexCount);
    Bytes_WriteI32(this, indexCount);
    Bytes_Write(
        this,
        (*mesh).vertex.as_ptr() as *const _,
        (vertexCount as usize).wrapping_mul(std::mem::size_of::<Vertex>()) as u32,
    );
    Bytes_Write(
        this,
        (*mesh).index.as_ptr() as *const _,
        (indexCount as usize).wrapping_mul(std::mem::size_of::<i32>()) as u32,
    );
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromBytes(buf: *mut Bytes) -> *mut Mesh {
    let this: *mut Mesh = Mesh_Create();
    let vertexCount: i32 = Bytes_ReadI32(buf);
    let indexCount: i32 = Bytes_ReadI32(buf);
    Mesh_ReserveVertexData(this, vertexCount);
    Mesh_ReserveIndexData(this, indexCount);
    (*this)
        .vertex
        .resize(vertexCount as usize, Vertex::default());
    (*this).index.resize(indexCount as usize, 0);
    Bytes_Read(
        buf,
        (*this).vertex.as_mut_ptr() as *mut _,
        (vertexCount as usize).wrapping_mul(std::mem::size_of::<Vertex>()) as u32,
    );
    Bytes_Read(
        buf,
        (*this).index.as_mut_ptr() as *mut _,
        (indexCount as usize).wrapping_mul(std::mem::size_of::<i32>()) as u32,
    );
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromSDF(sdf: *mut SDF) -> *mut Mesh {
    SDF_ToMesh(sdf)
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddIndex(this: *mut Mesh, newIndex: i32) {
    (*this).index.push(newIndex);
    (*this).version += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddMesh(this: *mut Mesh, other: *mut Mesh) {
    let indexOffset: i32 = (*this).vertex.len() as i32;
    for i in 0..(*other).vertex.len() {
        Mesh_AddVertexRaw(this, &mut (*other).vertex[i]);
    }
    for i in 0..(*other).index.len() {
        Mesh_AddIndex(this, (*other).index[i] + indexOffset);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddQuad(this: *mut Mesh, i1: i32, i2: i32, i3: i32, i4: i32) {
    Mesh_AddTri(this, i1, i2, i3);
    Mesh_AddTri(this, i1, i3, i4);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddTri(this: *mut Mesh, i1: i32, i2: i32, i3: i32) {
    Mesh_AddIndex(this, i1);
    Mesh_AddIndex(this, i2);
    Mesh_AddIndex(this, i3);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertex(
    this: *mut Mesh,
    px: f32,
    py: f32,
    pz: f32,
    nx: f32,
    ny: f32,
    nz: f32,
    u: f32,
    v: f32,
) {
    (*this).vertex.push(Vertex {
        p: Vec3::new(px, py, pz),
        n: Vec3::new(nx, ny, nz),
        uv: Vec2::new(u, v),
    });
    (*this).version += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_AddVertexRaw(this: *mut Mesh, vertex: *const Vertex) {
    (*this).vertex.push(*vertex);
    (*this).version += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBind(this: *mut Mesh) {
    /* Release cached GL buffers if the mesh has changed since we built them. */
    if (*this).vbo != 0 && (*this).version != (*this).versionBuffers {
        gl::DeleteBuffers(1, &mut (*this).vbo);
        gl::DeleteBuffers(1, &mut (*this).ibo);
        (*this).vbo = 0;
        (*this).ibo = 0;
    }

    /* Generate cached GL buffers for fast drawing. */
    if (*this).vbo == 0 {
        gl::GenBuffers(1, &mut (*this).vbo);
        gl::GenBuffers(1, &mut (*this).ibo);
        gl::BindBuffer(gl::ARRAY_BUFFER, (*this).vbo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, (*this).ibo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            ((*this).vertex.len() as i32 as usize).wrapping_mul(std::mem::size_of::<Vertex>())
                as gl::types::GLsizeiptr,
            (*this).vertex.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        /* TODO : 16-bit index optimization */
        /* TODO : Check if 8-bit indices are supported by hardware. IIRC they
         *        weren't last time I checked. */

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            ((*this).index.len() as i32 as usize).wrapping_mul(std::mem::size_of::<i32>())
                as gl::types::GLsizeiptr,
            (*this).index.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        (*this).versionBuffers = (*this).version;
    }

    gl::BindBuffer(gl::ARRAY_BUFFER, (*this).vbo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, (*this).ibo);

    gl::EnableVertexAttribArray(0);
    gl::EnableVertexAttribArray(1);
    gl::EnableVertexAttribArray(2);

    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        std::mem::size_of::<Vertex>() as gl::types::GLsizei,
        offset_of!(Vertex, p) as *const _,
    );
    gl::VertexAttribPointer(
        1,
        3,
        gl::FLOAT,
        gl::FALSE,
        std::mem::size_of::<Vertex>() as gl::types::GLsizei,
        offset_of!(Vertex, n) as *const _,
    );
    gl::VertexAttribPointer(
        2,
        2,
        gl::FLOAT,
        gl::FALSE,
        std::mem::size_of::<Vertex>() as gl::types::GLsizei,
        offset_of!(Vertex, uv) as *const _,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawBound(this: *mut Mesh) {
    Metric_AddDraw(
        (*this).index.len() as i32 / 3,
        (*this).index.len() as i32 / 3,
        (*this).vertex.len() as i32,
    );
    gl::DrawElements(
        gl::TRIANGLES,
        (*this).index.len() as i32,
        gl::UNSIGNED_INT,
        std::ptr::null(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawUnbind(_this: *mut Mesh) {
    gl::DisableVertexAttribArray(0);
    gl::DisableVertexAttribArray(1);
    gl::DisableVertexAttribArray(2);
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Draw(this: *mut Mesh) {
    Mesh_DrawBind(this);
    Mesh_DrawBound(this);
    Mesh_DrawUnbind(this);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_DrawNormals(this: *mut Mesh, scale: f32) {
    gl::Begin(gl::LINES);
    for v in (*this).vertex.iter() {
        gl::Vertex3f((*v).p.x, (*v).p.y, (*v).p.z);
        gl::Vertex3f(
            (*v).p.x + scale * (*v).n.x,
            (*v).p.y + scale * (*v).n.y,
            (*v).p.z + scale * (*v).n.z,
        );
    }
    gl::End();
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetBound(this: *mut Mesh, out: *mut Box3) {
    Mesh_UpdateInfo(this);
    *out = (*this).info.bound;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetCenter(this: *mut Mesh, out: *mut Vec3) {
    Mesh_UpdateInfo(this);
    *out = (*this).info.bound.center();
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexCount(this: *mut Mesh) -> i32 {
    (*this).index.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetIndexData(this: *mut Mesh) -> *mut i32 {
    (*this).index.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetRadius(this: *mut Mesh) -> f32 {
    Mesh_UpdateInfo(this);
    (*this).info.radius
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVersion(this: *mut Mesh) -> u64 {
    (*this).version
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_IncVersion(this: *mut Mesh) {
    (*this).version += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Validate(this: *mut Mesh) -> Error {
    let indexLen: i32 = Mesh_GetIndexCount(this);
    let indexData: *mut i32 = Mesh_GetIndexData(this);
    let vertexData: *mut Vertex = Mesh_GetVertexData(this);
    if indexLen % 3 != 0 {
        return (0x100000 | 0x80) as Error;
    }
    let mut i: i32 = 0;
    while i < indexLen {
        let i0: i32 = *indexData.offset((i + 0) as isize);
        let i1: i32 = *indexData.offset((i + 1) as isize);
        let i2: i32 = *indexData.offset((i + 2) as isize);
        let mut triangle: Triangle = Triangle {
            vertices: [Vec3::ZERO; 3],
        };
        triangle.vertices[0] = (*vertexData.offset(i0 as isize)).p;
        triangle.vertices[1] = (*vertexData.offset(i1 as isize)).p;
        triangle.vertices[2] = (*vertexData.offset(i2 as isize)).p;
        let e: Error = Triangle_Validate(&mut triangle);
        if e != 0 {
            return 0x400000 | e;
        }
        i += 3;
    }
    for v in (*this).vertex.iter() {
        let mut e_0: Error = 0;
        e_0 = Vec3_Validate((*v).p);
        if e_0 != 0 {
            return 0x400000 | e_0;
        }
        e_0 = Vec3_Validate((*v).n);
        if e_0 != 0 {
            return 0x800000 | e_0;
        }
        e_0 = Vec2_Validate((*v).uv);
        if e_0 != 0 {
            return 0x1000000 | e_0;
        }
    }
    0 as Error
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertex(this: *mut Mesh, index: i32) -> *mut Vertex {
    &mut (*this).vertex[index as usize]
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexCount(this: *mut Mesh) -> i32 {
    (*this).vertex.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_GetVertexData(this: *mut Mesh) -> *mut Vertex {
    (*this).vertex.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveIndexData(this: *mut Mesh, capacity: i32) {
    (*this).index.reserve(capacity as usize);
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ReserveVertexData(this: *mut Mesh, capacity: i32) {
    (*this).vertex.reserve(capacity as usize)
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Center(this: *mut Mesh) -> *mut Mesh {
    let mut c = Vec3::ZERO;
    Mesh_GetCenter(this, &mut c);
    Mesh_Translate(this, -c.x, -c.y, -c.z);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Invert(this: *mut Mesh) -> *mut Mesh {
    for i in (0..(*this).index.len()).step_by(3) {
        std::mem::swap(&mut (*this).index[i + 1], &mut (*this).index[i + 2]);
    }
    (*this).version += 1;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateX(this: *mut Mesh, rads: f32) -> *mut Mesh {
    let matrix: *mut Matrix = Matrix_RotationX(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateY(this: *mut Mesh, rads: f32) -> *mut Mesh {
    let matrix: *mut Matrix = Matrix_RotationY(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateZ(this: *mut Mesh, rads: f32) -> *mut Mesh {
    let matrix: *mut Matrix = Matrix_RotationZ(rads);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_RotateYPR(
    this: *mut Mesh,
    yaw: f32,
    pitch: f32,
    roll: f32,
) -> *mut Mesh {
    let matrix: *mut Matrix = Matrix_YawPitchRoll(yaw, pitch, roll);
    Mesh_Transform(this, matrix);
    Matrix_Free(matrix);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Scale(this: *mut Mesh, x: f32, y: f32, z: f32) -> *mut Mesh {
    for v in (*this).vertex.iter_mut() {
        (*v).p.x *= x;
        (*v).p.y *= y;
        (*v).p.z *= z;
    }
    (*this).version += 1;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ScaleUniform(this: *mut Mesh, s: f32) -> *mut Mesh {
    Mesh_Scale(this, s, s, s);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Translate(this: *mut Mesh, x: f32, y: f32, z: f32) -> *mut Mesh {
    for v in (*this).vertex.iter_mut() {
        (*v).p.x += x;
        (*v).p.y += y;
        (*v).p.z += z;
    }
    (*this).version += 1;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Transform(this: *mut Mesh, matrix: *mut Matrix) -> *mut Mesh {
    for v in (*this).vertex.iter_mut() {
        Matrix_MulPoint(matrix, &mut (*v).p, (*v).p.x, (*v).p.y, (*v).p.z);
    }
    (*this).version += 1;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_ComputeNormals(this: *mut Mesh) {
    for v in (*this).vertex.iter_mut() {
        (*v).n = Vec3::ZERO;
    }
    for i in (0..(*this).index.len()).step_by(3) {
        let v1 = &mut (*this).vertex[(*this).index[i + 0] as usize];
        let v2 = &mut (*this).vertex[(*this).index[i + 1] as usize];
        let v3 = &mut (*this).vertex[(*this).index[i + 2] as usize];
        let e1: Vec3 = (*v2).p - (*v1).p;
        let e2: Vec3 = (*v3).p - (*v2).p;
        let en: Vec3 = Vec3::cross(e1, e2);
        (*v1).n += en;
        (*v2).n += en;
        (*v3).n += en;
    }
    for v in (*this).vertex.iter_mut() {
        (*v).n = (*v).n.normalize();
    }
    (*this).version += 1;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_SplitNormals(this: *mut Mesh, minDot: f32) {
    for v in (*this).vertex.iter_mut() {
        (*v).n = Vec3::ZERO;
    }
    for i in (0..(*this).index.len()).step_by(3) {
        let index: [&mut i32; 3] = [
            &mut (*this).index[i + 0],
            &mut (*this).index[i + 1],
            &mut (*this).index[i + 2],
        ];
        let v: [&mut Vertex; 3] = [
            &mut (*this).vertex[*index[0] as usize],
            &mut (*this).vertex[*index[1] as usize],
            &mut (*this).vertex[*index[2] as usize],
        ];
        let face: Vec3 = Vec3::cross((*v[1]).p - (*v[0]).p, (*v[2]).p - (*v[0]).p);
        for j in 0..3 {
            let cn: &mut Vec3 = &mut (*this).vertex[*index[j as usize] as usize].n;
            if (*cn).length_squared() > 0.0f32 {
                let cDot: f32 = Vec3::dot(face.normalize(), (*cn).normalize());
                if cDot < minDot {
                    let mut nv = (*this).vertex[*index[j as usize] as usize];
                    nv.n = face;
                    (*this).vertex.push(nv);
                    *index[j as usize] = (*this).vertex.len() as i32 - 1;
                } else {
                    (*cn) += face;
                }
            } else {
                (*cn) += face;
            }
        }
    }

    for v in (*this).vertex.iter_mut() {
        (*v).n = (*v).n.normalize();
    }
}
