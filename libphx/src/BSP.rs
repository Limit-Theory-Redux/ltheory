use crate::internal::Memory::*;
use crate::Common::*;
use crate::Common::*;
use crate::Draw::*;
use crate::Intersect::*;
use crate::LineSegment::*;
use crate::Math::Sphere;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Mesh::*;
use crate::Plane::*;
use crate::Polygon::*;
use crate::Ray::*;
use crate::RenderState::*;
use crate::Triangle::*;
use crate::RNG::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSP {
    pub rootNode: BSPNodeRef,
    pub emptyLeaf: BSPNodeRef,
    pub nodes_size: i32,
    pub nodes_capacity: i32,
    pub nodes_data: *mut BSPNode,
    pub triangles_size: i32,
    pub triangles_capacity: i32,
    pub triangles_data: *mut Triangle,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPNode {
    pub plane: Plane,
    pub child: [BSPNodeRef; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPNodeRef {
    pub index: i32,
    pub triangleCount: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IntersectSphereProfiling {
    pub nodes: i32,
    pub leaves: i32,
    pub triangles: i32,
    pub triangleTests_size: i32,
    pub triangleTests_capacity: i32,
    pub triangleTests_data: *mut TriangleTest,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TriangleTest {
    pub triangle: *mut Triangle,
    pub hit: bool,
}

pub type BSPNodeRel = u8;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPBuild {
    pub rootNode: *mut BSPBuild_Node,
    pub rng: *mut RNG,
    pub nodeCount: i32,
    pub leafCount: i32,
    pub triangleCount: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPBuild_Node {
    pub plane: Plane,
    pub child: [*mut BSPBuild_Node; 2],
    pub polygons_size: i32,
    pub polygons_capacity: i32,
    pub polygons_data: *mut PolygonEx,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PolygonEx {
    pub vertices_size: i32,
    pub vertices_capacity: i32,
    pub vertices_data: *mut Vec3,
    pub flags: PolygonFlag,
}
pub type PolygonFlag = u8;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPBuild_NodeData {
    pub polygons_size: i32,
    pub polygons_capacity: i32,
    pub polygons_data: *mut PolygonEx,
    pub validPolygonCount: i32,
    pub triangleCount: i32,
    pub depth: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DelayRay {
    pub nodeRef: BSPNodeRef,
    pub tMin: f32,
    pub tMax: f32,
    pub depth: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Delay {
    pub nodeRef: BSPNodeRef,
    pub depth: i32,
}

#[inline]
unsafe extern "C" fn Lerp(mut a: f64, mut b: f64, mut t: f64) -> f64 {
    a + t * (b - a)
}

#[no_mangle]
pub static BSPNodeRel_Parent: BSPNodeRel = 0_i32 as BSPNodeRel;

#[no_mangle]
pub static BSPNodeRel_Back: BSPNodeRel = 1_i32 as BSPNodeRel;

#[no_mangle]
pub static BSPNodeRel_Front: BSPNodeRel = 2_i32 as BSPNodeRel;

static mut BackIndex: i32 = 0_i32;

static mut FrontIndex: i32 = 1_i32;

static mut RootNodeIndex: i32 = 1_i32;

static mut EmptyLeafIndex: i32 = 1_i32;

#[no_mangle]
pub static mut rayStack_size: i32 = 0;

#[no_mangle]
pub static mut rayStack_capacity: i32 = 0;

#[no_mangle]
pub static mut rayStack_data: *mut DelayRay = std::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn BSP_IntersectRay(
    mut this: *mut BSP,
    mut _ray: *const Ray,
    mut tHit: *mut f32,
) -> bool {
    let mut ray: Ray = *_ray;
    *tHit = 3.40282347e+38f32;
    let mut nodeRef: BSPNodeRef = (*this).rootNode;
    let mut tEpsilon: f32 = (8.0f32 as f64 * 1e-4f64 / ray.dir.length() as f64) as f32;
    let mut hit: bool = false;
    let mut depth: i32 = 0_i32;
    let mut maxDepth: i32 = 0_i32;
    loop {
        maxDepth = f64::max(depth as f64, maxDepth as f64) as i32;
        if nodeRef.index >= 0_i32 {
            let mut node: *mut BSPNode = ((*this).nodes_data).offset(nodeRef.index as isize);
            let mut dist: f32 = Vec3::dot((*node).plane.n, ray.p) - (*node).plane.d;
            let mut denom: f32 = -Vec3::dot((*node).plane.n, ray.dir);
            let mut nearIndex: i32 = (dist > 0.0f32) as i32;
            let mut earlyIndex: i32 = nearIndex;
            if denom != 0.0f32 {
                let mut t: f32 = dist / denom;
                let mut planeBegin: f32 = t - tEpsilon;
                let mut planeEnd: f32 = t + tEpsilon;
                if !(planeBegin >= ray.tMax) {
                    if planeEnd <= ray.tMin {
                        earlyIndex = (t >= 0.0f32) as i32 ^ nearIndex;
                    } else {
                        earlyIndex = (t < 0.0f32) as i32 ^ nearIndex;
                        let mut min: f32 = f64::max(planeBegin as f64, ray.tMin as f64) as f32;
                        let mut max: f32 = f64::min(planeEnd as f64, ray.tMax as f64) as f32;
                        let mut d: DelayRay = DelayRay {
                            nodeRef: (*node).child[(1_i32 ^ earlyIndex) as usize],
                            tMin: min,
                            tMax: ray.tMax,
                            depth: depth,
                        };
                        if (rayStack_capacity == rayStack_size) as i32 as libc::c_long != 0 {
                            rayStack_capacity = if rayStack_capacity != 0 {
                                rayStack_capacity * 2_i32
                            } else {
                                1_i32
                            };
                            let mut elemSize: usize = ::core::mem::size_of::<DelayRay>();
                            let mut pData: *mut *mut libc::c_void =
                                &mut rayStack_data as *mut *mut DelayRay as *mut *mut libc::c_void;
                            *pData = MemRealloc(
                                rayStack_data as *mut libc::c_void,
                                (rayStack_capacity as usize).wrapping_mul(elemSize),
                            );
                        }
                        let fresh0 = rayStack_size;
                        rayStack_size += 1;
                        *rayStack_data.offset(fresh0 as isize) = d;
                        ray.tMax = max;
                    }
                }
            } else if f64::abs(dist as f64) < 8.0f32 as f64 * 1e-4f64 {
                earlyIndex = nearIndex;
                let mut d_0: DelayRay = DelayRay {
                    nodeRef: (*node).child[(1_i32 ^ earlyIndex) as usize],
                    tMin: ray.tMin,
                    tMax: ray.tMax,
                    depth: depth,
                };
                if (rayStack_capacity == rayStack_size) as libc::c_long != 0 {
                    rayStack_capacity = if rayStack_capacity != 0 {
                        rayStack_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize_0: usize = ::core::mem::size_of::<DelayRay>();
                    let mut pData_0: *mut *mut libc::c_void =
                        &mut rayStack_data as *mut *mut DelayRay as *mut *mut libc::c_void;
                    *pData_0 = MemRealloc(
                        rayStack_data as *mut libc::c_void,
                        (rayStack_capacity as usize).wrapping_mul(elemSize_0),
                    );
                }
                let fresh1 = rayStack_size;
                rayStack_size += 1;
                *rayStack_data.offset(fresh1 as isize) = d_0;
            }
            depth += 1;
            nodeRef = (*node).child[earlyIndex as usize];
        } else {
            let mut leaf: *const Triangle =
                ((*this).triangles_data).offset(-nodeRef.index as isize);
            let mut i: u8 = 0_i32 as u8;
            while (i as i32) < nodeRef.triangleCount as i32 {
                let mut triangle: *const Triangle = leaf.offset(i as i32 as isize);
                let mut t_0: f32 = 0.;
                if Intersect_RayTriangle_Moller1(&mut ray, triangle, &mut t_0) {
                    if !hit || t_0 < *tHit {
                        hit = true;
                        *tHit = t_0;
                    }
                }
                i = i.wrapping_add(1);
            }
            if hit {
                break;
            }
            if rayStack_size == 0_i32 {
                break;
            }
            rayStack_size -= 1;
            let mut d_1: DelayRay = *rayStack_data.offset(rayStack_size as isize);
            nodeRef = d_1.nodeRef;
            ray.tMin = d_1.tMin;
            ray.tMax = d_1.tMax;
            depth = d_1.depth;
        }
    }
    rayStack_size = 0_i32;
    hit
}

#[no_mangle]
pub unsafe extern "C" fn BSP_IntersectLineSegment(
    mut this: *mut BSP,
    mut lineSegment: *const LineSegment,
    mut pHit: *mut Vec3,
) -> bool {
    let mut t: f32 = 0.;
    let mut dir: Vec3 = (*lineSegment).p1 - (*lineSegment).p0;
    let mut ray: Ray = Ray {
        p: (*lineSegment).p0,
        dir: dir,
        tMin: 0.0f32,
        tMax: 1.0f32,
    };
    if BSP_IntersectRay(this, &mut ray, &mut t) {
        Ray_GetPoint(&mut ray, t, pHit);
        return true;
    }
    false
}

#[no_mangle]
pub static mut nodeStack_size: i32 = 0;

#[no_mangle]
pub static mut nodeStack_data: *mut Delay = std::ptr::null_mut();

#[no_mangle]
pub static mut nodeStack_capacity: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn BSP_IntersectSphere(
    mut this: *mut BSP,
    mut sphere: *const Sphere,
    mut pHit: *mut Vec3,
) -> bool {
    let mut nodeRef: BSPNodeRef = (*this).rootNode;
    let mut hit: bool = false;
    let mut depth: i32 = 0_i32;
    let mut maxDepth: i32 = 0_i32;
    loop {
        maxDepth = f64::max(depth as f64, maxDepth as f64) as i32;
        if nodeRef.index >= 0_i32 {
            let mut node: *mut BSPNode = ((*this).nodes_data).offset(nodeRef.index as isize);
            let mut dist: f32 = Vec3::dot((*node).plane.n, (*sphere).p) - (*node).plane.d;
            if dist as f64 > (*sphere).r as f64 + 2.0f32 as f64 * 1e-4f64 {
                nodeRef = (*node).child[FrontIndex as usize];
            } else if (dist as f64) < -((*sphere).r as f64 + 2.0f32 as f64 * 1e-4f64) {
                nodeRef = (*node).child[BackIndex as usize];
            } else {
                let mut d: Delay = Delay {
                    nodeRef: (*node).child[BackIndex as usize],
                    depth: depth,
                };
                if (nodeStack_capacity == nodeStack_size) as libc::c_long != 0 {
                    nodeStack_capacity = if nodeStack_capacity != 0 {
                        nodeStack_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize: usize = ::core::mem::size_of::<Delay>();
                    let mut pData: *mut *mut libc::c_void =
                        &mut nodeStack_data as *mut *mut Delay as *mut *mut libc::c_void;
                    *pData = MemRealloc(
                        nodeStack_data as *mut libc::c_void,
                        (nodeStack_capacity as usize).wrapping_mul(elemSize),
                    );
                }
                let fresh2 = nodeStack_size;
                nodeStack_size += 1;
                *nodeStack_data.offset(fresh2 as isize) = d;
                nodeRef = (*node).child[FrontIndex as usize];
            }
            depth += 1;
        } else {
            let mut leaf: *mut Triangle = ((*this).triangles_data).offset(-nodeRef.index as isize);
            let mut i: u8 = 0_i32 as u8;
            while (i as i32) < nodeRef.triangleCount as i32 {
                let mut triangle: *mut Triangle = leaf.offset(i as i32 as isize);
                let mut pHit2 = Vec3::ZERO;
                if Intersect_SphereTriangle(sphere, triangle, &mut pHit2) {
                    hit = true;
                    *pHit = pHit2;
                    break;
                } else {
                    i = i.wrapping_add(1);
                }
            }
            if hit {
                break;
            }
            if nodeStack_size == 0_i32 {
                break;
            }
            nodeStack_size -= 1;
            let mut d_0: Delay = *nodeStack_data.offset(nodeStack_size as isize);
            nodeRef = d_0.nodeRef;
            depth = d_0.depth;
        }
    }
    nodeStack_size = 0_i32;
    hit
}

#[no_mangle]
pub static PolygonFlag_None: PolygonFlag = (0_i32 << 0_i32) as PolygonFlag;

#[no_mangle]
pub static PolygonFlag_InvalidFaceSplit: PolygonFlag = (1_i32 << 0_i32) as PolygonFlag;

#[no_mangle]
pub static PolygonFlag_InvalidDecompose: PolygonFlag = (1_i32 << 1_i32) as PolygonFlag;

#[no_mangle]
pub static PolygonFlag_InvalidEdgeSplit: PolygonFlag = (1_i32 << 2_i32) as PolygonFlag;

unsafe extern "C" fn BSPBuild_ScoreSplitPlane(
    mut nodeData: *mut BSPBuild_NodeData,
    mut plane: Plane,
    mut k: f32,
) -> f32 {
    let mut numInFront: i32 = 0_i32;
    let mut numBehind: i32 = 0_i32;
    let mut numStraddling: i32 = 0_i32;
    let mut polygon: *mut PolygonEx = (*nodeData).polygons_data;
    let mut __iterend: *mut PolygonEx =
        ((*nodeData).polygons_data).offset((*nodeData).polygons_size as isize);
    while polygon < __iterend {
        let mut classification: PolygonClassification =
            Plane_ClassifyPolygon(&mut plane, polygon as *mut Polygon);
        let mut current_block_4: u64;
        match classification as i32 {
            3 | 2 => {
                current_block_4 = 11316911015026613471;
            }
            1 => {
                numInFront += 1;
                current_block_4 = 11812396948646013369;
            }
            4 => {
                numStraddling += 1;
                current_block_4 = 11812396948646013369;
            }
            _ => Fatal(
                b"BSPBuild_ScoreSplitPlane: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                classification as i32,
            ),
        }
        match current_block_4 {
            11316911015026613471 => {
                numBehind += 1;
            }
            _ => {}
        }
        polygon = polygon.offset(1);
    }
    let mut score: f32 = Lerp(
        f64::abs((numInFront - numBehind) as f64) as f32 as f64,
        numStraddling as f32 as f64,
        k as f64,
    ) as f32;
    score
}

unsafe extern "C" fn BSPBuild_ChooseSplitPlane(
    mut bsp: *mut BSPBuild,
    mut nodeData: *mut BSPBuild_NodeData,
    mut splitPlane: *mut Plane,
) -> bool {
    let mut maxDepth: f32 = 1000.0f32;
    let mut biasedDepth: f32 = (*nodeData).depth as f32 - 100.0f32;
    let mut t: f32 = f64::max((biasedDepth / maxDepth) as f64, 0.0f32 as f64) as f32;
    let mut k: f32 = Lerp(0.85f32 as f64, 0.25f32 as f64, t as f64) as f32;
    let mut bestScore: f32 = 3.40282347e+38f32;
    let mut bestPlane: Plane = Plane {
        n: Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        d: 0.,
    };
    let mut bestPolygon: *mut PolygonEx = std::ptr::null_mut();
    let mut numToCheck: i32 = 10_i32;
    let mut polygonsLen: i32 = (*nodeData).polygons_size;
    if (*nodeData).validPolygonCount > 0_i32 {
        numToCheck = f64::min(numToCheck as f64, (*nodeData).validPolygonCount as f64) as i32;
        let mut i: i32 = 0_i32;
        while i < numToCheck {
            let mut polygonIndex: i32 =
                (RNG_Get32((*bsp).rng)).wrapping_rem(polygonsLen as u32) as i32;
            let mut j: i32 = 0_i32;
            while j < polygonsLen {
                let mut polygon: *mut PolygonEx =
                    ((*nodeData).polygons_data).offset(polygonIndex as isize);
                if (*polygon).flags as i32 & PolygonFlag_InvalidFaceSplit as i32 == 0 {
                    let mut plane: Plane = Plane {
                        n: Vec3 {
                            x: 0.,
                            y: 0.,
                            z: 0.,
                        },
                        d: 0.,
                    };
                    Polygon_ToPlane(polygon as *mut Polygon, &mut plane);
                    let mut score: f32 = BSPBuild_ScoreSplitPlane(nodeData, plane, k);
                    if score < bestScore {
                        bestScore = score;
                        bestPlane = plane;
                        bestPolygon = polygon;
                    }
                    break;
                } else {
                    polygonIndex = (polygonIndex + 1_i32) % polygonsLen;
                    j += 1;
                }
            }
            i += 1;
        }
        if !bestPolygon.is_null() {
            (*bestPolygon).flags =
                ((*bestPolygon).flags as i32 | PolygonFlag_InvalidFaceSplit as i32) as PolygonFlag;
        }
    } else if polygonsLen > 0_i32 {
        let mut splitFound: bool = false;
        if !splitFound {
            let mut polygonIndex_0: i32 =
                (RNG_Get32((*bsp).rng)).wrapping_rem(polygonsLen as u32) as i32;
            let mut i_0: i32 = 0_i32;
            while i_0 < polygonsLen {
                let mut polygon_0: *mut PolygonEx =
                    ((*nodeData).polygons_data).offset(polygonIndex_0 as isize);
                if !((*polygon_0).flags as i32 & PolygonFlag_InvalidDecompose as i32 != 0) {
                    let mut v: *mut Vec3 = (*polygon_0).vertices_data;
                    let mut vLen: i32 = (*polygon_0).vertices_size;
                    let mut j_0: i32 = 2_i32;
                    while j_0 < vLen - 1_i32 {
                        let mut edge: Vec3 = *v.offset(0) - *v.offset(j_0 as isize);
                        let mut mid: Vec3 =
                            Vec3::lerp(*v.offset(0), *v.offset(j_0 as isize), 0.5f32);
                        let mut polygonPlane: Plane = Plane {
                            n: Vec3 {
                                x: 0.,
                                y: 0.,
                                z: 0.,
                            },
                            d: 0.,
                        };
                        Polygon_ToPlane(polygon_0 as *mut Polygon, &mut polygonPlane);
                        let mut plane_0: Plane = Plane {
                            n: Vec3 {
                                x: 0.,
                                y: 0.,
                                z: 0.,
                            },
                            d: 0.,
                        };
                        plane_0.n = Vec3::cross(edge, polygonPlane.n).normalize();
                        plane_0.d = Vec3::dot(plane_0.n, mid);
                        if Plane_ClassifyPolygon(&mut plane_0, polygon_0 as *mut Polygon) as i32
                            == 4_i32
                        {
                            splitFound = true;
                            bestScore = 0.0f32;
                            bestPlane = plane_0;
                            bestPolygon = polygon_0;
                            break;
                        } else {
                            (*polygon_0).flags = ((*polygon_0).flags as i32
                                | PolygonFlag_InvalidDecompose as i32)
                                as PolygonFlag;
                            j_0 += 1;
                        }
                    }
                    if splitFound {
                        break;
                    }
                    polygonIndex_0 = (polygonIndex_0 + 1_i32) % polygonsLen;
                }
                i_0 += 1;
            }
            if splitFound {
                (*bestPolygon).flags = ((*bestPolygon).flags as i32
                    | PolygonFlag_InvalidDecompose as i32)
                    as PolygonFlag;
            }
        }
        if !splitFound {
            let mut polygonIndex_1: i32 =
                (RNG_Get32((*bsp).rng)).wrapping_rem(polygonsLen as u32) as i32;
            let mut i_1: i32 = 0_i32;
            while i_1 < polygonsLen {
                let mut polygon_1: *mut PolygonEx =
                    ((*nodeData).polygons_data).offset(polygonIndex_1 as isize);
                if !((*polygon_1).flags as i32 & PolygonFlag_InvalidEdgeSplit as i32 != 0) {
                    let mut polygonPlane_0: Plane = Plane {
                        n: Vec3 {
                            x: 0.,
                            y: 0.,
                            z: 0.,
                        },
                        d: 0.,
                    };
                    Polygon_ToPlane(polygon_1 as *mut Polygon, &mut polygonPlane_0);
                    let mut v_0: *mut Vec3 = (*polygon_1).vertices_data;
                    let mut vLen_0: i32 = (*polygon_1).vertices_size;
                    let mut vPrev: Vec3 = *v_0.offset((vLen_0 - 1_i32) as isize);
                    let mut j_1: i32 = 0_i32;
                    while j_1 < vLen_0 {
                        let mut vCur: Vec3 = *v_0.offset(j_1 as isize);
                        let mut edge_0: Vec3 = vCur - vPrev;
                        let mut mid_0: Vec3 = Vec3::lerp(vPrev, vCur, 0.5f32);
                        let mut plane_1: Plane = Plane {
                            n: Vec3 {
                                x: 0.,
                                y: 0.,
                                z: 0.,
                            },
                            d: 0.,
                        };
                        plane_1.n = Vec3::cross(edge_0, polygonPlane_0.n).normalize();
                        plane_1.d = Vec3::dot(plane_1.n, mid_0);
                        let mut score_0: f32 = BSPBuild_ScoreSplitPlane(nodeData, plane_1, 0.0f32);
                        if score_0 < bestScore {
                            splitFound = true;
                            bestPolygon = polygon_1;
                            bestScore = score_0;
                            bestPlane = plane_1;
                        }
                        vPrev = vCur;
                        numToCheck -= 1;
                        if numToCheck == 0_i32 {
                            break;
                        }
                        j_1 += 1;
                    }
                    if numToCheck == 0_i32 {
                        break;
                    }
                    polygonIndex_1 = (polygonIndex_1 + 1_i32) % polygonsLen;
                }
                i_1 += 1;
            }
            if splitFound {
                (*bestPolygon).flags = ((*bestPolygon).flags as i32
                    | PolygonFlag_InvalidEdgeSplit as i32)
                    as PolygonFlag;
            }
        }
    }
    if bestScore < 3.40282347e+38f32 {
        *splitPlane = bestPlane;
        true
    } else {
        false
    }
}

#[inline]
unsafe extern "C" fn BSPBuild_AppendPolygon(
    mut nodeData: *mut BSPBuild_NodeData,
    mut polygon: *mut PolygonEx,
) {
    (*nodeData).triangleCount += (*polygon).vertices_size - 2_i32;
    (*nodeData).validPolygonCount +=
        ((*polygon).flags as i32 & PolygonFlag_InvalidFaceSplit as i32 == 0) as i32;
    if ((*nodeData).polygons_capacity == (*nodeData).polygons_size) as i32 as libc::c_long != 0 {
        (*nodeData).polygons_capacity = if (*nodeData).polygons_capacity != 0 {
            (*nodeData).polygons_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = ::core::mem::size_of::<PolygonEx>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*nodeData).polygons_data as *mut *mut PolygonEx as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*nodeData).polygons_data as *mut libc::c_void,
            ((*nodeData).polygons_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh3 = (*nodeData).polygons_size;
    (*nodeData).polygons_size += 1;
    *((*nodeData).polygons_data).offset(fresh3 as isize) = *polygon;
}

unsafe extern "C" fn BSPBuild_CreateNode(
    mut bsp: *mut BSPBuild,
    mut nodeData: *mut BSPBuild_NodeData,
) -> *mut BSPBuild_Node {
    let mut node: *mut BSPBuild_Node =
        MemAllocZero(::core::mem::size_of::<BSPBuild_Node>()) as *mut BSPBuild_Node;
    let mut splitPlane: Plane = Plane {
        n: Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        d: 0.,
    };
    let mut makeLeaf: bool = false;
    makeLeaf = makeLeaf as i32 != 0 || (*nodeData).triangleCount <= 12_i32;
    makeLeaf = makeLeaf as i32 != 0 || !BSPBuild_ChooseSplitPlane(bsp, nodeData, &mut splitPlane);
    if makeLeaf {
        if (*nodeData).triangleCount != 0_i32 {
            (*bsp).leafCount += 1;
        }
        (*bsp).triangleCount += (*nodeData).triangleCount;
        (*node).polygons_capacity = (*nodeData).polygons_capacity;
        (*node).polygons_size = (*nodeData).polygons_size;
        (*node).polygons_data = (*nodeData).polygons_data;
        return node;
    }
    (*bsp).nodeCount += 1;
    let mut polygonsLen: i32 = (*nodeData).polygons_size;
    let mut backNodeData: BSPBuild_NodeData = BSPBuild_NodeData {
        polygons_size: 0,
        polygons_capacity: 0,
        polygons_data: std::ptr::null_mut(),
        validPolygonCount: 0,
        triangleCount: 0,
        depth: 0,
    };
    if (backNodeData.polygons_capacity < polygonsLen) as libc::c_long != 0 {
        backNodeData.polygons_capacity = polygonsLen;
        let mut elemSize: usize = ::core::mem::size_of::<PolygonEx>();
        let mut pData: *mut *mut libc::c_void =
            &mut backNodeData.polygons_data as *mut *mut PolygonEx as *mut *mut libc::c_void;
        *pData = MemRealloc(
            backNodeData.polygons_data as *mut libc::c_void,
            (backNodeData.polygons_capacity as usize).wrapping_mul(elemSize),
        );
    }
    backNodeData.depth = ((*nodeData).depth as i32 + 1_i32) as u16;
    let mut frontNodeData: BSPBuild_NodeData = BSPBuild_NodeData {
        polygons_size: 0,
        polygons_capacity: 0,
        polygons_data: std::ptr::null_mut(),
        validPolygonCount: 0,
        triangleCount: 0,
        depth: 0,
    };
    if (frontNodeData.polygons_capacity < polygonsLen) as libc::c_long != 0 {
        frontNodeData.polygons_capacity = polygonsLen;
        let mut elemSize_0: usize = ::core::mem::size_of::<PolygonEx>();
        let mut pData_0: *mut *mut libc::c_void =
            &mut frontNodeData.polygons_data as *mut *mut PolygonEx as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            frontNodeData.polygons_data as *mut libc::c_void,
            (frontNodeData.polygons_capacity as usize).wrapping_mul(elemSize_0),
        );
    }
    frontNodeData.depth = ((*nodeData).depth as i32 + 1_i32) as u16;
    let mut polygon: *mut PolygonEx = (*nodeData).polygons_data;
    let mut __iterend: *mut PolygonEx =
        ((*nodeData).polygons_data).offset((*nodeData).polygons_size as isize);
    while polygon < __iterend {
        let mut classification: PolygonClassification =
            Plane_ClassifyPolygon(&mut splitPlane, polygon as *mut Polygon);
        let mut current_block_37: u64;
        match classification as i32 {
            3 => {
                current_block_37 = 18363606670811337990;
            }
            2 => {
                current_block_37 = 1190587995684967772;
            }
            1 => {
                BSPBuild_AppendPolygon(&mut frontNodeData, polygon);
                current_block_37 = 17184638872671510253;
            }
            4 => {
                let mut backPart: PolygonEx = PolygonEx {
                    vertices_size: 0,
                    vertices_capacity: 0,
                    vertices_data: std::ptr::null_mut(),
                    flags: 0,
                };
                backPart.flags = (*polygon).flags;
                let mut frontPart: PolygonEx = PolygonEx {
                    vertices_size: 0,
                    vertices_capacity: 0,
                    vertices_data: std::ptr::null_mut(),
                    flags: 0,
                };
                frontPart.flags = (*polygon).flags;
                Polygon_SplitSafe(
                    polygon as *mut Polygon,
                    splitPlane,
                    &mut backPart as *mut PolygonEx as *mut Polygon,
                    &mut frontPart as *mut PolygonEx as *mut Polygon,
                );
                BSPBuild_AppendPolygon(&mut backNodeData, &mut backPart);
                BSPBuild_AppendPolygon(&mut frontNodeData, &mut frontPart);
                MemFree((*polygon).vertices_data as *const libc::c_void);
                current_block_37 = 17184638872671510253;
            }
            _ => Fatal(
                b"BSPBuild_CreateNode: Unhandled case: %i\0" as *const u8 as *const libc::c_char,
                classification as i32,
            ),
        }
        match current_block_37 {
            18363606670811337990 => {
                (*polygon).flags =
                    ((*polygon).flags as i32 | PolygonFlag_InvalidFaceSplit as i32) as PolygonFlag;
                current_block_37 = 1190587995684967772;
            }
            _ => {}
        }
        match current_block_37 {
            1190587995684967772 => {
                BSPBuild_AppendPolygon(&mut backNodeData, polygon);
            }
            _ => {}
        }
        polygon = polygon.offset(1);
    }
    MemFree((*nodeData).polygons_data as *const libc::c_void);
    (*node).plane = splitPlane;
    (*node).child[BackIndex as usize] = BSPBuild_CreateNode(bsp, &mut backNodeData);
    (*node).child[FrontIndex as usize] = BSPBuild_CreateNode(bsp, &mut frontNodeData);
    node
}

unsafe extern "C" fn BSPBuild_OptimizeTree(
    mut this: *mut BSP,
    mut buildNode: *mut BSPBuild_Node,
) -> BSPNodeRef {
    if !((*buildNode).child[BackIndex as usize]).is_null()
        || !((*buildNode).child[FrontIndex as usize]).is_null()
    {
        let mut dummy: BSPNode = BSPNode {
            plane: Plane {
                n: Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
                d: 0.,
            },
            child: [BSPNodeRef {
                index: 0,
                triangleCount: 0,
            }; 2],
        };
        let mut nodeIndex: i32 = (*this).nodes_size;
        if ((*this).nodes_capacity == (*this).nodes_size) as i32 as libc::c_long != 0 {
            (*this).nodes_capacity = if (*this).nodes_capacity != 0 {
                (*this).nodes_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize: usize = ::core::mem::size_of::<BSPNode>();
            let mut pData: *mut *mut libc::c_void =
                &mut (*this).nodes_data as *mut *mut BSPNode as *mut *mut libc::c_void;
            *pData = MemRealloc(
                (*this).nodes_data as *mut libc::c_void,
                ((*this).nodes_capacity as usize).wrapping_mul(elemSize),
            );
        }
        let fresh4 = (*this).nodes_size;
        (*this).nodes_size += 1;
        *((*this).nodes_data).offset(fresh4 as isize) = dummy;
        let mut node: *mut BSPNode = ((*this).nodes_data)
            .offset((*this).nodes_size as isize)
            .offset(-(1));
        (*node).plane = (*buildNode).plane;
        (*node).child[BackIndex as usize] =
            BSPBuild_OptimizeTree(this, (*buildNode).child[BackIndex as usize]);
        (*node).child[FrontIndex as usize] =
            BSPBuild_OptimizeTree(this, (*buildNode).child[FrontIndex as usize]);
        let mut result: BSPNodeRef = BSPNodeRef {
            index: nodeIndex,
            triangleCount: 0_i32 as u8,
        };
        result
    } else {
        if (*buildNode).polygons_size == 0_i32 {
            return (*this).emptyLeaf;
        }
        let mut leafIndex: i32 = (*this).triangles_size;
        let mut polygon: *mut PolygonEx = (*buildNode).polygons_data;
        let mut __iterend: *mut PolygonEx =
            ((*buildNode).polygons_data).offset((*buildNode).polygons_size as isize);
        while polygon < __iterend {
            Polygon_ConvexToTriangles(
                polygon as *mut Polygon,
                &mut (*this).triangles_capacity,
                &mut (*this).triangles_size,
                &mut (*this).triangles_data,
            );
            polygon = polygon.offset(1);
        }
        let mut leafLen: u8 = ((*this).triangles_size - leafIndex) as u8;
        let mut result_0: BSPNodeRef = BSPNodeRef {
            index: -leafIndex,
            triangleCount: leafLen,
        };
        result_0
    }
}

unsafe extern "C" fn BSPBuild_FreeNode(mut node: *mut BSPBuild_Node) {
    if !((*node).child[BackIndex as usize]).is_null()
        || !((*node).child[FrontIndex as usize]).is_null()
    {
        BSPBuild_FreeNode((*node).child[BackIndex as usize]);
        BSPBuild_FreeNode((*node).child[FrontIndex as usize]);
    } else {
        let mut polygon: *mut PolygonEx = (*node).polygons_data;
        let mut __iterend: *mut PolygonEx =
            ((*node).polygons_data).offset((*node).polygons_size as isize);
        while polygon < __iterend {
            MemFree((*polygon).vertices_data as *const libc::c_void);
            polygon = polygon.offset(1);
        }
        MemFree((*node).polygons_data as *const libc::c_void);
    }
    MemFree(node as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn BSP_Create(mut mesh: *mut Mesh) -> *mut BSP {
    let mut this: *mut BSP = MemAllocZero(::core::mem::size_of::<BSP>()) as *mut BSP;
    let mut indexLen: i32 = Mesh_GetIndexCount(mesh);
    let mut indexData: *mut i32 = Mesh_GetIndexData(mesh);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(mesh);
    let mut nodeData: BSPBuild_NodeData = BSPBuild_NodeData {
        polygons_size: 0,
        polygons_capacity: 0,
        polygons_data: std::ptr::null_mut(),
        validPolygonCount: 0,
        triangleCount: 0,
        depth: 0,
    };
    nodeData.triangleCount = indexLen / 3_i32;
    nodeData.validPolygonCount = indexLen / 3_i32;
    if (nodeData.polygons_capacity < nodeData.triangleCount) as i32 as libc::c_long != 0 {
        nodeData.polygons_capacity = nodeData.triangleCount;
        let mut elemSize: usize = ::core::mem::size_of::<PolygonEx>();
        let mut pData: *mut *mut libc::c_void =
            &mut nodeData.polygons_data as *mut *mut PolygonEx as *mut *mut libc::c_void;
        *pData = MemRealloc(
            nodeData.polygons_data as *mut libc::c_void,
            (nodeData.polygons_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let mut i: i32 = 0_i32;
    while i < indexLen {
        let mut i0: i32 = *indexData.offset((i + 0_i32) as isize);
        let mut i1: i32 = *indexData.offset((i + 1_i32) as isize);
        let mut i2: i32 = *indexData.offset((i + 2_i32) as isize);
        let mut v0: Vec3 = (*vertexData.offset(i0 as isize)).p;
        let mut v1: Vec3 = (*vertexData.offset(i1 as isize)).p;
        let mut v2: Vec3 = (*vertexData.offset(i2 as isize)).p;
        let mut polygon: PolygonEx = PolygonEx {
            vertices_size: 0,
            vertices_capacity: 0,
            vertices_data: std::ptr::null_mut(),
            flags: 0,
        };
        if (polygon.vertices_capacity < 3_i32) as libc::c_long != 0 {
            polygon.vertices_capacity = 3_i32;
            let mut elemSize_0: usize = ::core::mem::size_of::<Vec3>();
            let mut pData_0: *mut *mut libc::c_void =
                &mut polygon.vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
            *pData_0 = MemRealloc(
                polygon.vertices_data as *mut libc::c_void,
                (polygon.vertices_capacity as usize).wrapping_mul(elemSize_0),
            );
        }
        if (polygon.vertices_capacity == polygon.vertices_size) as i32 as libc::c_long != 0 {
            polygon.vertices_capacity = if polygon.vertices_capacity != 0 {
                polygon.vertices_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize_1: usize = ::core::mem::size_of::<Vec3>();
            let mut pData_1: *mut *mut libc::c_void =
                &mut polygon.vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
            *pData_1 = MemRealloc(
                polygon.vertices_data as *mut libc::c_void,
                (polygon.vertices_capacity as usize).wrapping_mul(elemSize_1),
            );
        }
        let fresh5 = polygon.vertices_size;
        polygon.vertices_size += 1;
        *(polygon.vertices_data).offset(fresh5 as isize) = v0;
        if (polygon.vertices_capacity == polygon.vertices_size) as i32 as libc::c_long != 0 {
            polygon.vertices_capacity = if polygon.vertices_capacity != 0 {
                polygon.vertices_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize_2: usize = ::core::mem::size_of::<Vec3>();
            let mut pData_2: *mut *mut libc::c_void =
                &mut polygon.vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
            *pData_2 = MemRealloc(
                polygon.vertices_data as *mut libc::c_void,
                (polygon.vertices_capacity as usize).wrapping_mul(elemSize_2),
            );
        }
        let fresh6 = polygon.vertices_size;
        polygon.vertices_size += 1;
        *(polygon.vertices_data).offset(fresh6 as isize) = v1;
        if (polygon.vertices_capacity == polygon.vertices_size) as i32 as libc::c_long != 0 {
            polygon.vertices_capacity = if polygon.vertices_capacity != 0 {
                polygon.vertices_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize_3: usize = ::core::mem::size_of::<Vec3>();
            let mut pData_3: *mut *mut libc::c_void =
                &mut polygon.vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
            *pData_3 = MemRealloc(
                polygon.vertices_data as *mut libc::c_void,
                (polygon.vertices_capacity as usize).wrapping_mul(elemSize_3),
            );
        }
        let fresh7 = polygon.vertices_size;
        polygon.vertices_size += 1;
        *(polygon.vertices_data).offset(fresh7 as isize) = v2;
        if (nodeData.polygons_capacity == nodeData.polygons_size) as i32 as libc::c_long != 0 {
            nodeData.polygons_capacity = if nodeData.polygons_capacity != 0 {
                nodeData.polygons_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize_4: usize = ::core::mem::size_of::<PolygonEx>();
            let mut pData_4: *mut *mut libc::c_void =
                &mut nodeData.polygons_data as *mut *mut PolygonEx as *mut *mut libc::c_void;
            *pData_4 = MemRealloc(
                nodeData.polygons_data as *mut libc::c_void,
                (nodeData.polygons_capacity as usize).wrapping_mul(elemSize_4),
            );
        }
        let fresh8 = nodeData.polygons_size;
        nodeData.polygons_size += 1;
        *(nodeData.polygons_data).offset(fresh8 as isize) = polygon;
        i += 3_i32;
    }
    let mut bspBuild: BSPBuild = BSPBuild {
        rootNode: std::ptr::null_mut(),
        rng: std::ptr::null_mut(),
        nodeCount: 0,
        leafCount: 0,
        triangleCount: 0,
    };
    bspBuild.rng = RNG_Create(1235_i32 as u64);
    bspBuild.rootNode = BSPBuild_CreateNode(&mut bspBuild, &mut nodeData);
    let mut nullLeaf: Triangle = Triangle {
        vertices: [Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        }; 3],
    };
    if ((*this).triangles_capacity < bspBuild.triangleCount + 2_i32) as libc::c_long != 0 {
        (*this).triangles_capacity = bspBuild.triangleCount + 2_i32;
        let mut elemSize_5: usize = ::core::mem::size_of::<Triangle>();
        let mut pData_5: *mut *mut libc::c_void =
            &mut (*this).triangles_data as *mut *mut Triangle as *mut *mut libc::c_void;
        *pData_5 = MemRealloc(
            (*this).triangles_data as *mut libc::c_void,
            ((*this).triangles_capacity as usize).wrapping_mul(elemSize_5),
        );
    }
    if ((*this).triangles_capacity == (*this).triangles_size) as i32 as libc::c_long != 0 {
        (*this).triangles_capacity = if (*this).triangles_capacity != 0 {
            (*this).triangles_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize_6: usize = ::core::mem::size_of::<Triangle>();
        let mut pData_6: *mut *mut libc::c_void =
            &mut (*this).triangles_data as *mut *mut Triangle as *mut *mut libc::c_void;
        *pData_6 = MemRealloc(
            (*this).triangles_data as *mut libc::c_void,
            ((*this).triangles_capacity as usize).wrapping_mul(elemSize_6),
        );
    }
    let fresh9 = (*this).triangles_size;
    (*this).triangles_size += 1;
    *((*this).triangles_data).offset(fresh9 as isize) = nullLeaf;
    if ((*this).triangles_capacity == (*this).triangles_size) as i32 as libc::c_long != 0 {
        (*this).triangles_capacity = if (*this).triangles_capacity != 0 {
            (*this).triangles_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize_7: usize = ::core::mem::size_of::<Triangle>();
        let mut pData_7: *mut *mut libc::c_void =
            &mut (*this).triangles_data as *mut *mut Triangle as *mut *mut libc::c_void;
        *pData_7 = MemRealloc(
            (*this).triangles_data as *mut libc::c_void,
            ((*this).triangles_capacity as usize).wrapping_mul(elemSize_7),
        );
    }
    let fresh10 = (*this).triangles_size;
    (*this).triangles_size += 1;
    *((*this).triangles_data).offset(fresh10 as isize) = nullLeaf;
    (*this).emptyLeaf.index = -EmptyLeafIndex;
    (*this).emptyLeaf.triangleCount = 0_i32 as u8;
    let mut nullNode: BSPNode = BSPNode {
        plane: Plane {
            n: Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            d: 0.,
        },
        child: [BSPNodeRef {
            index: 0,
            triangleCount: 0,
        }; 2],
    };
    if ((*this).nodes_capacity < bspBuild.nodeCount + 1_i32) as i32 as libc::c_long != 0 {
        (*this).nodes_capacity = bspBuild.nodeCount + 1_i32;
        let mut elemSize_8: usize = ::core::mem::size_of::<BSPNode>();
        let mut pData_8: *mut *mut libc::c_void =
            &mut (*this).nodes_data as *mut *mut BSPNode as *mut *mut libc::c_void;
        *pData_8 = MemRealloc(
            (*this).nodes_data as *mut libc::c_void,
            ((*this).nodes_capacity as usize).wrapping_mul(elemSize_8),
        );
    }
    if ((*this).nodes_capacity == (*this).nodes_size) as libc::c_long != 0 {
        (*this).nodes_capacity = if (*this).nodes_capacity != 0 {
            (*this).nodes_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize_9: usize = ::core::mem::size_of::<BSPNode>();
        let mut pData_9: *mut *mut libc::c_void =
            &mut (*this).nodes_data as *mut *mut BSPNode as *mut *mut libc::c_void;
        *pData_9 = MemRealloc(
            (*this).nodes_data as *mut libc::c_void,
            ((*this).nodes_capacity as usize).wrapping_mul(elemSize_9),
        );
    }
    let fresh11 = (*this).nodes_size;
    (*this).nodes_size += 1;
    *((*this).nodes_data).offset(fresh11 as isize) = nullNode;
    (*this).rootNode = BSPBuild_OptimizeTree(this, bspBuild.rootNode);
    BSPBuild_FreeNode(bspBuild.rootNode);
    RNG_Free(bspBuild.rng);
    this
}

#[no_mangle]
pub unsafe extern "C" fn BSP_Free(mut this: *mut BSP) {
    if this.is_null() {
        return;
    }
    MemFree((*this).nodes_data as *const libc::c_void);
    MemFree((*this).triangles_data as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_GetNode(
    mut this: *mut BSP,
    mut nodeRef: BSPNodeRef,
    mut relationship: BSPNodeRel,
) -> BSPNodeRef {
    if this.is_null() {
        Fatal(b"BSP_GetNode: bsp is null\0" as *const u8 as *const libc::c_char);
    }
    if nodeRef.index == 0 {
        return (*this).rootNode;
    }
    let mut node: *mut BSPNode = std::ptr::null_mut();
    if nodeRef.index > 0_i32 {
        node = ((*this).nodes_data).offset(nodeRef.index as isize);
    }
    let mut newNode: BSPNodeRef = BSPNodeRef {
        index: 0,
        triangleCount: 0,
    };
    let mut current_block_15: u64;
    match relationship as i32 {
        0 => {
            current_block_15 = 1626635900302357725;
        }
        1 => {
            if !node.is_null() {
                newNode = (*node).child[BackIndex as usize];
            }
            current_block_15 = 4495394744059808450;
        }
        2 => {
            if !node.is_null() {
                newNode = (*node).child[FrontIndex as usize];
            }
            current_block_15 = 4495394744059808450;
        }
        _ => Fatal(
            b"BSPDebug_GetNode: Unhandled case: %i\0" as *const u8 as *const libc::c_char,
            relationship as i32,
        ),
    }
    match current_block_15 {
        1626635900302357725 => {
            if nodeRef.index != 0 {
                let mut i: i32 = 0_i32;
                while i < (*this).nodes_size {
                    let mut nodeToCheck: *mut BSPNode = ((*this).nodes_data).offset(i as isize);
                    if (*nodeToCheck).child[BackIndex as usize].index == nodeRef.index {
                        newNode.index = i;
                        break;
                    } else if (*nodeToCheck).child[FrontIndex as usize].index == nodeRef.index {
                        newNode.index = i;
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
        }
        _ => {}
    }
    if newNode.index != 0 {
        newNode
    } else {
        nodeRef
    }
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawNode(mut this: *mut BSP, mut nodeRef: BSPNodeRef) {
    if nodeRef.index > 0_i32 {
        let mut node: *mut BSPNode = ((*this).nodes_data).offset(nodeRef.index as isize);
        BSPDebug_DrawNode(this, (*node).child[BackIndex as usize]);
        BSPDebug_DrawNode(this, (*node).child[FrontIndex as usize]);
    } else {
        let mut leaf: *mut Triangle = ((*this).triangles_data).offset(-nodeRef.index as isize);
        let mut i: u8 = 0_i32 as u8;
        while (i as i32) < nodeRef.triangleCount as i32 {
            let mut triangle: *mut Triangle = leaf.offset(i as i32 as isize);
            Draw_Poly3(((*triangle).vertices).as_mut_ptr(), 3_i32);
            i = i.wrapping_add(1);
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawNodeSplit(mut this: *mut BSP, mut nodeRef: BSPNodeRef) {
    RenderState_PushBlendMode(1_i32);
    RenderState_PushCullFace(1_i32);
    RenderState_PushDepthTest(true);
    RenderState_PushWireframe(true);
    if nodeRef.index > 0_i32 {
        let mut node: *mut BSPNode = ((*this).nodes_data).offset(nodeRef.index as isize);
        Draw_Color(0.5f32, 0.3f32, 0.3f32, 0.4f32);
        BSPDebug_DrawNode(this, (*node).child[BackIndex as usize]);
        Draw_Color(0.3f32, 0.5f32, 0.3f32, 0.4f32);
        BSPDebug_DrawNode(this, (*node).child[FrontIndex as usize]);
        let mut closestPoint = Vec3::ZERO;
        let mut origin: Vec3 = Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        };
        let mut t: f32 = Vec3::dot((*node).plane.n, origin) - (*node).plane.d;
        closestPoint = origin - ((*node).plane.n * t);
        RenderState_PushWireframe(false);
        Draw_Color(0.3f32, 0.5f32, 0.3f32, 0.4f32);
        Draw_Plane(&mut closestPoint, &mut (*node).plane.n, 2.0f32);
        Draw_Color(0.5f32, 0.3f32, 0.3f32, 0.4f32);
        let mut neg: Vec3 = (*node).plane.n * -1.0f32;
        Draw_Plane(&mut closestPoint, &mut neg, 2.0f32);
        RenderState_PopWireframe();
    } else {
        Draw_Color(0.5f32, 0.5f32, 0.3f32, 0.4f32);
        BSPDebug_DrawNode(this, nodeRef);
    }
    RenderState_PopWireframe();
    RenderState_PopDepthTest();
    RenderState_PopCullFace();
    RenderState_PopBlendMode();
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawLineSegment(
    mut bsp: *mut BSP,
    mut lineSegment: *mut LineSegment,
) {
    let mut pHit = Vec3::ZERO;
    if BSP_IntersectLineSegment(bsp, lineSegment, &mut pHit) {
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 0.1f32);
        Draw_Line3(&mut (*lineSegment).p0, &mut pHit);
        Draw_Color(1.0f32, 0.0f32, 0.0f32, 1.0f32);
        Draw_Line3(&mut pHit, &mut (*lineSegment).p1);
        Draw_PointSize(5.0f32);
        Draw_Point3(pHit.x, pHit.y, pHit.z);
    } else {
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
        Draw_Line3(&mut (*lineSegment).p0, &mut (*lineSegment).p1);
    };
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawSphere(mut this: *mut BSP, mut sphere: *mut Sphere) {
    let mut pHit = Vec3::ZERO;
    if BSP_IntersectSphere(this, sphere, &mut pHit) {
        RenderState_PushWireframe(false);
        Draw_Color(1.0f32, 0.0f32, 0.0f32, 0.3f32);
        Draw_Sphere(&mut (*sphere).p, (*sphere).r);
        RenderState_PopWireframe();
        Draw_Color(1.0f32, 0.0f32, 0.0f32, 1.0f32);
        Draw_Sphere(&mut (*sphere).p, (*sphere).r);
        RenderState_PushDepthTest(false);
        Draw_PointSize(8.0f32);
        Draw_Point3(pHit.x, pHit.y, pHit.z);
        RenderState_PopDepthTest();
    } else {
        RenderState_PushWireframe(false);
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 0.3f32);
        Draw_Sphere(&mut (*sphere).p, (*sphere).r);
        RenderState_PopWireframe();
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
        Draw_Sphere(&mut (*sphere).p, (*sphere).r);
    };
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_PrintRayProfilingData(mut _this: *mut BSP, mut _totalTime: f64) {
    Warn(
        b"BSP_PrintRayProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function.\0"
            as *const u8 as *const libc::c_char,
    );
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_PrintSphereProfilingData(
    mut _this: *mut BSP,
    mut _totalTime: f64,
) {
    Warn(
        b"BSP_PrintSphereProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function.\0"
            as *const u8 as *const libc::c_char,
    );
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_GetIntersectSphereTriangles(
    mut this: *mut BSP,
    mut sphere: *mut Sphere,
    mut sphereProf: *mut IntersectSphereProfiling,
) -> bool {
    let mut nodeRef: BSPNodeRef = (*this).rootNode;
    let mut hit: bool = false;
    let mut depth: i32 = 0_i32;
    let mut maxDepth: i32 = 0_i32;
    loop {
        maxDepth = f64::max(depth as f64, maxDepth as f64) as i32;
        if nodeRef.index >= 0_i32 {
            let mut node: *mut BSPNode = ((*this).nodes_data).offset(nodeRef.index as isize);
            (*sphereProf).nodes += 1;
            let mut dist: f32 = Vec3::dot((*node).plane.n, (*sphere).p) - (*node).plane.d;
            if dist as f64 > (*sphere).r as f64 + 2.0f32 as f64 * 1e-4f64 {
                nodeRef = (*node).child[FrontIndex as usize];
            } else if (dist as f64) < -((*sphere).r as f64 + 2.0f32 as f64 * 1e-4f64) {
                nodeRef = (*node).child[BackIndex as usize];
            } else {
                let mut d: Delay = Delay {
                    nodeRef: (*node).child[BackIndex as usize],
                    depth: depth,
                };
                if (nodeStack_capacity == nodeStack_size) as libc::c_long != 0 {
                    nodeStack_capacity = if nodeStack_capacity != 0 {
                        nodeStack_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize: usize = ::core::mem::size_of::<Delay>();
                    let mut pData: *mut *mut libc::c_void =
                        &mut nodeStack_data as *mut *mut Delay as *mut *mut libc::c_void;
                    *pData = MemRealloc(
                        nodeStack_data as *mut libc::c_void,
                        (nodeStack_capacity as usize).wrapping_mul(elemSize),
                    );
                }
                let fresh12 = nodeStack_size;
                nodeStack_size += 1;
                *nodeStack_data.offset(fresh12 as isize) = d;
                nodeRef = (*node).child[FrontIndex as usize];
            }
            depth += 1;
        } else {
            let mut leaf: *mut Triangle = ((*this).triangles_data).offset(-nodeRef.index as isize);
            (*sphereProf).leaves += 1;
            let mut i: u8 = 0_i32 as u8;
            while (i as i32) < nodeRef.triangleCount as i32 {
                let mut triangle: *mut Triangle = leaf.offset(i as i32 as isize);
                (*sphereProf).triangles += 1;
                let mut pHit2 = Vec3::ZERO;
                if Intersect_SphereTriangle(sphere, triangle, &mut pHit2) {
                    let mut t: TriangleTest = TriangleTest {
                        triangle: triangle,
                        hit: true,
                    };
                    if ((*sphereProf).triangleTests_capacity == (*sphereProf).triangleTests_size)
                        as i32 as libc::c_long
                        != 0
                    {
                        (*sphereProf).triangleTests_capacity =
                            if (*sphereProf).triangleTests_capacity != 0 {
                                (*sphereProf).triangleTests_capacity * 2_i32
                            } else {
                                1_i32
                            };
                        let mut elemSize_0: usize = ::core::mem::size_of::<TriangleTest>();
                        let mut pData_0: *mut *mut libc::c_void =
                            &mut (*sphereProf).triangleTests_data as *mut *mut TriangleTest
                                as *mut *mut libc::c_void;
                        *pData_0 = MemRealloc(
                            (*sphereProf).triangleTests_data as *mut libc::c_void,
                            ((*sphereProf).triangleTests_capacity as usize)
                                .wrapping_mul(elemSize_0),
                        );
                    }
                    let fresh13 = (*sphereProf).triangleTests_size;
                    (*sphereProf).triangleTests_size += 1;
                    *((*sphereProf).triangleTests_data).offset(fresh13 as isize) = t;
                    hit = true;
                    break;
                } else {
                    let mut t_0: TriangleTest = TriangleTest {
                        triangle: triangle,
                        hit: false,
                    };
                    if ((*sphereProf).triangleTests_capacity == (*sphereProf).triangleTests_size)
                        as i32 as libc::c_long
                        != 0
                    {
                        (*sphereProf).triangleTests_capacity =
                            if (*sphereProf).triangleTests_capacity != 0 {
                                (*sphereProf).triangleTests_capacity * 2_i32
                            } else {
                                1_i32
                            };
                        let mut elemSize_1: usize = ::core::mem::size_of::<TriangleTest>();
                        let mut pData_1: *mut *mut libc::c_void =
                            &mut (*sphereProf).triangleTests_data as *mut *mut TriangleTest
                                as *mut *mut libc::c_void;
                        *pData_1 = MemRealloc(
                            (*sphereProf).triangleTests_data as *mut libc::c_void,
                            ((*sphereProf).triangleTests_capacity as usize)
                                .wrapping_mul(elemSize_1),
                        );
                    }
                    let fresh14 = (*sphereProf).triangleTests_size;
                    (*sphereProf).triangleTests_size += 1;
                    *((*sphereProf).triangleTests_data).offset(fresh14 as isize) = t_0;
                    i = i.wrapping_add(1);
                }
            }
            if hit {
                break;
            }
            if nodeStack_size == 0_i32 {
                break;
            }
            nodeStack_size -= 1;
            let mut d_0: Delay = *nodeStack_data.offset(nodeStack_size as isize);
            nodeRef = d_0.nodeRef;
            depth = d_0.depth;
        }
    }
    nodeStack_size = 0_i32;
    hit
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_GetLeaf(mut this: *mut BSP, mut leafIndex: i32) -> BSPNodeRef {
    let mut index: i32 = -1_i32;
    let mut node: *mut BSPNode = (*this).nodes_data;
    let mut __iterend: *mut BSPNode = ((*this).nodes_data).offset((*this).nodes_size as isize);
    while node < __iterend {
        if (*node).child[0].index < 0_i32 {
            let fresh15 = index;
            index += 1;
            if fresh15 == leafIndex {
                return (*node).child[0];
            }
        }
        if (*node).child[1].index < 0_i32 {
            let fresh16 = index;
            index += 1;
            if fresh16 == leafIndex {
                return (*node).child[1];
            }
        }
        node = node.offset(1);
    }
    let mut result: BSPNodeRef = BSPNodeRef {
        index: RootNodeIndex,
        triangleCount: 0_i32 as u8,
    };
    result
}
