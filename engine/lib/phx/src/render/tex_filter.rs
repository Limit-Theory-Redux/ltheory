#![allow(unsafe_code)] // TODO: remove

use super::gl;

pub type TexFilter = i32;

#[no_mangle]
pub static TexFilter_Point: TexFilter = gl::NEAREST as TexFilter;

#[no_mangle]
pub static TexFilter_PointMipPoint: TexFilter = gl::NEAREST_MIPMAP_NEAREST as TexFilter;

#[no_mangle]
pub static TexFilter_PointMipLinear: TexFilter = gl::NEAREST_MIPMAP_LINEAR as TexFilter;

#[no_mangle]
pub static TexFilter_Linear: TexFilter = gl::LINEAR as TexFilter;

#[no_mangle]
pub static TexFilter_LinearMipPoint: TexFilter = gl::LINEAR_MIPMAP_NEAREST as TexFilter;

#[no_mangle]
pub static TexFilter_LinearMipLinear: TexFilter = gl::LINEAR_MIPMAP_LINEAR as TexFilter;
