use super::gl;

#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum TexFilter {
    Point = gl::NEAREST,
    PointMipPoint = gl::NEAREST_MIPMAP_NEAREST,
    PointMipLinear = gl::NEAREST_MIPMAP_LINEAR,
    Linear = gl::LINEAR,
    LinearMipPoint = gl::LINEAR_MIPMAP_NEAREST,
    LinearMipLinear = gl::LINEAR_MIPMAP_LINEAR,
}
