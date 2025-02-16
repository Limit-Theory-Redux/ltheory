mod bit;
mod box3;
mod clip_rect;
mod intersect;
mod line_segment;
mod math;
mod matrix;
mod octree;
mod plane;
mod polygon;
mod position;
mod quat;
mod ray;
mod rng;
mod triangle;

pub use bit::*;
pub use box3::*;
pub use clip_rect::*;
pub use intersect::*;
pub use line_segment::*;
pub use math::*;
pub use matrix::*;
pub use octree::*;
pub use plane::*;
pub use polygon::*;
pub use position::*;
pub use quat::*;
pub use ray::*;
pub use rng::*;
pub use triangle::*;

use crate::error::Error;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    pub p: Vec3,
    pub r: f32,
}

#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

#[inline]
pub fn saturate(t: f64) -> f64 {
    f64::clamp(t, 0.0, 1.0)
}

#[inline]
pub fn approximately_equal_f64(x: f64, y: f64) -> bool {
    f64::abs(x - y) < 1e-3
}

#[inline]
pub fn approximately_equal(x: f32, y: f32) -> bool {
    f32::abs(x - y) < 1e-3
}

#[inline]
pub fn validatef(x: f32) -> Error {
    // TODO: this condition is always true, what is the point in it? Bad conversion from C?
    let classification = if std::mem::size_of::<f32>() == std::mem::size_of::<f32>() {
        f32::classify(x) as u32
    } else if std::mem::size_of::<f32>() == std::mem::size_of::<f64>() {
        f64::classify(x as f64) as u32
    } else {
        3
    };
    match classification {
        1 => return 0x20 as Error,
        2 => return 0x4 as Error,
        3 | 4 => return 0 as Error,
        5 => {}
        _ => {
            panic!("validatef: Unhandled case: {classification}");
        }
    }
    0 as Error
}

#[inline]
pub fn validate_f64(x: f64) -> Error {
    let classification = if std::mem::size_of::<f64>() as libc::c_ulong
        == std::mem::size_of::<f32>() as libc::c_ulong
    {
        f32::classify(x as f32) as u32
    } else if std::mem::size_of::<f64>() as libc::c_ulong
        == std::mem::size_of::<f64>() as libc::c_ulong
    {
        f64::classify(x) as u32
    } else {
        3
    };
    match classification {
        1 => 0x20 as Error,
        2 => 0x4 as Error,
        3 | 4 => 0 as Error,
        5 => 0x8 as Error,
        _ => {
            panic!("validate_f64: Unhandled case: {classification}");
        }
    }
}

#[inline]
pub fn validate_vec2(v: Vec2) -> Error {
    let mut e = 0 as Error;
    e |= validatef(v.x);
    e |= validatef(v.y);
    e
}

#[inline]
pub fn validate_vec3(v: Vec3) -> Error {
    let mut e = 0 as Error;
    e |= validatef(v.x);
    e |= validatef(v.y);
    e |= validatef(v.z);
    e
}

#[inline]
pub fn reject_vec3(a: Vec3, b: Vec3) -> Vec3 {
    let d = a.dot(b);
    a - d * b
}
