use std::collections::HashMap;
use std::mem::size_of;

use glam::{vec3, vec4, Vec3, Vec4};

use crate::render::glcheck;

use super::gl;

#[derive(PartialEq)]
pub enum PrimitiveType {
    /// A list of points.
    Points,

    /// A list of lines.
    Lines,

    /// A list of triangles.
    Triangles,

    /// A list of quads.
    Quads,

    /// A single convex polygon, using the first vertex as the point shared by all triangles.
    Polygon,
}

impl PrimitiveType {
    fn to_gl(&self) -> gl::types::GLenum {
        match self {
            PrimitiveType::Points => gl::POINTS,
            PrimitiveType::Lines => gl::LINES,
            PrimitiveType::Triangles => gl::TRIANGLES,
            PrimitiveType::Quads => gl::TRIANGLES,
            PrimitiveType::Polygon => gl::TRIANGLE_FAN,
        }
    }
}

// A type which constructs dynamic vertex / index buffers like the OpenGL immediate mode API.
pub struct PrimitiveBuilder {
    primitive: PrimitiveType,

    positions: Vec<Vec4>,
    normals: Vec<Vec3>,
    texcoords: Vec<Vec4>,
    colors: Vec<Vec4>,

    // The dimensions of each attribute of the vertex data and if it exists.
    positions_dim: u32,
    normals_dim: Option<u32>,
    texcoords_dim: Option<u32>,
    colors_dim: Option<u32>,

    // The current attributes that will be used to seed the next vertex.
    current_normal: Vec3,
    current_texcoord: Vec4,
    current_color: Vec4,

    // The OpenGL buffers storing each attribute.
    positions_vb: gl::types::GLuint,
    normals_vb: gl::types::GLuint,
    texcoords_vb: gl::types::GLuint,
    colors_vb: gl::types::GLuint,

    // Vertex array object cache.
    vao_cache: HashMap<u32, gl::types::GLuint>,
}

impl Drop for PrimitiveBuilder {
    fn drop(&mut self) {
        let vertex_buffers: [gl::types::GLuint; 4] = [
            self.positions_vb,
            self.normals_vb,
            self.texcoords_vb,
            self.colors_vb,
        ];
        glcheck!(gl::DeleteBuffers(4, vertex_buffers.as_ptr()))
    }
}

impl PrimitiveBuilder {
    pub fn new() -> PrimitiveBuilder {
        let mut vertex_buffers: [gl::types::GLuint; 4] = [0, 0, 0, 0];
        glcheck!(gl::GenBuffers(4, vertex_buffers.as_mut_ptr()));

        PrimitiveBuilder {
            primitive: PrimitiveType::Triangles,
            positions: vec![],
            normals: vec![],
            texcoords: vec![],
            colors: vec![],
            positions_dim: 4,
            normals_dim: None,
            texcoords_dim: None,
            colors_dim: None,
            current_normal: Vec3::ZERO,
            current_texcoord: vec4(0.0, 0.0, 0.0, 1.0),
            current_color: vec4(0.0, 0.0, 0.0, 1.0),
            positions_vb: vertex_buffers[0],
            normals_vb: vertex_buffers[1],
            texcoords_vb: vertex_buffers[2],
            colors_vb: vertex_buffers[3],
            vao_cache: HashMap::new(),
        }
    }

    pub fn begin(&mut self, primitive_type: PrimitiveType) {
        self.primitive = primitive_type;

        self.normals_dim = None;
        self.texcoords_dim = None;
        self.colors_dim = None;
        self.current_normal = vec3(0.0, 0.0, 0.0);
        self.current_texcoord = vec4(0.0, 0.0, 0.0, 1.0);
        self.current_color = vec4(1.0, 1.0, 1.0, 1.0);
    }

    pub fn end(&mut self) {
        self.flush_and_draw();
    }

    pub fn color3(&mut self, r: f32, g: f32, b: f32) {
        self.colors_dim = Some(3);
        self.current_color = vec4(r, g, b, 1.0);
    }

    pub fn color4(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.colors_dim = Some(4);
        self.current_color = vec4(r, g, b, a);
    }

    pub fn texcoord2(&mut self, s: f32, t: f32) {
        self.texcoords_dim = Some(2);
        self.current_texcoord = vec4(s, t, 0.0, 1.0);
    }

    pub fn texcoord3(&mut self, s: f32, t: f32, r: f32) {
        self.texcoords_dim = Some(3);
        self.current_texcoord = vec4(s, t, r, 1.0);
    }

    pub fn texcoord4(&mut self, s: f32, t: f32, r: f32, q: f32) {
        self.texcoords_dim = Some(4);
        self.current_texcoord = vec4(s, t, r, q);
    }

    pub fn normal3(&mut self, x: f32, y: f32, z: f32) {
        self.normals_dim = Some(3);
        self.current_normal = vec3(x, y, z);
    }

    pub fn vertex2(&mut self, x: f32, y: f32) {
        self.positions_dim = 2;
        self.finish_vertex(x, y, 0.0, 1.0);
    }

    pub fn vertex3(&mut self, x: f32, y: f32, z: f32) {
        self.positions_dim = 3;
        self.finish_vertex(x, y, z, 1.0);
    }

    pub fn vertex4(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.positions_dim = 4;
        self.finish_vertex(x, y, z, w);
    }

    fn finish_vertex(&mut self, x: f32, y: f32, z: f32, w: f32) {
        // IDEA: We have a "current batch" which gets filled from here. One array for each vertex attribute.
        // Then when the batch hits the limit of the VB (8192 vertices?) or end() is called, a draw call is made by uploading
        // the data to the dynamic vertex buffers (one buffer for positions, one for normals, etc). We only bind the relevant
        // buffers when drawing.
        self.positions.push(vec4(x, y, z, w));
        self.normals.push(self.current_normal);
        self.texcoords.push(self.current_texcoord);
        self.colors.push(self.current_color);

        // If this is a quad list, and we've just inserted the 4th vertex (in a group of 6), we can automatically generate the 2nd triangle of the quad.
        //
        // A---B
        // | \ |
        // D---C
        //
        // To build a triangle list after inserting D, we now need to re-insert C, then A
        if self.positions.len() % 6 == 4 && self.primitive == PrimitiveType::Quads {
            let last_idx = self.positions.len() - 1;

            // Insert A
            self.positions.push(self.positions[last_idx - 3]);
            self.normals.push(self.normals[last_idx - 3]);
            self.texcoords.push(self.texcoords[last_idx - 3]);
            self.colors.push(self.colors[last_idx - 3]);

            // Insert C
            self.positions.push(self.positions[last_idx - 1]);
            self.normals.push(self.normals[last_idx - 1]);
            self.texcoords.push(self.texcoords[last_idx - 1]);
            self.colors.push(self.colors[last_idx - 1]);
        }
    }

    fn flush_and_draw(&mut self) {
        // Update buffers.

        // Position is always included.
        let len_bytes = self.positions.len() * size_of::<Vec4>();
        glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.positions_vb));
        glcheck!(gl::BufferData(
            gl::ARRAY_BUFFER,
            len_bytes as isize,
            std::ptr::null(),
            gl::STREAM_DRAW
        ));
        let mapped = glcheck!(gl::MapBufferRange(
            gl::ARRAY_BUFFER,
            0,
            len_bytes as isize,
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ));

        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.positions.as_ptr(),
                mapped as *mut Vec4,
                self.positions.len(),
            );
        }
        glcheck!(gl::UnmapBuffer(gl::ARRAY_BUFFER));

        if self.normals_dim.is_some() {
            let len_bytes = self.normals.len() * size_of::<Vec3>();
            glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.normals_vb));
            glcheck!(gl::BufferData(
                gl::ARRAY_BUFFER,
                len_bytes as isize,
                std::ptr::null(),
                gl::STREAM_DRAW
            ));
            let mapped = glcheck!(gl::MapBufferRange(
                gl::ARRAY_BUFFER,
                0,
                len_bytes as isize,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ));

            #[allow(unsafe_code)] // TODO: remove
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.normals.as_ptr(),
                    mapped as *mut Vec3,
                    self.normals.len(),
                );
            }
            glcheck!(gl::UnmapBuffer(gl::ARRAY_BUFFER));
        }

        if self.texcoords_dim.is_some() {
            let len_bytes = self.texcoords.len() * size_of::<Vec4>();
            glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.texcoords_vb));
            glcheck!(gl::BufferData(
                gl::ARRAY_BUFFER,
                len_bytes as isize,
                std::ptr::null(),
                gl::STREAM_DRAW
            ));
            let mapped = glcheck!(gl::MapBufferRange(
                gl::ARRAY_BUFFER,
                0,
                len_bytes as isize,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ));

            #[allow(unsafe_code)] // TODO: remove
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.texcoords.as_ptr(),
                    mapped as *mut Vec4,
                    self.texcoords.len(),
                );
            }
            glcheck!(gl::UnmapBuffer(gl::ARRAY_BUFFER));
        }

        if self.colors_dim.is_some() {
            let len_bytes = self.colors.len() * size_of::<Vec4>();
            glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.colors_vb));
            glcheck!(gl::BufferData(
                gl::ARRAY_BUFFER,
                len_bytes as isize,
                std::ptr::null(),
                gl::STREAM_DRAW
            ));
            let mapped = glcheck!(gl::MapBufferRange(
                gl::ARRAY_BUFFER,
                0,
                len_bytes as isize,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ));

            #[allow(unsafe_code)] // TODO: remove
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.colors.as_ptr(),
                    mapped as *mut Vec4,
                    self.colors.len(),
                );
            }
            glcheck!(gl::UnmapBuffer(gl::ARRAY_BUFFER));
        }

        // Bind vertex array object for the requested dimensions.
        let vao_key = (self.normals_dim.is_some() as u32)
            + (self.texcoords_dim.is_some() as u32) * 2
            + (self.colors_dim.is_some() as u32) * 4;
        let vao = *self.vao_cache.entry(vao_key).or_insert_with(|| {
            let mut vao = 0;
            glcheck!(gl::GenVertexArrays(1, &mut vao));
            glcheck!(gl::BindVertexArray(vao));
            glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.positions_vb));
            glcheck!(gl::VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null()
            ));
            if self.normals_dim.is_some() {
                glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.normals_vb));
                glcheck!(gl::VertexAttribPointer(
                    1,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null()
                ));
            }
            if self.texcoords_dim.is_some() {
                glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.texcoords_vb));
                glcheck!(gl::VertexAttribPointer(
                    2,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null()
                ));
            }
            if self.colors_dim.is_some() {
                glcheck!(gl::BindBuffer(gl::ARRAY_BUFFER, self.colors_vb));
                glcheck!(gl::VertexAttribPointer(
                    3,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null()
                ));
            }
            vao
        });

        // Issue draw call.
        glcheck!(gl::BindVertexArray(vao));
        glcheck!(gl::EnableVertexAttribArray(0));
        if self.normals_dim.is_some() {
            glcheck!(gl::EnableVertexAttribArray(1));
        }
        if self.texcoords_dim.is_some() {
            glcheck!(gl::EnableVertexAttribArray(2));
        }
        if self.colors_dim.is_some() {
            glcheck!(gl::EnableVertexAttribArray(3));
        }
        glcheck!(gl::DrawArrays(
            self.primitive.to_gl(),
            0,
            self.positions.len() as gl::types::GLsizei
        ));
        glcheck!(gl::BindVertexArray(0));

        // Create data for next draw call.
        self.positions.clear();
        self.normals.clear();
        self.texcoords.clear();
        self.colors.clear();
    }
}
