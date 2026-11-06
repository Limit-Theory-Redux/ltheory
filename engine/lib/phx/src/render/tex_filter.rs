#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexFilter {
    Point,
    PointMipPoint,
    PointMipLinear,
    Linear,
    LinearMipPoint,
    LinearMipLinear,
}
