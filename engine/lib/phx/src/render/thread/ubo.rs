//! Uniform Buffer Object (UBO) management for efficient uniform data sharing.
//!
//! UBOs allow sharing uniform data across multiple shaders with a single buffer update,
//! reducing per-draw glUniform calls significantly.

use glam::{Mat4, Vec3};

use crate::render::{gl, glcheck};

/// Binding points for standard UBOs
pub const CAMERA_UBO_BINDING: u32 = 0;
pub const MATERIAL_UBO_BINDING: u32 = 1;
pub const LIGHT_UBO_BINDING: u32 = 2;

pub type CameraUboArray = [u8; CameraUboData::SIZE];

/// Camera uniform buffer data with std140 layout.
///
/// std140 layout rules:
/// - mat4: 64 bytes, aligned to 16 bytes
/// - vec4: 16 bytes, aligned to 16 bytes
/// - vec3: 12 bytes BUT aligned to 16 bytes (so we use vec4)
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct CameraUboData {
    pub m_view: [[f32; 4]; 4],     // 64 bytes
    pub m_proj: [[f32; 4]; 4],     // 64 bytes
    pub m_view_inv: [[f32; 4]; 4], // 64 bytes
    pub m_proj_inv: [[f32; 4]; 4], // 64 bytes
    pub eye: [f32; 4],             // 16 bytes (xyz + padding)
    pub star_dir: [f32; 4],        // 16 bytes (xyz + padding)
}

impl CameraUboData {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn new() -> Self {
        Self {
            m_view: Mat4::IDENTITY.to_cols_array_2d(),
            m_proj: Mat4::IDENTITY.to_cols_array_2d(),
            m_view_inv: Mat4::IDENTITY.to_cols_array_2d(),
            m_proj_inv: Mat4::IDENTITY.to_cols_array_2d(),
            eye: [0.0, 0.0, 0.0, 1.0],
            star_dir: [0.0, 1.0, 0.0, 0.0],
        }
    }

    pub fn set_view(&mut self, view: &Mat4) {
        self.m_view = view.to_cols_array_2d();
    }

    pub fn set_view_inv(&mut self, view_inv: &Mat4) {
        self.m_view_inv = view_inv.to_cols_array_2d();
    }

    pub fn set_proj(&mut self, proj: &Mat4) {
        self.m_proj = proj.to_cols_array_2d();
        self.m_proj_inv = proj.inverse().to_cols_array_2d();
    }

    pub fn set_eye(&mut self, eye: Vec3) {
        self.eye = [eye.x, eye.y, eye.z, 1.0];
    }

    pub fn set_star_dir(&mut self, dir: Vec3) {
        self.star_dir = [dir.x, dir.y, dir.z, 0.0];
    }

    /// Convert to bytes for GPU upload
    #[allow(unsafe_code)]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: CameraUboData is repr(C) with known size, all fields are POD
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, Self::SIZE) }
    }
}

/// Material uniform buffer data with std140 layout.
///
/// Packs common per-draw material properties to reduce uniform calls.
/// 32 bytes total (2x vec4).
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct MaterialUboData {
    /// Base color (RGBA)
    pub color: [f32; 4], // 16 bytes
    /// Material parameters: x=metallic, y=roughness, z=emission, w=padding
    pub params: [f32; 4], // 16 bytes
}

impl MaterialUboData {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn new() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            params: [0.0, 0.5, 0.0, 0.0], // Default: non-metallic, medium roughness
        }
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = [r, g, b, a];
    }

    pub fn set_metallic(&mut self, metallic: f32) {
        self.params[0] = metallic;
    }

    pub fn set_roughness(&mut self, roughness: f32) {
        self.params[1] = roughness;
    }

    pub fn set_emission(&mut self, emission: f32) {
        self.params[2] = emission;
    }

    /// Convert to bytes for GPU upload
    #[allow(unsafe_code)]
    pub fn as_bytes(&self) -> &[u8; Self::SIZE] {
        // SAFETY: MaterialUboData is repr(C) with known size, all fields are POD
        unsafe { &*(self as *const Self as *const [u8; Self::SIZE]) }
    }
}

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

/// Global camera UBO manager
pub struct CameraUbo {
    buffer: UniformBuffer,
    data: CameraUboData,
    dirty: bool,
}

impl CameraUbo {
    pub fn new() -> Self {
        Self {
            buffer: UniformBuffer::new(CameraUboData::SIZE, CAMERA_UBO_BINDING),
            data: CameraUboData::new(),
            dirty: true,
        }
    }

    pub fn set_view(&mut self, view: &Mat4) {
        self.data.set_view(view);
        self.dirty = true;
    }

    pub fn set_view_inv(&mut self, view_inv: &Mat4) {
        self.data.set_view_inv(view_inv);
        self.dirty = true;
    }

    pub fn set_proj(&mut self, proj: &Mat4) {
        self.data.set_proj(proj);
        self.dirty = true;
    }

    pub fn set_eye(&mut self, eye: Vec3) {
        self.data.set_eye(eye);
        self.dirty = true;
    }

    pub fn set_star_dir(&mut self, dir: Vec3) {
        self.data.set_star_dir(dir);
        self.dirty = true;
    }

    /// Flush changes to GPU if dirty
    pub fn flush(&mut self) {
        if self.dirty {
            self.buffer.update(self.data.as_bytes());
            self.dirty = false;
        }
    }

    /// Force upload regardless of dirty flag
    pub fn force_upload(&mut self) {
        self.buffer.update(self.data.as_bytes());
        self.dirty = false;
    }

    pub fn data(&self) -> &CameraUboData {
        &self.data
    }

    pub fn buffer(&self) -> &UniformBuffer {
        &self.buffer
    }
}

impl Default for CameraUbo {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_ubo_size() {
        // Verify std140 layout produces expected size
        // 4 mat4 (64 each) + 2 vec4 (16 each) = 256 + 32 = 288 bytes
        assert_eq!(CameraUboData::SIZE, 288);
    }

    #[test]
    fn test_camera_ubo_alignment() {
        // Verify alignment is 16 bytes for std140
        assert_eq!(std::mem::align_of::<CameraUboData>(), 16);
    }
}
