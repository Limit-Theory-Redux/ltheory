use glam::Vec4;
use parley::style::Brush;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        check_range(r);
        check_range(g);
        check_range(b);
        check_range(a);

        Self { r, g, b, a }
    }

    pub fn with_red(self, r: f32) -> Self {
        check_range(r);

        Self { r, ..self }
    }

    pub fn with_green(self, g: f32) -> Self {
        check_range(g);

        Self { g, ..self }
    }

    pub fn with_blue(self, b: f32) -> Self {
        check_range(b);

        Self { b, ..self }
    }

    pub fn with_alpha(self, a: f32) -> Self {
        check_range(a);

        Self { a, ..self }
    }

    pub fn is_transparent(&self) -> bool {
        self.a <= 0.0
    }

    pub fn is_opaque(&self) -> bool {
        self.a > 0.0
    }

    /// Convert HSL color to RGB
    pub fn from_hsl(h: f32, s: f32, l: f32, a: f32) -> Self {
        if s == 0.0 {
            // Achromatic, i.e., grey.
            return Self {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a,
            };
        }

        let h = h / 360.0; // treat this as 0..1 instead of degrees

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - (l * s)
        };
        let p = 2.0 * l - q;

        Self::new(
            Self::hue_to_rgb(p, q, h + 1.0 / 3.0),
            Self::hue_to_rgb(p, q, h),
            Self::hue_to_rgb(p, q, h - 1.0 / 3.0),
            a,
        )
    }

    /// Blends two colors proportionally to their alpha parameters
    pub fn blend(&self, other: &Color) -> Color {
        let t1 = self.a / (self.a + other.a);
        let t2 = other.a / (self.a + other.a);

        let r = self.r * t1 + other.r * t2;
        let g = self.g * t1 + other.g * t2;
        let b = self.b * t1 + other.b * t2;
        let a = self.a * t1 + other.a * t2;

        Color { r, g, b, a }
    }

    /// Blends current color with the given one
    /// proportionally to their alpha parameters
    pub fn blend_with(&mut self, other: &Color) {
        *self = self.blend(other);
    }

    fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
        // Normalize
        let t = if t < 0.0 {
            t + 1.0
        } else if t > 1.0 {
            t - 1.0
        } else {
            t
        };

        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0 / 2.0 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Brush for Color {}

impl From<[f32; 4]> for Color {
    #[inline]
    fn from(a: [f32; 4]) -> Self {
        Self::new(a[0], a[1], a[2], a[3])
    }
}

impl From<Vec4> for Color {
    fn from(v: Vec4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

impl From<Color> for Vec4 {
    fn from(v: Color) -> Self {
        Self::new(v.r, v.g, v.b, v.a)
    }
}

#[inline]
fn check_range(v: f32){
    assert!((0.0..=1.0).contains(&v));
}
