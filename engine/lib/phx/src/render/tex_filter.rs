pub type TexFilter = i32;

#[no_mangle]
pub static TexFilter_Point: TexFilter = 1; //gl::NEAREST as TexFilter;

#[no_mangle]
pub static TexFilter_PointMipPoint: TexFilter = 1; //gl::NEAREST_MIPMAP_NEAREST as TexFilter;

#[no_mangle]
pub static TexFilter_PointMipLinear: TexFilter = 1; //gl::NEAREST_MIPMAP_LINEAR as TexFilter;

#[no_mangle]
pub static TexFilter_Linear: TexFilter = 1; //gl::LINEAR as TexFilter;

#[no_mangle]
pub static TexFilter_LinearMipPoint: TexFilter = 1; //gl::LINEAR_MIPMAP_NEAREST as TexFilter;

#[no_mangle]
pub static TexFilter_LinearMipLinear: TexFilter = 1; //gl::LINEAR_MIPMAP_LINEAR as TexFilter;
