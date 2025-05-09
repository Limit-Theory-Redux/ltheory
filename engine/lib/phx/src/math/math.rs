// Re-export glam types.
pub use glam::{
    DVec2, DVec3, DVec4, IVec2, IVec3, IVec4, Vec2, Vec3, Vec4, dvec2, dvec3, dvec4, ivec2, ivec3,
    ivec4, vec2, vec3, vec4,
};

use super::lerp;

pub struct Math;

#[luajit_ffi_gen::luajit_ffi]
impl Math {
    pub fn bezier3(x: f64, y1: f64, y2: f64, y3: f64) -> f64 {
        let y12 = lerp(y1, y2, x);
        let y23 = lerp(y2, y3, x);
        lerp(y12, y23, x)
    }

    pub fn bezier4(x: f64, y1: f64, y2: f64, y3: f64, y4: f64) -> f64 {
        let y12 = lerp(y1, y2, x);
        let y23 = lerp(y2, y3, x);
        let y34 = lerp(y3, y4, x);
        let y123 = lerp(y12, y23, x);
        let y234 = lerp(y23, y34, x);
        lerp(y123, y234, x)
    }

    pub fn clamp(x: f64, a: f64, b: f64) -> f64 {
        x.clamp(a, b)
    }

    pub fn clamp01(x: f64) -> f64 {
        x.clamp(0.0, 1.0)
    }

    pub fn clamp_safe(x: f64, a: f64, b: f64) -> f64 {
        if b < a { x.clamp(b, a) } else { x.clamp(a, b) }
    }

    pub fn clamp_unit(x: f64) -> f64 {
        f64::clamp(x, -1.0, 1.0)
    }

    pub fn exp_map(x: f64, p: f64) -> f64 {
        1.0 - f64::exp(-f64::powf(f64::abs(x), p))
    }

    pub fn exp_map_signed(x: f64, p: f64) -> f64 {
        f64::signum(x) * (1.0 - f64::exp(-f64::powf(f64::abs(x), p)))
    }

    pub fn exp_map1(x: f64) -> f64 {
        1.0 - f64::exp(-f64::abs(x))
    }

    pub fn exp_map1_signed(x: f64) -> f64 {
        f64::signum(x) * (1.0 - f64::exp(-f64::abs(x)))
    }

    pub fn exp_map2(x: f64) -> f64 {
        1.0 - f64::exp(-x * x)
    }

    pub fn exp_map2_signed(x: f64) -> f64 {
        f64::signum(x) * (1.0 - f64::exp(-x * x))
    }

    pub fn pow_signed(x: f64, p: f64) -> f64 {
        f64::signum(x) * f64::powf(f64::abs(x), p)
    }

    pub fn round(x: f64) -> f64 {
        f64::round(x)
    }

    pub fn sign(x: f64) -> f64 {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            0.0
        }
    }
}
