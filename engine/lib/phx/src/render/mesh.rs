use std::cell::{Ref, RefMut};
use std::io::BufReader;
use std::time::SystemTime;

use glam::{Vec2, Vec3, Vec4};
use memoffset::offset_of;
use tobj::LoadError;

use super::{gl, DataFormat, Draw, PixelFormat, RenderTarget, Tex2D, Tex3D, TexFormat};
use crate::error::Error;
use crate::math::{validate_vec2, validate_vec3, Box3, Matrix, Triangle};
use crate::render::{glcheck, RenderState, Shader};
use crate::rf::Rf;
use crate::system::*;

#[derive(Clone)]
pub struct Mesh {
    shared: Rf<MeshShared>,
}

struct MeshShared {
    vbo: gl::types::GLuint,
    ibo: gl::types::GLuint,
    vao: gl::types::GLuint,
    uuid: u64,
    version: u64,
    version_buffers: u64,
    version_info: u64,
    info: Computed,
    index: Vec<i32>,
    vertex: Vec<Vertex>,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct MeshCacheKey(u128);

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
}

pub struct Computed {
    pub bound: Box3,
    pub radius: f32,
}

impl MeshShared {
    fn update_info(&mut self) {
        if self.version_info == self.version {
            return;
        }

        self.info.bound.lower = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        self.info.bound.upper = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in self.vertex.iter() {
            self.info.bound.add(v.p);
        }

        let center: Vec3 = self.info.bound.center();
        let mut r2: f64 = 0.0f64;
        for v in self.vertex.iter() {
            let dx: f64 = (v.p.x - center.x) as f64;
            let dy: f64 = (v.p.y - center.y) as f64;
            let dz: f64 = (v.p.z - center.z) as f64;
            r2 = f64::max(r2, dx * dx + dy * dy + dz * dz);
        }
        self.info.radius = f64::sqrt(r2) as f32;
        self.version_info = self.version;
    }

    pub fn get_cache_key(&self) -> MeshCacheKey {
        MeshCacheKey(((self.uuid as u128) << 64) | self.version as u128)
    }
}

impl Drop for MeshShared {
    fn drop(&mut self) {
        if self.vbo != 0 {
            glcheck!(gl::DeleteVertexArrays(1, &self.vao));
            glcheck!(gl::DeleteBuffers(1, &self.vbo));
            glcheck!(gl::DeleteBuffers(1, &self.ibo));
        }
    }
}

impl Mesh {
    pub fn get_cache_key(&self) -> MeshCacheKey {
        self.shared.as_ref().get_cache_key()
    }

    pub fn get_vertex_data(&self) -> Ref<[Vertex]> {
        Ref::map(self.shared.as_ref(), |shared| shared.vertex.as_slice())
    }

    pub fn get_vertex_data_mut(&mut self) -> RefMut<[Vertex]> {
        RefMut::map(self.shared.as_mut(), |shared| shared.vertex.as_mut_slice())
    }

    pub fn get_index_data(&self) -> Ref<[i32]> {
        Ref::map(self.shared.as_ref(), |shared| shared.index.as_slice())
    }

    pub fn get_index_data_mut(&mut self) -> RefMut<[i32]> {
        RefMut::map(self.shared.as_mut(), |shared| shared.index.as_mut_slice())
    }

    fn add_plane(&mut self, origin: Vec3, du: Vec3, dv: Vec3, res_u: i32, res_v: i32) {
        let n: Vec3 = Vec3::cross(du, dv).normalize();
        for iu in 0..res_u {
            let u: f32 = iu as f32 / (res_u - 1) as f32;
            for iv in 0..res_v {
                let v: f32 = iv as f32 / (res_v - 1) as f32;
                let p: Vec3 = origin + du * u + dv * v;
                if iu != 0 && iv != 0 {
                    let vc = self.get_vertex_count();
                    self.add_quad(vc, vc - res_v, vc - res_v - 1, vc - 1);
                }
                self.add_vertex(p.x, p.y, p.z, n.x, n.y, n.z, u, v);
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Mesh {
    #[bind(name = "Create")]
    pub fn new() -> Mesh {
        // Take the first 64 bits of the nanoseconds since unix epoch as a UUID. 2^63 nanoseconds
        // is 300 years, so we wont ever have any wraparounds of UUIDs.
        let uuid = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time cannot be before Unix epoch")
            .as_nanos() as u64;
        Mesh {
            shared: Rf::new(MeshShared {
                vbo: 0,
                ibo: 0,
                vao: 0,
                uuid,
                version: 1,
                version_buffers: 0,
                version_info: 0,
                info: Computed {
                    bound: Box3::default(),
                    radius: 0.0,
                },
                vertex: Vec::new(),
                index: Vec::new(),
            }),
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Mesh {
        self.clone()
    }

    pub fn load(name: &str) -> Mesh {
        Self::from_bytes(&mut Resource::load_bytes(ResourceType::Mesh, name))
    }

    pub fn to_bytes(&self) -> Bytes {
        let this = self.shared.as_ref();

        let vertex_count = this.vertex.len();
        let index_count = this.index.len();

        let size = 2 * std::mem::size_of::<i32>()
            + (vertex_count * std::mem::size_of::<Vertex>())
            + (index_count * std::mem::size_of::<i32>());

        let mut bytes = Bytes::new_with_capacity(size);
        bytes.write_i32(vertex_count as i32);
        bytes.write_i32(index_count as i32);
        bytes.write(this.vertex.as_slice());
        bytes.write(this.index.as_slice());
        bytes
    }

    pub fn from_bytes(buf: &mut Bytes) -> Mesh {
        let mut mesh = Mesh::new();

        let vertex_count = buf.read_i32();
        let index_count = buf.read_i32();

        mesh.reserve_vertex_data(vertex_count);
        mesh.reserve_index_data(index_count);

        {
            let this = &mut *mesh.shared.as_mut();

            this.vertex.resize(vertex_count as usize, Vertex::default());
            this.index.resize(index_count as usize, 0);

            buf.read(this.vertex.as_mut_slice());
            buf.read(this.index.as_mut_slice());
        }

        mesh
    }

    pub fn from_obj(bytes: &str) -> Mesh {
        let (models, _) = tobj::load_obj_buf(
            &mut BufReader::new(bytes.as_bytes()),
            &tobj::GPU_LOAD_OPTIONS,
            |_| Err(LoadError::OpenFileFailed),
        )
        .expect("Failed to OBJ load file");

        let mut mesh = Mesh::new();

        // All models in the OBJ file are flattened into a single Mesh.
        let mut start_index = 0;
        for m in &models {
            let model_mesh = &m.mesh;

            // Load vertex data.
            let num_vertices = model_mesh.positions.len() / 3;
            for v in 0..num_vertices {
                let mut vertex: Vertex = Vertex {
                    p: Vec3::new(
                        model_mesh.positions[3 * v],
                        model_mesh.positions[3 * v + 1],
                        model_mesh.positions[3 * v + 2],
                    ),
                    n: Vec3::ZERO,
                    uv: Vec2::ZERO,
                };

                if !model_mesh.normals.is_empty() {
                    vertex.n.x = model_mesh.normals[3 * v];
                    vertex.n.y = model_mesh.normals[3 * v + 1];
                    vertex.n.z = model_mesh.normals[3 * v + 2];
                }

                if !model_mesh.texcoords.is_empty() {
                    vertex.uv.x = model_mesh.texcoords[2 * v];
                    vertex.uv.y = model_mesh.texcoords[2 * v + 1];
                }

                mesh.add_vertex_raw(&vertex);
            }

            // Load index data.
            for i in &model_mesh.indices {
                mesh.add_index(start_index + *i as i32);
            }
            start_index += model_mesh.indices.len() as i32;
        }

        mesh
    }

    #[bind(name = "Box")]
    pub fn new_box(res: i32) -> Mesh {
        let origin: [Vec3; 6] = [
            Vec3::new(-1.0f32, -1.0f32, 1.0f32),
            Vec3::new(-1.0f32, -1.0f32, -1.0f32),
            Vec3::new(1.0f32, -1.0f32, -1.0f32),
            Vec3::new(-1.0f32, -1.0f32, -1.0f32),
            Vec3::new(-1.0f32, 1.0f32, -1.0f32),
            Vec3::new(-1.0f32, -1.0f32, -1.0f32),
        ];
        let du: [Vec3; 6] = [
            Vec3::new(2.0f32, 0.0f32, 0.0f32),
            Vec3::new(0.0f32, 2.0f32, 0.0f32),
            Vec3::new(0.0f32, 2.0f32, 0.0f32),
            Vec3::new(0.0f32, 0.0f32, 2.0f32),
            Vec3::new(0.0f32, 0.0f32, 2.0f32),
            Vec3::new(2.0f32, 0.0f32, 0.0f32),
        ];
        let dv: [Vec3; 6] = [
            Vec3::new(0.0f32, 2.0f32, 0.0f32),
            Vec3::new(2.0f32, 0.0f32, 0.0f32),
            Vec3::new(0.0f32, 0.0f32, 2.0f32),
            Vec3::new(0.0f32, 2.0f32, 0.0f32),
            Vec3::new(2.0f32, 0.0f32, 0.0f32),
            Vec3::new(0.0f32, 0.0f32, 2.0f32),
        ];

        let mut this = Self::new();
        for i in 0..6 {
            this.add_plane(origin[i as usize], du[i as usize], dv[i as usize], res, res);
        }
        this
    }

    #[bind(name = "BoxSphere")]
    pub fn new_box_sphere(res: i32) -> Mesh {
        let mut this: Mesh = Mesh::new_box(res);

        // Normalize all points.
        {
            let mut vertex_data = this.get_vertex_data_mut();
            for v in vertex_data.iter_mut() {
                v.p = v.p.normalize();
            }
        }

        this
    }

    #[bind(name = "Plane")]
    pub fn new_plane(origin: Vec3, du: Vec3, dv: Vec3, res_u: i32, res_v: i32) -> Mesh {
        let mut this = Mesh::new();
        this.add_plane(origin, du, dv, res_u, res_v);
        this
    }

    pub fn add_index(&mut self, new_index: i32) {
        self.shared.as_mut().index.push(new_index);
        self.shared.as_mut().version += 1;
    }

    pub fn add_mesh(&mut self, other: &Mesh) {
        let index_offset: i32 = self.shared.as_ref().vertex.len() as i32;
        for i in 0..other.shared.as_ref().vertex.len() {
            self.add_vertex_raw(&other.shared.as_ref().vertex[i]);
        }
        for i in 0..other.shared.as_ref().index.len() {
            self.add_index(other.shared.as_ref().index[i] + index_offset);
        }
    }

    pub fn add_quad(&mut self, i1: i32, i2: i32, i3: i32, i4: i32) {
        self.add_tri(i1, i2, i3);
        self.add_tri(i1, i3, i4);
    }

    pub fn add_tri(&mut self, i1: i32, i2: i32, i3: i32) {
        self.add_index(i1);
        self.add_index(i2);
        self.add_index(i3);
    }

    pub fn add_vertex(
        &mut self,
        px: f32,
        py: f32,
        pz: f32,
        nx: f32,
        ny: f32,
        nz: f32,
        u: f32,
        v: f32,
    ) {
        self.shared.as_mut().vertex.push(Vertex {
            p: Vec3::new(px, py, pz),
            n: Vec3::new(nx, ny, nz),
            uv: Vec2::new(u, v),
        });
        self.shared.as_mut().version += 1;
    }

    pub fn add_vertex_raw(&mut self, vertex: &Vertex) {
        self.shared.as_mut().vertex.push(*vertex);
        self.shared.as_mut().version += 1;
    }

    pub fn draw_bind(&mut self) {
        let this = &mut *self.shared.as_mut();

        /* Release cached GL buffers if the mesh has changed since we built them. */
        if this.vbo != 0 && this.version != this.version_buffers {
            glcheck!(gl::DeleteVertexArrays(1, &this.vao));
            glcheck!(gl::DeleteBuffers(1, &this.vbo));
            glcheck!(gl::DeleteBuffers(1, &this.ibo));
            this.vao = 0;
            this.vbo = 0;
            this.ibo = 0;
        }

        /* Generate cached GL buffers for fast drawing. */
        if this.vbo == 0 {
            glcheck!(gl::GenBuffers(1, &mut this.vbo));
            glcheck!(gl::GenBuffers(1, &mut this.ibo));
            glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, this.vbo));
            glcheck!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, this.ibo));
            glcheck!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (this.vertex.len() as i32 as usize).wrapping_mul(std::mem::size_of::<Vertex>())
                    as gl::types::GLsizeiptr,
                this.vertex.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));

            /* TODO : 16-bit index optimization */
            /* TODO : Check if 8-bit indices are supported by hardware. IIRC they
             *        weren't last time I checked. */

            glcheck!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (this.index.len() as i32 as usize).wrapping_mul(std::mem::size_of::<i32>())
                    as gl::types::GLsizeiptr,
                this.index.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));

            glcheck!(gl::GenVertexArrays(1, &mut this.vao));
            glcheck!(gl::BindVertexArray(this.vao));

            glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, this.vbo));
            glcheck!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, this.ibo));
            glcheck!(gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as gl::types::GLsizei,
                offset_of!(Vertex, p) as *const _,
            ));
            glcheck!(gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as gl::types::GLsizei,
                offset_of!(Vertex, n) as *const _,
            ));
            glcheck!(gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as gl::types::GLsizei,
                offset_of!(Vertex, uv) as *const _,
            ));

            this.version_buffers = this.version;
        }

        glcheck!(gl::BindVertexArray(this.vao));
        glcheck!(gl::EnableVertexAttribArray(0));
        glcheck!(gl::EnableVertexAttribArray(1));
        glcheck!(gl::EnableVertexAttribArray(2));
    }

    pub fn draw_bound(&self) {
        let this = self.shared.as_ref();

        Metric::add_draw(
            this.index.len() as u64 / 3,
            this.index.len() as u64 / 3,
            this.vertex.len() as u64,
        );

        glcheck!(gl::DrawElements(
            gl::TRIANGLES,
            this.index.len() as i32,
            gl::UNSIGNED_INT,
            std::ptr::null(),
        ));
    }

    pub fn draw_unbind(&self) {
        glcheck!(gl::DisableVertexAttribArray(0));
        glcheck!(gl::DisableVertexAttribArray(1));
        glcheck!(gl::DisableVertexAttribArray(2));
        glcheck!(gl::BindVertexArray(0));
    }

    pub fn draw(&mut self) {
        self.draw_bind();
        self.draw_bound();
        self.draw_unbind();
    }

    pub fn draw_normals(&self, scale: f32) {
        for v in &self.shared.as_ref().vertex {
            Draw::line3(&v.p, &(v.p + scale * v.n));
        }
    }

    // TODO: convert out to return
    pub fn get_bound(&mut self, out: &mut Box3) {
        self.shared.as_mut().update_info();
        *out = self.shared.as_ref().info.bound;
    }

    pub fn get_center(&mut self, out: &mut Vec3) {
        self.shared.as_mut().update_info();
        *out = self.shared.as_ref().info.bound.center();
    }

    pub fn get_index_count(&self) -> i32 {
        self.shared.as_ref().index.len() as i32
    }

    #[bind(name = "LockIndexData")]
    pub fn lock_index_data_mut(&mut self, f: impl FnOnce(&mut [i32])) {
        f(self.shared.as_mut().index.as_mut_slice());
    }

    pub fn get_radius(&mut self) -> f32 {
        self.shared.as_mut().update_info();
        self.shared.as_ref().info.radius
    }

    pub fn get_version(&self) -> u64 {
        self.shared.as_ref().version
    }

    pub fn inc_version(&mut self) {
        self.shared.as_mut().version += 1;
    }

    // TODO: Make `Error` an enum.
    pub fn validate(&self) -> u32 {
        let this = self.shared.as_ref();

        let index_len = this.index.len();
        if index_len % 3 != 0 {
            // return Error_Index | Error_BadCount;
            return (0x100000 | 0x80) as Error;
        }

        for i in (0..index_len).step_by(3) {
            let i0 = this.index[i] as usize;
            let i1 = this.index[i + 1] as usize;
            let i2 = this.index[i + 2] as usize;
            let triangle = Triangle {
                vertices: [this.vertex[i0].p, this.vertex[i1].p, this.vertex[i2].p],
            };
            let e = triangle.validate();
            if e != 0 {
                return 0x400000 | e;
            }
        }

        for v in this.vertex.iter() {
            let e = validate_vec3(v.p);
            if e != 0 {
                return 0x400000 | e;
            }
            let e = validate_vec3(v.n);
            if e != 0 {
                return 0x800000 | e;
            }
            let e = validate_vec2(v.uv);
            if e != 0 {
                return 0x1000000 | e;
            }
        }

        0 as Error
    }

    pub fn get_vertex(&mut self, index: i32) -> &mut Vertex {
        let ptr = &mut self.shared.as_mut().vertex[index as usize] as *mut _;
        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            &mut *ptr
        }
    }

    pub fn get_vertex_count(&self) -> i32 {
        self.shared.as_ref().vertex.len() as i32
    }

    #[bind(name = "LockVertexData")]
    pub fn lock_vertex_data_mut(&mut self, f: impl FnOnce(&mut [Vertex])) {
        f(self.shared.as_mut().vertex.as_mut_slice());
    }

    pub fn reserve_index_data(&mut self, capacity: i32) {
        self.shared.as_mut().index.reserve(capacity as usize);
    }

    pub fn reserve_vertex_data(&mut self, capacity: i32) {
        self.shared.as_mut().vertex.reserve(capacity as usize)
    }

    pub fn center(&mut self) -> &mut Mesh {
        let mut c = Vec3::ZERO;
        self.get_center(&mut c);
        self.translate(-c.x, -c.y, -c.z);
        self
    }

    pub fn invert(&mut self) -> &mut Mesh {
        {
            let this = &mut *self.shared.as_mut();
            for i in (0..this.index.len()).step_by(3) {
                this.index.swap(i + 1, i + 2);
            }
            this.version += 1;
        }

        self
    }

    pub fn rotate_x(&mut self, rads: f32) -> &mut Mesh {
        let matrix = Matrix::rotation_x(rads);
        self.transform(&matrix);
        self
    }

    pub fn rotate_y(&mut self, rads: f32) -> &mut Mesh {
        let matrix = Matrix::rotation_y(rads);
        self.transform(&matrix);
        self
    }

    pub fn rotate_z(&mut self, rads: f32) -> &mut Mesh {
        let matrix = Matrix::rotation_z(rads);
        self.transform(&matrix);
        self
    }

    #[bind(name = "RotateYPR")]
    pub fn rotate_ypr(&mut self, yaw: f32, pitch: f32, roll: f32) -> &mut Mesh {
        let matrix = Matrix::yaw_pitch_roll(yaw, pitch, roll);
        self.transform(&matrix);
        self
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Mesh {
        {
            let mut this = self.shared.as_mut();

            for v in &mut this.vertex {
                v.p.x *= x;
                v.p.y *= y;
                v.p.z *= z;
            }
            this.version += 1;
        }

        self
    }

    pub fn scale_uniform(&mut self, s: f32) -> &mut Mesh {
        self.scale(s, s, s)
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Mesh {
        {
            let mut this = self.shared.as_mut();

            for v in &mut this.vertex {
                v.p.x += x;
                v.p.y += y;
                v.p.z += z;
            }
            this.version += 1;
        }

        self
    }

    pub fn transform(&mut self, matrix: &Matrix) {
        let mut this = self.shared.as_mut();

        for v in &mut this.vertex {
            v.p = matrix.transform_point3(v.p);
        }
        this.version += 1;
    }

    pub fn compute_normals(&mut self) {
        let this = &mut *self.shared.as_mut();

        for v in &mut this.vertex {
            v.n = Vec3::ZERO;
        }

        for i in (0..this.index.len()).step_by(3) {
            let v1 = this.vertex[this.index[i] as usize].p;
            let v2 = this.vertex[this.index[i + 1] as usize].p;
            let v3 = this.vertex[this.index[i + 2] as usize].p;
            let e1: Vec3 = v2 - v1;
            let e2: Vec3 = v3 - v2;
            let en: Vec3 = Vec3::cross(e1, e2);

            this.vertex[this.index[i] as usize].n += en;
            this.vertex[this.index[i + 1] as usize].n += en;
            this.vertex[this.index[i + 2] as usize].n += en;
        }

        for v in &mut this.vertex {
            v.n = v.n.normalize();
        }

        this.version += 1;
    }

    pub fn split_normals(&mut self, min_dot: f32) {
        let this = &mut *self.shared.as_mut();

        for v in &mut this.vertex {
            v.n = Vec3::ZERO;
        }

        for i in (0..this.index.len()).step_by(3) {
            let index_range: &mut [i32] = &mut this.index[i..=i + 2];
            let face: Vec3 = Vec3::cross(
                this.vertex[index_range[1] as usize].p - this.vertex[index_range[0] as usize].p,
                this.vertex[index_range[2] as usize].p - this.vertex[index_range[0] as usize].p,
            );

            for index in index_range {
                let cn = &mut this.vertex[*index as usize].n;
                if cn.length_squared() > 0.0 {
                    let c_dot = Vec3::dot(face.normalize(), cn.normalize());
                    if c_dot < min_dot {
                        let mut nv = this.vertex[*index as usize];
                        nv.n = face;
                        this.vertex.push(nv);
                        *index = this.vertex.len() as i32 - 1;
                    } else {
                        *cn += face;
                    }
                } else {
                    *cn += face;
                }
            }
        }

        for v in &mut this.vertex {
            v.n = v.n.normalize();
        }
    }

    #[bind(name = "ComputeAO")]
    pub fn compute_ao(&mut self, radius: f32) {
        let this = &mut *self.shared.as_mut();

        let s_dim = f64::ceil(f64::sqrt((this.index.len() / 3) as f64)) as usize;
        let v_dim = f64::ceil(f64::sqrt(this.vertex.len() as f64)) as usize;
        let surfels = s_dim * s_dim;
        let vertices = v_dim * v_dim;
        let buf_size = usize::max(surfels, vertices);

        let mut point_buffer = vec![Vec4::ZERO; buf_size];
        let mut normal_buffer = vec![Vec4::ZERO; buf_size];

        for i in (0..this.index.len()).step_by(3) {
            let v1 = &this.vertex[this.index[i] as usize];
            let v2 = &this.vertex[this.index[i + 1] as usize];
            let v3 = &this.vertex[this.index[i + 2] as usize];
            let mut normal: Vec3 = Vec3::cross(v3.p - v1.p, v2.p - v1.p);
            let length: f32 = normal.length();
            let area: f32 = 0.5f32 * length / std::f32::consts::PI;
            if f64::abs(length as f64) > 1e-6f64 {
                normal /= length;
            } else {
                normal = Vec3::X;
            }
            let center: Vec3 = (v1.p + v2.p + v3.p) / 3.0f32;
            point_buffer[i / 3] = Vec4::new(center.x, center.y, center.z, area);
            normal_buffer[i / 3] = Vec4::new(normal.x, normal.y, normal.z, 0.0f32);
        }

        let mut tex_spoints = Tex2D::new(s_dim as i32, s_dim as i32, TexFormat::RGBA32F);
        let mut tex_snormals = Tex2D::new(s_dim as i32, s_dim as i32, TexFormat::RGBA32F);
        tex_spoints.set_data(&point_buffer, PixelFormat::RGBA, DataFormat::Float);
        tex_snormals.set_data(&normal_buffer, PixelFormat::RGBA, DataFormat::Float);

        point_buffer.clear();
        point_buffer.resize(buf_size as usize, Vec4::ZERO);
        normal_buffer.clear();
        normal_buffer.resize(buf_size as usize, Vec4::ZERO);
        for i in 0..this.vertex.len() {
            let v = &this.vertex[i];
            point_buffer[i] = Vec4::new(v.p.x, v.p.y, v.p.z, 0.0);
            normal_buffer[i] = Vec4::new(v.n.x, v.n.y, v.n.z, 0.0);
        }

        let mut tex_vpoints = Tex2D::new(v_dim as i32, v_dim as i32, TexFormat::RGBA32F);
        let mut tex_vnormals = Tex2D::new(v_dim as i32, v_dim as i32, TexFormat::RGBA32F);
        tex_vpoints.set_data(&point_buffer, PixelFormat::RGBA, DataFormat::Float);
        tex_vnormals.set_data(&normal_buffer, PixelFormat::RGBA, DataFormat::Float);

        let tex_output = Tex2D::new(v_dim as i32, v_dim as i32, TexFormat::R32F);

        // TODO: Store shader properly
        #[allow(unsafe_code)] // TODO: remove
        let shader = unsafe {
            static mut SHADER: *mut Shader = std::ptr::null_mut();
            if SHADER.is_null() {
                SHADER = Box::into_raw(Box::new(Shader::load(
                    "vertex/identity",
                    "fragment/compute/occlusion",
                )));
            }
            &mut *SHADER
        };

        RenderState::push_all_defaults();
        RenderTarget::push_tex2d(&tex_output);

        shader.start();
        shader.set_int("sDim", s_dim as i32);
        shader.set_float("radius", radius);
        shader.set_tex2d("sPointBuffer", &tex_spoints);
        shader.set_tex2d("sNormalBuffer", &tex_snormals);
        shader.set_tex2d("vPointBuffer", &tex_vpoints);
        shader.set_tex2d("vNormalBuffer", &tex_vnormals);
        Draw::rect(-1.0, -1.0, 2.0, 2.0);
        shader.stop();

        RenderTarget::pop();
        RenderState::pop_all();

        let result: Vec<f32> = tex_output.get_data(PixelFormat::Red, DataFormat::Float);
        for (i, result_uv_value) in result.iter().enumerate().take(this.vertex.len()) {
            this.vertex[i].uv.x = *result_uv_value;
        }
    }

    pub fn compute_occlusion(&mut self, sdf: &mut Tex3D, radius: f32) {
        let this = &mut *self.shared.as_mut();

        let v_dim: i32 = f64::ceil(f64::sqrt(this.vertex.len() as f64)) as i32;
        let mut tex_points = Tex2D::new(v_dim, v_dim, TexFormat::RGBA32F);
        let tex_output = Tex2D::new(v_dim, v_dim, TexFormat::R32F);

        let mut point_buffer = vec![Vec3::ZERO; (v_dim * v_dim) as usize];
        for (i, point) in point_buffer.iter_mut().enumerate().take(this.vertex.len()) {
            *point = this.vertex[i].p;
        }

        tex_points.set_data(&point_buffer, PixelFormat::RGB, DataFormat::Float);

        // TODO: Store shader properly.
        #[allow(unsafe_code)] // TODO: remove
        let shader = unsafe {
            static mut SHADER: *mut Shader = std::ptr::null_mut();
            if SHADER.is_null() {
                SHADER = Box::into_raw(Box::new(Shader::load(
                    "vertex/identity",
                    "fragment/compute/occlusion_sdf",
                )));
            }
            &mut *SHADER
        };

        RenderState::push_all_defaults();
        RenderTarget::push_tex2d(&tex_output);

        shader.start();
        shader.set_float("radius", radius);
        shader.set_tex2d("points", &tex_points);
        shader.set_tex3d("sdf", sdf);
        Draw::rect(-1.0, -1.0, 2.0, 2.0);
        shader.stop();

        RenderTarget::pop();
        RenderState::pop_all();

        let result: Vec<f32> = tex_output.get_data(PixelFormat::Red, DataFormat::Float);
        for (i, result_uv_value) in result.iter().enumerate().take(this.vertex.len()) {
            this.vertex[i].uv.x = *result_uv_value;
        }
    }
}
