use crate::*;

use crate::intersect::*;
use crate::math::*;
use crate::polygon::*;

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

#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPoint(
    plane: *const Plane,
    p: *const Vec3,
) -> PointClassification {
    let _magnitude: f32 = f64::abs((1.0f32 - (*plane).n.length()) as f64) as f32;
    let dist: f32 = Vec3::dot((*plane).n, *p) - (*plane).d;
    if dist > PLANE_THICKNESS_EPSILON {
        PointClassification::InFront
    } else if dist < -PLANE_THICKNESS_EPSILON {
        PointClassification::Behind
    } else {
        PointClassification::Coplanar
    }
}

#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPolygon(
    plane: *const Plane,
    polygon: *const Polygon,
) -> PolygonClassification {
    let mut numInFront: i32 = 0;
    let mut numBehind: i32 = 0;
    for i in 0..(*polygon).vertices.len() {
        match Plane_ClassifyPoint(plane, &(*polygon).vertices[i]) {
            PointClassification::InFront => {
                numInFront += 1;
            }
            PointClassification::Behind => {
                numBehind += 1;
            }
            PointClassification::Coplanar => {}
        }

        // TODO : This early out may not make as much sense if the BSP stops cutting triangles.
        if numInFront != 0 && numBehind != 0 {
            return PolygonClassification::Straddling;
        }
    }

    if numInFront != 0 {
        PolygonClassification::InFront
    } else if numBehind != 0 {
        PolygonClassification::Behind
    } else {
        PolygonClassification::Coplanar
    }
}

#[no_mangle]
pub unsafe extern "C" fn Plane_Validate(plane: *const Plane) -> Error {
    let mut e = 0 as Error;

    e |= Float_Validate((*plane).d as f64);
    e |= Vec3_Validate((*plane).n);

    e
}

#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygon(polygon: *const Polygon, plane: *mut Plane) {
    Polygon_ToPlane(polygon, plane);
}

#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygonFast(polygon: *const Polygon, plane: *mut Plane) {
    Polygon_ToPlaneFast(polygon, plane);
}
