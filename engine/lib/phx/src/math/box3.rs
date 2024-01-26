use super::*;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[repr(C)]
pub struct Box3 {
    pub lower: Vec3,
    pub upper: Vec3,
}

impl Box3 {
    pub const fn new(lower: Vec3, upper: Vec3) -> Box3 {
        Box3 { lower, upper }
    }

    pub fn union(a: Box3, b: Box3) -> Box3 {
        Box3 {
            lower: Vec3 {
                x: f32::min(a.lower.x, b.lower.x),
                y: f32::min(a.lower.y, b.lower.y),
                z: f32::min(a.lower.z, b.lower.z),
            },
            upper: Vec3 {
                x: f32::max(a.upper.x, b.upper.x),
                y: f32::max(a.upper.y, b.upper.y),
                z: f32::max(a.upper.z, b.upper.z),
            },
        }
    }

    pub fn intersection(a: Box3, b: Box3) -> Box3 {
        Box3 {
            lower: Vec3 {
                x: f32::max(a.lower.x, b.lower.x),
                y: f32::max(a.lower.y, b.lower.y),
                z: f32::max(a.lower.z, b.lower.z),
            },
            upper: Vec3 {
                x: f32::min(a.upper.x, b.upper.x),
                y: f32::min(a.upper.y, b.upper.y),
                z: f32::min(a.upper.z, b.upper.z),
            },
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.lower.x + self.upper.x) / 2.0f32,
            (self.lower.y + self.upper.y) / 2.0f32,
            (self.lower.z + self.upper.z) / 2.0f32,
        )
    }

    pub fn add(&mut self, point: Vec3) {
        self.lower = Vec3::min(self.lower, point);
        self.upper = Vec3::max(self.upper, point);
    }

    pub fn volume(&self) -> f32 {
        (self.upper.x - self.lower.x)
            * (self.upper.y - self.lower.y)
            * (self.upper.z - self.lower.z)
    }

    pub fn half_extents(&self) -> Vec3 {
        Vec3::new(
            self.upper.x - self.lower.x,
            self.upper.y - self.lower.y,
            self.upper.z - self.lower.z,
        ) * 0.5
    }

    pub fn contains(a: Box3, b: Box3) -> bool {
        a.lower.x <= b.lower.x
            && a.upper.x >= b.upper.x
            && a.lower.y <= b.lower.y
            && a.upper.y >= b.upper.y
            && a.lower.z <= b.lower.z
            && a.upper.z >= b.upper.z
    }

    pub fn intersects_ray(&self, ro: Vec3, rdi: Vec3) -> bool {
        let mut t1: f64 = (rdi.x * (self.lower.x - ro.x)) as f64;
        let mut t2: f64 = (rdi.x * (self.upper.x - ro.x)) as f64;
        let mut tMin: f64 = f64::min(t1, t2);
        let mut tMax: f64 = f64::max(t1, t2);
        t1 = (rdi.y * (self.lower.y - ro.y)) as f64;
        t2 = (rdi.y * (self.upper.y - ro.y)) as f64;
        tMin = f64::max(tMin, f64::min(t1, t2));
        tMax = f64::min(tMax, f64::max(t1, t2));
        t1 = (rdi.z * (self.lower.z - ro.z)) as f64;
        t2 = (rdi.z * (self.upper.z - ro.z)) as f64;
        tMin = f64::max(tMin, f64::min(t1, t2));
        tMax = f64::min(tMax, f64::max(t1, t2));
        tMax >= tMin && tMax > 0.0
    }

    pub fn intersects_box(a: Box3, b: Box3) -> bool {
        if a.lower.x > b.upper.x || a.upper.x < b.lower.x {
            return false;
        }
        if a.lower.y > b.upper.y || a.upper.y < b.lower.y {
            return false;
        }
        if a.lower.z > b.upper.z || a.upper.z < b.lower.z {
            return false;
        }
        true
    }
}
