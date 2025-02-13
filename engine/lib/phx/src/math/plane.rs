use super::*;
use crate::error::Error;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PointClassification {
    InFront = 1,
    Behind = 2,
    Coplanar = 3,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PolygonClassification {
    InFront = 1,
    Behind = 2,
    Coplanar = 3,
    Straddling = 4,
}

#[luajit_ffi_gen::luajit_ffi(typedef = "
    float nx;
    float ny;
    float nz;
    float d;")]
impl Plane {
    pub fn classify_point(&self, p: &Vec3) -> PointClassification {
        // let _magnitude = f64::abs((1.0 - self.n.length()) as f64) as f32;
        let dist = Vec3::dot(self.n, *p) - self.d;
        if dist > PLANE_THICKNESS_EPSILON {
            PointClassification::InFront
        } else if dist < -PLANE_THICKNESS_EPSILON {
            PointClassification::Behind
        } else {
            PointClassification::Coplanar
        }
    }

    pub fn classify_polygon(&self, polygon: &Polygon) -> PolygonClassification {
        let mut num_in_front = 0;
        let mut num_behind = 0;
        for vertex in &polygon.vertices {
            match self.classify_point(vertex) {
                PointClassification::InFront => {
                    num_in_front += 1;
                }
                PointClassification::Behind => {
                    num_behind += 1;
                }
                PointClassification::Coplanar => {}
            }

            // TODO : This early out may not make as much sense if the BSP stops cutting triangles.
            if num_in_front != 0 && num_behind != 0 {
                return PolygonClassification::Straddling;
            }
        }

        if num_in_front != 0 {
            PolygonClassification::InFront
        } else if num_behind != 0 {
            PolygonClassification::Behind
        } else {
            PolygonClassification::Coplanar
        }
    }

    pub fn validate(&self) -> Error {
        let mut e = 0 as Error;
        e |= Float_Validate(self.d as f64);
        e |= Vec3_Validate(self.n);
        e
    }

    pub fn from_polygon(polygon: &Polygon) -> Self {
        polygon.to_plane()
    }

    pub fn from_polygon_fast(polygon: &Polygon) -> Self {
        polygon.to_plane_fast()
    }
}
