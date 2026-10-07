//! Uniform Buffer Object (UBO) management for efficient uniform data sharing.
//!
//! UBOs allow sharing uniform data across multiple shaders with a single buffer update,
//! reducing per-draw glUniform calls significantly.

use crate::render::{gl, glcheck};

/// Binding point of the legacy light UBO (group 0 of the new binding
/// model owns 0..3; the per-pass `ViewBlock` is binding 0). It goes away in S5.
pub const LIGHT_UBO_BINDING: u32 = 2;

/// Light uniform buffer data with std140 layout.
///
/// Packs light properties for deferred shading.
/// 32 bytes total (2x vec4).
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct LightUboData {
    /// Light position in world space (xyz) + radius (w)
    pub position_radius: [f32; 4], // 16 bytes
    /// Light color (rgb) + intensity (w)
    pub color_intensity: [f32; 4], // 16 bytes
}

impl LightUboData {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn new() -> Self {
        Self {
            position_radius: [0.0, 0.0, 0.0, 100.0],
            color_intensity: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position_radius[0] = x;
        self.position_radius[1] = y;
        self.position_radius[2] = z;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.position_radius[3] = radius;
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.color_intensity[0] = r;
        self.color_intensity[1] = g;
        self.color_intensity[2] = b;
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.color_intensity[3] = intensity;
    }

    /// Convert to bytes for GPU upload
    #[allow(unsafe_code)]
    pub fn as_bytes(&self) -> &[u8; Self::SIZE] {
        // SAFETY: LightUboData is repr(C) with known size, all fields are POD
        unsafe { &*(self as *const Self as *const [u8; Self::SIZE]) }
    }
}

/// Manages a single UBO on the GPU
pub struct UniformBuffer {
    handle: gl::types::GLuint,
    size: usize,
    binding_point: u32,
}

impl UniformBuffer {
    /// Create a new UBO with the given size and binding point
    pub fn new(size: usize, binding_point: u32) -> Self {
        let mut handle = 0;
        glcheck!(gl::GenBuffers(1, &mut handle));

        glcheck!(gl::BindBuffer(gl::UNIFORM_BUFFER, handle));
        glcheck!(gl::BufferData(
            gl::UNIFORM_BUFFER,
            size as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        ));
        glcheck!(gl::BindBufferBase(
            gl::UNIFORM_BUFFER,
            binding_point,
            handle
        ));
        glcheck!(gl::BindBuffer(gl::UNIFORM_BUFFER, 0));

        Self {
            handle,
            size,
            binding_point,
        }
    }

    /// Update the buffer data
    pub fn update(&self, data: &[u8]) {
        debug_assert!(data.len() <= self.size, "UBO data exceeds buffer size");

        glcheck!(gl::BindBuffer(gl::UNIFORM_BUFFER, self.handle));
        glcheck!(gl::BufferSubData(
            gl::UNIFORM_BUFFER,
            0,
            data.len() as isize,
            data.as_ptr() as *const _,
        ));
        glcheck!(gl::BindBuffer(gl::UNIFORM_BUFFER, 0));
    }

    /// Bind this UBO to its binding point
    pub fn bind(&self) {
        glcheck!(gl::BindBufferBase(
            gl::UNIFORM_BUFFER,
            self.binding_point,
            self.handle
        ));
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn binding_point(&self) -> u32 {
        self.binding_point
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        if self.handle != 0 {
            glcheck!(gl::DeleteBuffers(1, &self.handle));
        }
    }
}

/// Global light UBO manager
pub struct LightUbo {
    buffer: UniformBuffer,
    data: LightUboData,
}

impl LightUbo {
    pub fn new() -> Self {
        Self {
            buffer: UniformBuffer::new(LightUboData::SIZE, LIGHT_UBO_BINDING),
            data: LightUboData::new(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.data.set_position(x, y, z);
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.data.set_radius(radius);
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.data.set_color(r, g, b);
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.data.set_intensity(intensity);
    }

    /// Upload data to GPU
    pub fn upload(&mut self) {
        self.buffer.update(self.data.as_bytes());
    }

    pub fn data(&self) -> &LightUboData {
        &self.data
    }

    pub fn buffer(&self) -> &UniformBuffer {
        &self.buffer
    }
}

impl Default for LightUbo {
    fn default() -> Self {
        Self::new()
    }
}
