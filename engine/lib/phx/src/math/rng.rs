use std::hash::{DefaultHasher, Hash, Hasher};

use glam::{Quat, Vec2, Vec3, Vec4};

use crate::system::TimeStamp;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rng {
    pub seed: u64,
    pub state: [u64; 2],
}

impl Rng {
    #[inline]
    fn random_split_mix64(state: &mut u64) -> u64 {
        *state = (*state).wrapping_add(0x9e3779b97f4a7c15);
        let mut z = *state;
        z = (z ^ z >> 30).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ z >> 27).wrapping_mul(0x94d049bb133111eb);
        z ^ z >> 31
    }

    #[inline]
    fn rotl(x: u64, k: i32) -> u64 {
        x << k | x >> (64 - k)
    }

    #[inline]
    fn random_xoroshiro128(state: &mut [u64; 2]) -> u64 {
        let s0 = state[0];
        let mut s1 = state[1];
        let result = s0.wrapping_add(s1);
        s1 ^= s0;
        state[0] = Self::rotl(s0, 55) ^ s1 ^ s1 << 14;
        state[1] = Self::rotl(s1, 36);
        result
    }

    #[inline]
    fn next64(&mut self) -> u64 {
        Self::random_xoroshiro128(&mut self.state)
    }

    #[inline]
    fn next32(&mut self) -> u32 {
        (self.next64() & 0xffffffff) as u32
    }

    #[inline]
    fn init(&mut self) {
        let mut seed: u64 = self.seed;
        let mut i = 0;
        while i < 64 {
            seed = Self::random_split_mix64(&mut seed);
            i += 1;
        }
        self.state[0] = Self::random_split_mix64(&mut seed);
        self.state[1] = Self::random_split_mix64(&mut seed);
        let mut i_0: i32 = 0;
        while i_0 < 64 {
            self.next64();
            i_0 += 1;
        }
    }
}

#[luajit_ffi_gen::luajit_ffi(name = "RNG")]
impl Rng {
    #[bind(name = "Create")]
    pub fn new(seed: u64) -> Self {
        let mut this = Self {
            seed,
            state: [0; 2],
        };
        this.init();
        this
    }

    #[bind(name = "FromStr")]
    pub fn from_string(s: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        Self::new(hasher.finish()) //unsafe { Hash_XX64(s as *const _, s.as_str().len() as i32, 0) };
    }

    pub fn from_time() -> Self {
        Self::new(TimeStamp::now().to_seconds())
    }

    pub fn rewind(&mut self) {
        self.init();
    }

    pub fn chance(&mut self, probability: f64) -> bool {
        self.get_uniform() < probability
    }

    pub fn get31(&mut self) -> i32 {
        (self.next32() & 0x7fffffff) as i32
    }

    pub fn get32(&mut self) -> u32 {
        self.next32()
    }

    pub fn get64(&mut self) -> u64 {
        self.next64()
    }

    pub fn get_angle(&mut self) -> f64 {
        std::f64::consts::TAU * self.get_uniform()
    }

    pub fn get_int(&mut self, lower: i32, upper: i32) -> i32 {
        let t = self.get_uniform();
        f64::round(lower as f64 + t * upper.wrapping_sub(lower) as f64) as i32
    }

    #[bind(name = "GetRNG")]
    pub fn get_rng(&mut self) -> Self {
        Self::new(self.get64())
    }

    pub fn get_uniform(&mut self) -> f64 {
        self.next32() as f64 * f64::exp2(-32.0)
    }

    pub fn get_uniform_range(&mut self, lower: f64, upper: f64) -> f64 {
        let t = self.next32() as f64 * f64::exp2(-32.0);
        lower + t * (upper - lower)
    }

    pub fn get_erlang(&mut self, k: i32) -> f64 {
        let mut sum = 0.0;
        let mut i = 0;
        while i < k {
            sum += self.get_exp();
            i += 1;
        }
        sum / k as f64
    }

    pub fn get_exp(&mut self) -> f64 {
        -f64::ln(f64::max(1.0f64 - self.get_uniform(), f64::EPSILON))
    }

    pub fn get_gaussian(&mut self) -> f64 {
        let angle = self.get_angle();
        let radius = 1.0 - self.get_uniform();
        f64::cos(angle) * f64::sqrt(-2.0 * f64::ln(radius))
    }

    pub fn get_axis2(&mut self, out: &mut Vec2) {
        *out = Vec2::ZERO;
        let axis = self.get_int(0, 3);
        match axis {
            0 => out.x = 1.0,
            1 => out.x = -1.0,
            2 => out.y = 1.0,
            3 => out.y = -1.0,
            _ => {}
        }
    }

    pub fn get_axis3(&mut self, out: &mut Vec3) {
        *out = Vec3::ZERO;
        let axis = self.get_int(0, 5);
        match axis {
            0 => out.x = 1.0,
            1 => out.x = -1.0,
            2 => out.y = 1.0,
            3 => out.y = -1.0,
            4 => out.z = 1.0,
            5 => out.z = -1.0,
            _ => {}
        }
    }

    pub fn get_dir2(&mut self, out: &mut Vec2) {
        let angle = self.get_angle();
        *out = Vec2::new(f64::cos(angle) as f32, f64::sin(angle) as f32);
    }

    pub fn get_dir3(&mut self, out: &mut Vec3) {
        loop {
            let x = 2.0 * self.get_uniform() - 1.0;
            let y = 2.0 * self.get_uniform() - 1.0;
            let z = 2.0 * self.get_uniform() - 1.0;
            let mut m2 = x * x + y * y + z * z;
            if m2 <= 1.0 && m2 > 1e-6 {
                m2 = f64::sqrt(m2);
                out.x = (x / m2) as f32;
                out.y = (y / m2) as f32;
                out.z = (z / m2) as f32;
                return;
            }
        }
    }

    pub fn get_disc(&mut self, out: &mut Vec2) {
        loop {
            let x = 2.0 * self.get_uniform() - 1.0;
            let y = 2.0 * self.get_uniform() - 1.0;
            if x * x + y * y <= 1.0 {
                out.x = x as f32;
                out.y = y as f32;
                return;
            }
        }
    }

    pub fn get_sign(&mut self) -> f64 {
        if self.get_uniform() > 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    pub fn get_sphere(&mut self, out: &mut Vec3) {
        loop {
            let x = 2.0 * self.get_uniform() - 1.0;
            let y = 2.0 * self.get_uniform() - 1.0;
            let z = 2.0 * self.get_uniform() - 1.0;
            if x * x + y * y + z * z <= 1.0 {
                out.x = x as f32;
                out.y = y as f32;
                out.z = z as f32;
                return;
            }
        }
    }

    pub fn get_vec2(&mut self, out: &mut Vec2, lower: f64, upper: f64) {
        out.x = self.get_uniform_range(lower, upper) as f32;
        out.y = self.get_uniform_range(lower, upper) as f32;
    }

    pub fn get_vec3(&mut self, out: &mut Vec3, lower: f64, upper: f64) {
        out.x = self.get_uniform_range(lower, upper) as f32;
        out.y = self.get_uniform_range(lower, upper) as f32;
        out.z = self.get_uniform_range(lower, upper) as f32;
    }

    pub fn get_vec4(&mut self, out: &mut Vec4, lower: f64, upper: f64) {
        out.x = self.get_uniform_range(lower, upper) as f32;
        out.y = self.get_uniform_range(lower, upper) as f32;
        out.z = self.get_uniform_range(lower, upper) as f32;
        out.w = self.get_uniform_range(lower, upper) as f32;
    }

    pub fn get_quat(&mut self, out: &mut Quat) {
        let mut p0 = Vec2::ZERO;
        let mut p1 = Vec2::ZERO;
        self.get_disc(&mut p0);
        self.get_disc(&mut p1);
        let d0 = p0.length_squared() as f64;
        let d1 = p1.length_squared() as f64 + f64::EPSILON;
        let s = f64::sqrt((1.0f64 - d0) / d1);
        out.x = p0.y;
        out.y = (p1.x as f64 * s) as f32;
        out.z = (p1.y as f64 * s) as f32;
        out.w = p0.x;
    }
}
