use crate::*;

use crate::hash::*;
use crate::internal::*;
use crate::math::*;
use crate::time_stamp::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rng {
    pub seed: u64,
    pub state: [u64; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
unsafe extern "C" fn Random_SplitMix64(state: *mut u64) -> u64 {
    *state = (*state as u64).wrapping_add(0x9e3779b97f4a7c15);
    let mut z: u64 = *state;
    z = (z ^ z >> 30_i32).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ z >> 27_i32).wrapping_mul(0x94d049bb133111eb);
    z ^ z >> 31
}

#[inline]
extern "C" fn rotl(x: u64, k: i32) -> u64 {
    x << k | x >> 64 - k
}

#[inline]
unsafe extern "C" fn Random_Xoroshiro128(state0: *mut u64, state1: *mut u64) -> u64 {
    let s0: u64 = *state0;
    let mut s1: u64 = *state1;
    let result: u64 = s0.wrapping_add(s1);
    s1 ^= s0;
    *state0 = rotl(s0, 55) ^ s1 ^ s1 << 14;
    *state1 = rotl(s1, 36);
    result
}

#[inline]
unsafe extern "C" fn RNG_Next64(this: &mut Rng) -> u64 {
    Random_Xoroshiro128(
        &mut *(this.state).as_mut_ptr().offset(0),
        &mut *(this.state).as_mut_ptr().offset(1),
    )
}

#[inline]
unsafe extern "C" fn RNG_Next32(this: &mut Rng) -> u32 {
    (RNG_Next64(this) & 0xffffffff) as u32
}

#[inline]
unsafe extern "C" fn RNG_Init(this: &mut Rng) {
    let mut seed: u64 = this.seed;
    let mut i: i32 = 0;
    while i < 64 {
        seed = Random_SplitMix64(&mut seed);
        i += 1;
    }
    this.state[0] = Random_SplitMix64(&mut seed);
    this.state[1] = Random_SplitMix64(&mut seed);
    let mut i_0: i32 = 0;
    while i_0 < 64 {
        RNG_Next64(this);
        i_0 += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_Create(seed: u64) -> Box<Rng> {
    let mut this = Box::new(Rng {
        seed,
        state: [0u64; 2],
    });
    RNG_Init(this.as_mut());
    this
}

#[no_mangle]
pub unsafe extern "C" fn RNG_FromStr(s: *const libc::c_char) -> Box<Rng> {
    RNG_Create(Hash_XX64(s as *const _, s.convert().len() as i32, 0))
}

#[no_mangle]
pub unsafe extern "C" fn RNG_FromTime() -> Box<Rng> {
    RNG_Create(TimeStamp_Get())
}

#[no_mangle]
pub extern "C" fn RNG_Free(_: Option<Box<Rng>>) {}

#[no_mangle]
pub unsafe extern "C" fn RNG_Rewind(this: &mut Rng) {
    RNG_Init(this);
}

#[no_mangle]
pub unsafe extern "C" fn RNG_Chance(this: &mut Rng, probability: f64) -> bool {
    RNG_GetUniform(this) < probability
}

#[no_mangle]
pub unsafe extern "C" fn RNG_Get31(this: &mut Rng) -> i32 {
    let mut i: u32 = RNG_Next32(this) & 0x7fffffff;
    *(&mut i as *mut u32 as *mut i32)
}

#[no_mangle]
pub unsafe extern "C" fn RNG_Get32(this: &mut Rng) -> u32 {
    RNG_Next32(this)
}

#[no_mangle]
pub unsafe extern "C" fn RNG_Get64(this: &mut Rng) -> u64 {
    RNG_Next64(this)
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetAngle(this: &mut Rng) -> f64 {
    std::f64::consts::TAU * RNG_GetUniform(this)
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetInt(this: &mut Rng, lower: i32, upper: i32) -> i32 {
    let t: f64 = RNG_GetUniform(this);
    f64::round(lower as f64 + t * upper.wrapping_sub(lower) as f64) as i32
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetRNG(this: &mut Rng) -> Box<Rng> {
    RNG_Create(RNG_Get64(this))
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetUniform(this: &mut Rng) -> f64 {
    RNG_Next32(this) as f64 * f64::exp2(-32.0)
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetUniformRange(this: &mut Rng, lower: f64, upper: f64) -> f64 {
    let t: f64 = RNG_Next32(this) as f64 * f64::exp2(-32.0);
    lower + t * (upper - lower)
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetErlang(this: &mut Rng, k: i32) -> f64 {
    let mut sum: f64 = 0.0f64;
    let mut i: i32 = 0;
    while i < k {
        sum += RNG_GetExp(this);
        i += 1;
    }
    sum / k as f64
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetExp(this: &mut Rng) -> f64 {
    -f64::ln(f64::max(1.0f64 - RNG_GetUniform(this), f64::EPSILON))
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetGaussian(this: &mut Rng) -> f64 {
    let angle: f64 = RNG_GetAngle(this);
    let radius: f64 = 1.0f64 - RNG_GetUniform(this);
    f64::cos(angle) * f64::sqrt(-2.0f64 * f64::ln(radius))
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetAxis2(this: &mut Rng, out: *mut Vec2) {
    *out = Vec2::ZERO;
    let axis: i32 = RNG_GetInt(this, 0, 3);
    match axis {
        0 => {
            (*out).x = 1.0f32;
        }
        1 => {
            (*out).x = -1.0f32;
        }
        2 => {
            (*out).y = 1.0f32;
        }
        3 => {
            (*out).y = -1.0f32;
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetAxis3(this: &mut Rng, out: &mut Vec3) {
    *out = Vec3::ZERO;
    let axis: i32 = RNG_GetInt(this, 0, 5);
    match axis {
        0 => {
            out.x = 1.0f32;
        }
        1 => {
            out.x = -1.0f32;
        }
        2 => {
            out.y = 1.0f32;
        }
        3 => {
            out.y = -1.0f32;
        }
        4 => {
            out.z = 1.0f32;
        }
        5 => {
            out.z = -1.0f32;
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetDir2(this: &mut Rng, out: *mut Vec2) {
    let angle: f64 = RNG_GetAngle(this);
    *out = Vec2::new(f64::cos(angle) as f32, f64::sin(angle) as f32);
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetDir3(this: &mut Rng, out: *mut Vec3) {
    loop {
        let x: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        let y: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        let z: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        let mut m2: f64 = x * x + y * y + z * z;
        if m2 <= 1.0f64 && m2 > 1e-6f64 {
            m2 = f64::sqrt(m2);
            (*out).x = (x / m2) as f32;
            (*out).y = (y / m2) as f32;
            (*out).z = (z / m2) as f32;
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetDisc(this: &mut Rng, out: *mut Vec2) {
    loop {
        let x: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        let y: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        if x * x + y * y <= 1.0f64 {
            (*out).x = x as f32;
            (*out).y = y as f32;
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetSign(this: &mut Rng) -> f64 {
    if RNG_GetUniform(this) > 0.5f64 {
        1.0f64
    } else {
        -1.0f64
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetSphere(this: &mut Rng, out: *mut Vec3) {
    loop {
        let x: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        let y: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        let z: f64 = 2.0f64 * RNG_GetUniform(this) - 1.0f64;
        if x * x + y * y + z * z <= 1.0f64 {
            (*out).x = x as f32;
            (*out).y = y as f32;
            (*out).z = z as f32;
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetVec2(this: &mut Rng, out: *mut Vec2, lower: f64, upper: f64) {
    (*out).x = RNG_GetUniformRange(this, lower, upper) as f32;
    (*out).y = RNG_GetUniformRange(this, lower, upper) as f32;
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetVec3(this: &mut Rng, out: *mut Vec3, lower: f64, upper: f64) {
    (*out).x = RNG_GetUniformRange(this, lower, upper) as f32;
    (*out).y = RNG_GetUniformRange(this, lower, upper) as f32;
    (*out).z = RNG_GetUniformRange(this, lower, upper) as f32;
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetVec4(this: &mut Rng, out: *mut Vec4, lower: f64, upper: f64) {
    (*out).x = RNG_GetUniformRange(this, lower, upper) as f32;
    (*out).y = RNG_GetUniformRange(this, lower, upper) as f32;
    (*out).z = RNG_GetUniformRange(this, lower, upper) as f32;
    (*out).w = RNG_GetUniformRange(this, lower, upper) as f32;
}

#[no_mangle]
pub unsafe extern "C" fn RNG_GetQuat(this: &mut Rng, out: *mut Quat) {
    let mut p0 = Vec2::ZERO;
    let mut p1 = Vec2::ZERO;
    RNG_GetDisc(this, &mut p0);
    RNG_GetDisc(this, &mut p1);
    let d0 = p0.length_squared() as f64;
    let d1 = p1.length_squared() as f64 + f64::EPSILON;
    let s = f64::sqrt((1.0f64 - d0) / d1);
    (*out).x = p0.y;
    (*out).y = (p1.x as f64 * s) as f32;
    (*out).z = (p1.y as f64 * s) as f32;
    (*out).w = p0.x;
}
