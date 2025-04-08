#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy)]
pub enum TexFilter {
    Point = 0x2600,           // gl::NEAREST
    PointMipPoint = 0x2700,   // gl::NEAREST_MIPMAP_NEAREST
    PointMipLinear = 0x2702,  // gl::NEAREST_MIPMAP_LINEAR
    Linear = 0x2601,          // gl::LINEAR
    LinearMipPoint = 0x2701,  // gl::LINEAR_MIPMAP_NEAREST
    LinearMipLinear = 0x2703, // gl::LINEAR_MIPMAP_LINEAR
}
