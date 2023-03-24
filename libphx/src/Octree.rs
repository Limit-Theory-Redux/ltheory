use crate::internal::Memory::*;
use crate::Common::*;
use crate::Draw::*;
use crate::Math::Box3;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Matrix::*;
use crate::Mesh::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Octree {
    pub box_0: Box3,
    pub child: [*mut Octree; 8],
    pub elems: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub next: *mut Node,
    pub id: u64,
    pub box_0: Box3,
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Create(mut box_0: Box3) -> *mut Octree {
    let mut this = MemNew!(Octree);
    MemZero(this as *mut _, std::mem::size_of::<Octree>());
    (*this).box_0 = box_0;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Free(this: *mut Octree) {
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            Octree_Free((*this).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        let mut next: *mut Node = (*elem).next;
        MemFree(elem as *const _);
        elem = next;
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_FromMesh(mut mesh: *mut Mesh) -> *mut Octree {
    let mut meshBox: Box3 = Box3 {
        lower: Vec3::ZERO,
        upper: Vec3::ZERO,
    };
    Mesh_GetBound(mesh, &mut meshBox);
    let this: *mut Octree = Octree_Create(meshBox);
    let mut indexCount: i32 = Mesh_GetIndexCount(mesh);
    let mut indexData: *const i32 = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let mut i: i32 = 0;
    while i < indexCount {
        let mut v0: *const Vertex = vertexData.offset(*indexData.offset((i + 0) as isize) as isize);
        let mut v1: *const Vertex = vertexData.offset(*indexData.offset((i + 1) as isize) as isize);
        let mut v2: *const Vertex = vertexData.offset(*indexData.offset((i + 2) as isize) as isize);
        let mut box_0: Box3 = Box3::new(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        Octree_Add(this, box_0, (i / 3) as u32);
        i += 3;
    }
    this
}

unsafe extern "C" fn Octree_GetAvgLoadImpl(
    this: *mut Octree,
    mut load: *mut f64,
    mut nodes: *mut f64,
) {
    *nodes += 1.0;
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        *load += 1.0;
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            Octree_GetAvgLoadImpl((*this).child[i as usize], load, nodes);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetAvgLoad(this: *mut Octree) -> f64 {
    let mut load: f64 = 0.0;
    let mut nodes: f64 = 0.0;
    Octree_GetAvgLoadImpl(this, &mut load, &mut nodes);
    load / nodes
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetMaxLoad(this: *mut Octree) -> i32 {
    let mut load: i32 = 0;
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        load += 1;
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            load = f64::max(
                load as f64,
                Octree_GetMaxLoad((*this).child[i as usize]) as f64,
            ) as i32;
        }
        i += 1;
    }
    load
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetMemory(this: *mut Octree) -> i32 {
    let mut memory: i32 = std::mem::size_of::<Octree>() as i32;
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            memory += Octree_GetMemory((*this).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        memory = (memory as usize).wrapping_add(std::mem::size_of::<Node>()) as i32;
        elem = (*elem).next;
    }
    memory
}

unsafe extern "C" fn Octree_IntersectRayImpl(
    this: *mut Octree,
    mut o: Vec3,
    mut di: Vec3,
) -> bool {
    if !(*this).box_0.intersects_ray(o, di) {
        return false;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        if (*elem).box_0.intersects_ray(o, di) {
            return true;
        }
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            if Octree_IntersectRayImpl((*this).child[i as usize], o, di) {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Octree_IntersectRay(
    this: *mut Octree,
    mut matrix: *mut Matrix,
    mut ro: *const Vec3,
    mut rd: *const Vec3,
) -> bool {
    let mut inv: *mut Matrix = Matrix_Inverse(matrix);
    let mut invRo = Vec3::ZERO;
    Matrix_MulPoint(inv, &mut invRo, (*ro).x, (*ro).y, (*ro).z);
    let mut invRd = Vec3::ZERO;
    Matrix_MulDir(inv, &mut invRd, (*rd).x, (*rd).y, (*rd).z);
    Matrix_Free(inv);
    Octree_IntersectRayImpl(this, invRo, invRd.recip())
}

unsafe extern "C" fn Octree_Insert(this: *mut Octree, mut box_0: Box3, mut id: u32) {
    let mut elem = MemNew!(Node);
    (*elem).box_0 = box_0;
    (*elem).id = id as u64;
    (*elem).next = (*this).elems;
    (*this).elems = elem;
}

unsafe extern "C" fn Octree_AddDepth(
    this: *mut Octree,
    mut box_0: Box3,
    mut id: u32,
    mut depth: i32,
) {
    let L: *const Vec3 = &mut (*this).box_0.lower;
    let U: *const Vec3 = &mut (*this).box_0.upper;
    let C: Vec3 = (*this).box_0.center();
    let childBound: [Box3; 8] = [
        Box3 {
            lower: Vec3 {
                x: (*L).x,
                y: (*L).y,
                z: (*L).z,
            },
            upper: Vec3 {
                x: C.x,
                y: C.y,
                z: C.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: C.x,
                y: (*L).y,
                z: (*L).z,
            },
            upper: Vec3 {
                x: (*U).x,
                y: C.y,
                z: C.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: (*L).x,
                y: C.y,
                z: (*L).z,
            },
            upper: Vec3 {
                x: C.x,
                y: (*U).y,
                z: C.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: C.x,
                y: C.y,
                z: (*L).z,
            },
            upper: Vec3 {
                x: (*U).x,
                y: (*U).y,
                z: C.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: (*L).x,
                y: (*L).y,
                z: C.z,
            },
            upper: Vec3 {
                x: C.x,
                y: C.y,
                z: (*U).z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: C.x,
                y: (*L).y,
                z: C.z,
            },
            upper: Vec3 {
                x: (*U).x,
                y: C.y,
                z: (*U).z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: (*L).x,
                y: C.y,
                z: C.z,
            },
            upper: Vec3 {
                x: C.x,
                y: (*U).y,
                z: (*U).z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: C.x,
                y: C.y,
                z: C.z,
            },
            upper: Vec3 {
                x: (*U).x,
                y: (*U).y,
                z: (*U).z,
            },
        },
    ];
    let mut intersections: i32 = 0;
    let mut lastIntersection: i32 = 0;
    let mut i: i32 = 0;
    while i < 8 {
        if Box3::intersects_box(box_0, childBound[i as usize]) {
            intersections += 1;
            lastIntersection = i;
        }
        i += 1;
    }
    if intersections == 0 {
        return;
    }
    if intersections == 1 {
        if ((*this).child[lastIntersection as usize]).is_null() {
            (*this).child[lastIntersection as usize] =
                Octree_Create(childBound[lastIntersection as usize]);
        }
        Octree_AddDepth(
            (*this).child[lastIntersection as usize],
            Box3::intersection(box_0, childBound[lastIntersection as usize]),
            id,
            depth + 1,
        );
        return;
    }
    Octree_Insert(this, box_0, id);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Add(this: *mut Octree, mut box_0: Box3, mut id: u32) {
    Octree_AddDepth(this, box_0, id, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Draw(this: *mut Octree) {
    Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    Draw_Box3(&mut (*this).box_0);
    Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        Draw_Box3(&mut (*elem).box_0);
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            Octree_Draw((*this).child[i as usize]);
        }
        i += 1;
    }
}
