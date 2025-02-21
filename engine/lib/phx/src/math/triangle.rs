use super::*;
use crate::error::Error;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

#[luajit_ffi_gen::luajit_ffi(typedef = "Vec3f vertices[3];")]
impl Triangle {
    pub fn to_plane(&self) -> Plane {
        let v = &self.vertices;
        let e1 = v[1] - v[0];
        let e2 = v[2] - v[0];
        let n = Vec3::cross(e1, e2).normalize();
        let mut centroid = v[0];
        centroid += v[1];
        centroid += v[2];
        centroid /= 3.0;

        Plane {
            n,
            d: centroid.dot(n),
        }
    }

    pub fn to_plane_fast(&self) -> Plane {
        let v = &self.vertices;
        let e1 = v[1] - v[0];
        let e2 = v[2] - v[0];
        let n = Vec3::cross(e1, e2);

        Plane { n, d: v[0].dot(n) }
    }

    pub fn get_area(&self) -> f32 {
        let e1 = self.vertices[1] - self.vertices[0];
        let e2 = self.vertices[2] - self.vertices[1];
        0.5 * Vec3::cross(e1, e2).length()
    }

    pub fn validate(&self) -> Error {
        let v = &self.vertices;
        for vertex in v {
            let e = validate_vec3(*vertex);
            if e != 0 {
                return 0x400000 | e;
            }
        }
        let eq01 = v[0] == v[1];
        let eq12 = v[1] == v[2];
        let eq20 = v[2] == v[0];
        if eq01 || eq12 || eq20 {
            return (0x400000 | 0x40) as Error;
        }
        let e01 = v[0].distance(v[1]);
        let e12 = v[1].distance(v[2]);
        let e20 = v[2].distance(v[0]);
        let shortest: f32 = f32::min(f32::min(e01, e12), e20);
        if shortest < 0.75 * 1e-4 {
            return (0x400000 | 0x8) as Error;
        }
        0 as Error
    }
}
