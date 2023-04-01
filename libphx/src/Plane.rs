use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::*;
use crate::Polygon::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PointClassification {
    InFront  = 1,
    Behind   = 2,
    Coplanar = 3
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PolygonClassification {
    InFront    = 1,
    Behind     = 2,
    Coplanar   = 3,
    Straddling = 4
}

#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPoint(
    mut plane: *mut Plane,
    p: *const Vec3,
) -> PointClassification {
    let mut _magnitude: f32 = f64::abs((1.0f32 - (*plane).n.length()) as f64) as f32;
    let mut dist: f32 = Vec3::dot((*plane).n, *p) - (*plane).d;
    if dist as f64 > 1e-4f64 {
        PointClassification::InFront
    } else if (dist as f64) < -1e-4f64 {
        PointClassification::Behind
    } else {
        PointClassification::Coplanar
    }
}

#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPolygon(
    mut plane: *mut Plane,
    polygon: *const Polygon,
) -> PolygonClassification {
    let mut numInFront: i32 = 0;
    let mut numBehind: i32 = 0;
    let mut i: usize = 0;
    while i < (*polygon).vertices.len() {
        let mut classification: PointClassification = Plane_ClassifyPoint(plane, &(*polygon).vertices[i]);
        match classification {
            PointClassification::InFront => {
                numInFront += 1;
            }
            PointClassification::Behind => {
                numBehind += 1;
            }
            PointClassification::Coplanar => {
            },
        }
        
        // TODO : This early out may not make as much sense if the BSP stops cutting triangles.
        if numInFront != 0 && numBehind != 0 {
            return PolygonClassification::Straddling;
        }
        i += 1;
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
pub unsafe extern "C" fn Plane_Validate(mut plane: *mut Plane) -> Error {
    let mut e: Error = 0 as Error;

    e |= Float_Validate((*plane).d as f64);
    e |= Vec3_Validate((*plane).n);
    
    e
}

#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygon(mut polygon: *mut Polygon, mut plane: *mut Plane) {
    Polygon_ToPlane(polygon, plane);
}

#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygonFast(mut polygon: *mut Polygon, mut plane: *mut Plane) {
    Polygon_ToPlaneFast(polygon, plane);
}
