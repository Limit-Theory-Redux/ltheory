use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use glam::Vec2;
extern "C" {
    fn Hash_XX64(buf: *const libc::c_void, len: libc::c_int, seed: uint64) -> uint64;
    fn cos(_: libc::c_double) -> libc::c_double;
    fn sin(_: libc::c_double) -> libc::c_double;
    fn log(_: libc::c_double) -> libc::c_double;
    fn ldexp(_: libc::c_double, _: libc::c_int) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn floor(_: libc::c_double) -> libc::c_double;
    fn TimeStamp_Get() -> TimeStamp;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RNG {
    pub seed: uint64,
    pub state: [uint64; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Quat {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type TimeStamp = uint64;

#[inline]
unsafe extern "C" fn Floor(mut t: libc::c_double) -> libc::c_double {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Log(mut t: libc::c_double) -> libc::c_double {
    return log(t);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Round(mut t: libc::c_double) -> libc::c_double {
    return Floor(t + 0.5f64);
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}
#[inline]
unsafe extern "C" fn Cos(mut t: libc::c_double) -> libc::c_double {
    return cos(t);
}
#[inline]
unsafe extern "C" fn Sin(mut t: libc::c_double) -> libc::c_double {
    return sin(t);
}
#[inline]
unsafe extern "C" fn Random_SplitMix64(mut state: *mut uint64) -> uint64 {
    *state = (*state as libc::c_ulonglong)
        .wrapping_add(0x9e3779b97f4a7c15 as libc::c_ulonglong) as uint64 as uint64;
    let mut z: uint64 = *state;
    z = (z ^ z >> 30 as libc::c_int)
        .wrapping_mul(0xbf58476d1ce4e5b9 as libc::c_ulonglong);
    z = (z ^ z >> 27 as libc::c_int)
        .wrapping_mul(0x94d049bb133111eb as libc::c_ulonglong);
    return z ^ z >> 31 as libc::c_int;
}
#[inline]
unsafe extern "C" fn rotl(x: uint64, mut k: libc::c_int) -> uint64 {
    return x << k | x >> 64 as libc::c_int - k;
}
#[inline]
unsafe extern "C" fn Random_Xoroshiro128(
    mut state0: *mut uint64,
    mut state1: *mut uint64,
) -> uint64 {
    let s0: uint64 = *state0;
    let mut s1: uint64 = *state1;
    let result: uint64 = s0.wrapping_add(s1);
    s1 ^= s0;
    *state0 = rotl(s0, 55 as libc::c_int) ^ s1 ^ s1 << 14 as libc::c_int;
    *state1 = rotl(s1, 36 as libc::c_int);
    return result;
}

#[inline]
unsafe extern "C" fn RNG_Next64(mut self_0: *mut RNG) -> uint64 {
    return Random_Xoroshiro128(
        &mut *((*self_0).state).as_mut_ptr().offset(0),
        &mut *((*self_0).state).as_mut_ptr().offset(1),
    );
}
#[inline]
unsafe extern "C" fn RNG_Next32(mut self_0: *mut RNG) -> uint32 {
    return (RNG_Next64(self_0) & 0xffffffff as libc::c_uint as libc::c_ulonglong)
        as uint32;
}
#[inline]
unsafe extern "C" fn RNG_Init(mut self_0: *mut RNG) {
    let mut seed: uint64 = (*self_0).seed;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 64 as libc::c_int {
        seed = Random_SplitMix64(&mut seed);
        i += 1;
    }
    (*self_0).state[0] = Random_SplitMix64(&mut seed);
    (*self_0).state[1] = Random_SplitMix64(&mut seed);
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < 64 as libc::c_int {
        RNG_Next64(self_0);
        i_0 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Create(mut seed: uint64) -> *mut RNG {
    let mut self_0: *mut RNG = MemAlloc(::core::mem::size_of::<RNG>())
        as *mut RNG;
    (*self_0).seed = seed;
    RNG_Init(self_0);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_FromStr(mut s: cstr) -> *mut RNG {
    return RNG_Create(
        Hash_XX64(
            s as *const libc::c_void,
            StrLen(s) as libc::c_int,
            0 as libc::c_int as uint64,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn RNG_FromTime() -> *mut RNG {
    return RNG_Create(TimeStamp_Get());
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Free(mut self_0: *mut RNG) {
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Rewind(mut self_0: *mut RNG) {
    RNG_Init(self_0);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Chance(
    mut self_0: *mut RNG,
    mut probability: libc::c_double,
) -> bool {
    return RNG_GetUniform(self_0) < probability;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Get31(mut self_0: *mut RNG) -> int32 {
    let mut i: uint32 = RNG_Next32(self_0) & 0x7fffffff as libc::c_uint;
    return *(&mut i as *mut uint32 as *mut int32);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Get32(mut self_0: *mut RNG) -> uint32 {
    return RNG_Next32(self_0);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_Get64(mut self_0: *mut RNG) -> uint64 {
    return RNG_Next64(self_0);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetAngle(mut self_0: *mut RNG) -> libc::c_double {
    return 6.28318531f32 as libc::c_double * RNG_GetUniform(self_0);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetInt(
    mut self_0: *mut RNG,
    mut lower: libc::c_int,
    mut upper: libc::c_int,
) -> libc::c_int {
    let mut t: libc::c_double = RNG_GetUniform(self_0);
    return Round(lower as libc::c_double + t * (upper - lower) as libc::c_double)
        as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetRNG(mut self_0: *mut RNG) -> *mut RNG {
    return RNG_Create(RNG_Get64(self_0));
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetUniform(mut self_0: *mut RNG) -> libc::c_double {
    return ldexp(RNG_Next32(self_0) as libc::c_double, -(32 as libc::c_int));
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetUniformRange(
    mut self_0: *mut RNG,
    mut lower: libc::c_double,
    mut upper: libc::c_double,
) -> libc::c_double {
    let mut t: libc::c_double = ldexp(
        RNG_Next32(self_0) as libc::c_double,
        -(32 as libc::c_int),
    );
    return lower + t * (upper - lower);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetErlang(
    mut self_0: *mut RNG,
    mut k: libc::c_int,
) -> libc::c_double {
    let mut sum: libc::c_double = 0.0f64;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < k {
        sum += RNG_GetExp(self_0);
        i += 1;
    }
    return sum / k as libc::c_double;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetExp(mut self_0: *mut RNG) -> libc::c_double {
    return -Log(Max(1.0f64 - RNG_GetUniform(self_0), 2.2204460492503131e-16f64));
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetGaussian(mut self_0: *mut RNG) -> libc::c_double {
    let mut angle: libc::c_double = RNG_GetAngle(self_0);
    let mut radius: libc::c_double = 1.0f64 - RNG_GetUniform(self_0);
    return Cos(angle) * Sqrt(-2.0f64 * Log(radius));
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetAxis2(mut self_0: *mut RNG, mut out: *mut Vec2) {
    *out = Vec2::new(0.0f32, 0.0f32);
    let mut axis: libc::c_int = RNG_GetInt(self_0, 0 as libc::c_int, 3 as libc::c_int);
    match axis {
        0 => {
            (*out).x = 1.0f32;
            return;
        }
        1 => {
            (*out).x = -1.0f32;
            return;
        }
        2 => {
            (*out).y = 1.0f32;
            return;
        }
        3 => {
            (*out).y = -1.0f32;
            return;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetAxis3(mut self_0: *mut RNG, mut out: *mut Vec3) {
    *out = Vec3::ZERO;
    let mut axis: libc::c_int = RNG_GetInt(self_0, 0 as libc::c_int, 5 as libc::c_int);
    match axis {
        0 => {
            (*out).x = 1.0f32;
            return;
        }
        1 => {
            (*out).x = -1.0f32;
            return;
        }
        2 => {
            (*out).y = 1.0f32;
            return;
        }
        3 => {
            (*out).y = -1.0f32;
            return;
        }
        4 => {
            (*out).z = 1.0f32;
            return;
        }
        5 => {
            (*out).z = -1.0f32;
            return;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetDir2(mut self_0: *mut RNG, mut out: *mut Vec2) {
    let mut angle: libc::c_double = RNG_GetAngle(self_0);
    *out = Vec2::new(Cos(angle) as libc::c_float, Sin(angle) as libc::c_float);
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetDir3(mut self_0: *mut RNG, mut out: *mut Vec3) {
    loop {
        let mut x: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        let mut y: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        let mut z: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        let mut m2: libc::c_double = x * x + y * y + z * z;
        if m2 <= 1.0f64 && m2 > 1e-6f64 {
            m2 = Sqrt(m2);
            (*out).x = (x / m2) as libc::c_float;
            (*out).y = (y / m2) as libc::c_float;
            (*out).z = (z / m2) as libc::c_float;
            return;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetDisc(mut self_0: *mut RNG, mut out: *mut Vec2) {
    loop {
        let mut x: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        let mut y: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        if x * x + y * y <= 1.0f64 {
            (*out).x = x as libc::c_float;
            (*out).y = y as libc::c_float;
            return;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetSign(mut self_0: *mut RNG) -> libc::c_double {
    return if RNG_GetUniform(self_0) > 0.5f64 { 1.0f64 } else { -1.0f64 };
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetSphere(mut self_0: *mut RNG, mut out: *mut Vec3) {
    loop {
        let mut x: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        let mut y: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        let mut z: libc::c_double = 2.0f64 * RNG_GetUniform(self_0) - 1.0f64;
        if x * x + y * y + z * z <= 1.0f64 {
            (*out).x = x as libc::c_float;
            (*out).y = y as libc::c_float;
            (*out).z = z as libc::c_float;
            return;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetVec2(
    mut self_0: *mut RNG,
    mut out: *mut Vec2,
    mut lower: libc::c_double,
    mut upper: libc::c_double,
) {
    (*out).x = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
    (*out).y = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetVec3(
    mut self_0: *mut RNG,
    mut out: *mut Vec3,
    mut lower: libc::c_double,
    mut upper: libc::c_double,
) {
    (*out).x = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
    (*out).y = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
    (*out).z = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetVec4(
    mut self_0: *mut RNG,
    mut out: *mut Vec4f,
    mut lower: libc::c_double,
    mut upper: libc::c_double,
) {
    (*out).x = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
    (*out).y = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
    (*out).z = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
    (*out).w = RNG_GetUniformRange(self_0, lower, upper) as libc::c_float;
}
#[no_mangle]
pub unsafe extern "C" fn RNG_GetQuat(mut self_0: *mut RNG, mut out: *mut Quat) {
    let mut p0 = Vec2::ZERO;
    let mut p1 = Vec2::ZERO;
    RNG_GetDisc(self_0, &mut p0);
    RNG_GetDisc(self_0, &mut p1);
    let mut d0 = p0.length_squared() as f64;
    let mut d1 = p1.length_squared() as f64 + 2.2204460492503131e-16f64;
    let mut s = Sqrt((1.0f64 - d0) / d1);
    (*out).x = p0.y;
    (*out).y = (p1.x as libc::c_double * s) as libc::c_float;
    (*out).z = (p1.y as libc::c_double * s) as libc::c_float;
    (*out).w = p0.x;
}
