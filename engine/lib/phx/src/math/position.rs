use core::ops::*;

use super::*;

/// Create a position.
#[inline(always)]
pub const fn position(x: f64, y: f64, z: f64) -> Position {
    Position::new(x, y, z)
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Position {
    /// The translation component of this position, represented as a double (64-bit) vector.
    pub v: DVec3,
}

impl Position {
    /// All zeroes, the origin.
    pub const ZERO: Self = Position::from_dvec(DVec3::ZERO);

    pub const fn new(x: f64, y: f64, z: f64) -> Position {
        Position {
            v: DVec3::new(x, y, z),
        }
    }

    pub const fn from_dvec(v: DVec3) -> Position {
        Position { v }
    }

    pub fn from_vec(v: Vec3) -> Position {
        Position { v: v.as_dvec3() }
    }
}

impl Position {
    /// Returns this position relative to 'frame''s frame of reference.
    ///
    /// This is usually used to turn this world position into a 32-bit translation relative to a camera.
    pub fn relative_to(&self, frame: Position) -> Vec3 {
        (self.v - frame.v).as_vec3()
    }

    /// Computes the euclidean distance between two points in space.
    pub fn distance(&self, rhs: Position) -> f64 {
        self.v.distance(rhs.v)
    }

    /// Computes the squared euclidean distance between two points in space.
    pub fn distance_squared(&self, rhs: Position) -> f64 {
        self.v.distance_squared(rhs.v)
    }

    /// Converts this Position into a 32-bit vector. Note that this loses precision.
    pub fn as_vec3(&self) -> Vec3 {
        self.v.as_vec3()
    }

    /// Converts this Position into a 64-bit vector.
    pub fn as_dvec3(self) -> DVec3 {
        self.v
    }
}

impl Default for Position {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl Div<Position> for Position {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        Self::from_dvec(self.v.div(rhs.v))
    }
}

impl DivAssign<Position> for Position {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.v.div_assign(rhs.v);
    }
}

impl Div<f64> for Position {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        Self::from_dvec(self.v.div(rhs))
    }
}

impl DivAssign<f64> for Position {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.v.div_assign(rhs);
    }
}

impl Div<Position> for f64 {
    type Output = Position;
    #[inline]
    fn div(self, rhs: Position) -> Position {
        Position::from_dvec(f64::div(self, rhs.v))
    }
}

impl Mul<Position> for Position {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::from_dvec(self.v.mul(rhs.v))
    }
}

impl MulAssign<Position> for Position {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.v.mul_assign(rhs.v);
    }
}

impl Mul<f64> for Position {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self::from_dvec(self.v.mul(rhs))
    }
}

impl MulAssign<f64> for Position {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.v.mul_assign(rhs);
    }
}

impl Mul<Position> for f64 {
    type Output = Position;
    #[inline]
    fn mul(self, rhs: Position) -> Position {
        Position::from_dvec(f64::mul(self, rhs.v))
    }
}

impl Add<Position> for Position {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::from_dvec(self.v.add(rhs.v))
    }
}

impl AddAssign<Position> for Position {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.v.add_assign(rhs.v);
    }
}

impl Add<f64> for Position {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f64) -> Self {
        Self::from_dvec(self.v.add(rhs))
    }
}

impl AddAssign<f64> for Position {
    #[inline]
    fn add_assign(&mut self, rhs: f64) {
        self.v.add_assign(rhs);
    }
}

impl Add<Position> for f64 {
    type Output = Position;
    #[inline]
    fn add(self, rhs: Position) -> Position {
        Position::from_dvec(f64::add(self, rhs.v))
    }
}

impl Sub<Position> for Position {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::from_dvec(self.v.sub(rhs.v))
    }
}

impl SubAssign<Position> for Position {
    #[inline]
    fn sub_assign(&mut self, rhs: Position) {
        self.v.sub_assign(rhs.v);
    }
}

impl Sub<f64> for Position {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f64) -> Self {
        Self::from_dvec(self.v.sub(rhs))
    }
}

impl SubAssign<f64> for Position {
    #[inline]
    fn sub_assign(&mut self, rhs: f64) {
        self.v.sub_assign(rhs);
    }
}

impl Sub<Position> for f64 {
    type Output = Position;
    #[inline]
    fn sub(self, rhs: Position) -> Position {
        Position::from_dvec(f64::sub(self, rhs.v))
    }
}

impl Rem<Position> for Position {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self::from_dvec(self.v.rem(rhs.v))
    }
}

impl RemAssign<Position> for Position {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.v.rem_assign(rhs.v);
    }
}

impl Rem<f64> for Position {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: f64) -> Self {
        Self::from_dvec(self.v.rem(rhs))
    }
}

impl RemAssign<f64> for Position {
    #[inline]
    fn rem_assign(&mut self, rhs: f64) {
        self.v.rem_assign(rhs);
    }
}

impl Rem<Position> for f64 {
    type Output = Position;
    #[inline]
    fn rem(self, rhs: Position) -> Position {
        Position::from_dvec(f64::rem(self, rhs.v))
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.v)
    }
}
