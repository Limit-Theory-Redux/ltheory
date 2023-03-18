use crate::internal::Memory::*;
use crate::Draw::*;
use crate::Matrix::*;
use crate::Mesh::*;
use glam::Vec2;
use glam::Vec3;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Octree {
    pub box_0: Box3f,
    pub child: [*mut Octree; 8],
    pub elems: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub next: *mut Node,
    pub id: u64,
    pub box_0: Box3f,
}

#[inline]
unsafe extern "C" fn Box3f_IntersectsRay(mut this: Box3f, mut ro: Vec3, mut rdi: Vec3) -> bool {
    let mut t1: f64 = (rdi.x * (this.lower.x - ro.x)) as f64;
    let mut t2: f64 = (rdi.x * (this.upper.x - ro.x)) as f64;
    let mut tMin: f64 = Min(t1, t2);
    let mut tMax: f64 = Max(t1, t2);
    t1 = (rdi.y * (this.lower.y - ro.y)) as f64;
    t2 = (rdi.y * (this.upper.y - ro.y)) as f64;
    tMin = Max(tMin, Min(t1, t2));
    tMax = Min(tMax, Max(t1, t2));
    t1 = (rdi.z * (this.lower.z - ro.z)) as f64;
    t2 = (rdi.z * (this.upper.z - ro.z)) as f64;
    tMin = Max(tMin, Min(t1, t2));
    tMax = Min(tMax, Max(t1, t2));
    return tMax >= tMin && tMax > 0_i32 as f64;
}

#[inline]
unsafe extern "C" fn Box3f_IntersectsBox(mut a: Box3f, mut b: Box3f) -> bool {
    if a.lower.x > b.upper.x || a.upper.x < b.lower.x {
        return 0_i32 != 0;
    }
    if a.lower.y > b.upper.y || a.upper.y < b.lower.y {
        return 0_i32 != 0;
    }
    if a.lower.z > b.upper.z || a.upper.z < b.lower.z {
        return 0_i32 != 0;
    }
    return 1_i32 != 0;
}

#[inline]
unsafe extern "C" fn Box3f_Intersection(mut a: Box3f, mut b: Box3f) -> Box3f {
    let mut this: Box3f = Box3f {
        lower: Vec3 {
            x: Maxf(a.lower.x, b.lower.x),
            y: Maxf(a.lower.y, b.lower.y),
            z: Maxf(a.lower.z, b.lower.z),
        },
        upper: Vec3 {
            x: Minf(a.upper.x, b.upper.x),
            y: Minf(a.upper.y, b.upper.y),
            z: Minf(a.upper.z, b.upper.z),
        },
    };
    return this;
}

#[inline]
unsafe extern "C" fn Box3f_Center(mut this: Box3f) -> Vec3 {
    let mut center: Vec3 = Vec3 {
        x: (this.lower.x + this.upper.x) / 2.0f32,
        y: (this.lower.y + this.upper.y) / 2.0f32,
        z: (this.lower.z + this.upper.z) / 2.0f32,
    };
    return center;
}

#[inline]
unsafe extern "C" fn Box3f_Create(mut lower: Vec3, mut upper: Vec3) -> Box3f {
    let mut result: Box3f = Box3f {
        lower: lower,
        upper: upper,
    };
    return result;
}

#[inline]
unsafe extern "C" fn Maxf(mut a: f32, mut b: f32) -> f32 {
    return if a > b { a } else { b };
}

#[inline]
unsafe extern "C" fn Max(mut a: f64, mut b: f64) -> f64 {
    return if a > b { a } else { b };
}

#[inline]
unsafe extern "C" fn Minf(mut a: f32, mut b: f32) -> f32 {
    return if a < b { a } else { b };
}

#[inline]
unsafe extern "C" fn Min(mut a: f64, mut b: f64) -> f64 {
    return if a < b { a } else { b };
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Create(mut box_0: Box3f) -> *mut Octree {
    let mut this: *mut Octree = MemAlloc(::core::mem::size_of::<Octree>()) as *mut Octree;
    MemZero(
        this as *mut libc::c_void,
        ::core::mem::size_of::<Octree>(),
    );
    (*this).box_0 = box_0;
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Free(mut this: *mut Octree) {
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if !((*this).child[i as usize]).is_null() {
            Octree_Free((*this).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        let mut next: *mut Node = (*elem).next;
        MemFree(elem as *const libc::c_void);
        elem = next;
    }
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_FromMesh(mut mesh: *mut Mesh) -> *mut Octree {
    let mut meshBox: Box3f = Box3f {
        lower: Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        upper: Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
    };
    Mesh_GetBound(mesh, &mut meshBox);
    let mut this: *mut Octree = Octree_Create(meshBox);
    let mut indexCount: i32 = Mesh_GetIndexCount(mesh);
    let mut indexData: *const i32 = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let mut i: i32 = 0_i32;
    while i < indexCount {
        let mut v0: *const Vertex =
            vertexData.offset(*indexData.offset((i + 0_i32) as isize) as isize);
        let mut v1: *const Vertex =
            vertexData.offset(*indexData.offset((i + 1_i32) as isize) as isize);
        let mut v2: *const Vertex =
            vertexData.offset(*indexData.offset((i + 2_i32) as isize) as isize);
        let mut box_0: Box3f = Box3f_Create(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        Octree_Add(this, box_0, (i / 3_i32) as u32);
        i += 3_i32;
    }
    return this;
}

unsafe extern "C" fn Octree_GetAvgLoadImpl(
    mut this: *mut Octree,
    mut load: *mut f64,
    mut nodes: *mut f64,
) {
    *nodes += 1_f64;
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        *load += 1_f64;
        elem = (*elem).next;
    }
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if !((*this).child[i as usize]).is_null() {
            Octree_GetAvgLoadImpl((*this).child[i as usize], load, nodes);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetAvgLoad(mut this: *mut Octree) -> f64 {
    let mut load: f64 = 0_i32 as f64;
    let mut nodes: f64 = 0_i32 as f64;
    Octree_GetAvgLoadImpl(this, &mut load, &mut nodes);
    return load / nodes;
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetMaxLoad(mut this: *mut Octree) -> i32 {
    let mut load: i32 = 0_i32;
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        load += 1_i32;
        elem = (*elem).next;
    }
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if !((*this).child[i as usize]).is_null() {
            load = Max(
                load as f64,
                Octree_GetMaxLoad((*this).child[i as usize]) as f64,
            ) as i32;
        }
        i += 1;
    }
    return load;
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetMemory(mut this: *mut Octree) -> i32 {
    let mut memory: i32 = ::core::mem::size_of::<Octree>() as i32;
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if !((*this).child[i as usize]).is_null() {
            memory += Octree_GetMemory((*this).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        memory = (memory as usize).wrapping_add(::core::mem::size_of::<Node>()) as i32;
        elem = (*elem).next;
    }
    return memory;
}

unsafe extern "C" fn Octree_IntersectRayImpl(
    mut this: *mut Octree,
    mut o: Vec3,
    mut di: Vec3,
) -> bool {
    if !Box3f_IntersectsRay((*this).box_0, o, di) {
        return 0_i32 != 0;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        if Box3f_IntersectsRay((*elem).box_0, o, di) {
            return 1_i32 != 0;
        }
        elem = (*elem).next;
    }
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if !((*this).child[i as usize]).is_null() {
            if Octree_IntersectRayImpl((*this).child[i as usize], o, di) {
                return 1_i32 != 0;
            }
        }
        i += 1;
    }
    return 0_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn Octree_IntersectRay(
    mut this: *mut Octree,
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
    return Octree_IntersectRayImpl(this, invRo, invRd.recip());
}

unsafe extern "C" fn Octree_Insert(mut this: *mut Octree, mut box_0: Box3f, mut id: u32) {
    let mut elem: *mut Node = MemAlloc(::core::mem::size_of::<Node>()) as *mut Node;
    (*elem).box_0 = box_0;
    (*elem).id = id as u64;
    (*elem).next = (*this).elems;
    (*this).elems = elem;
}

unsafe extern "C" fn Octree_AddDepth(
    mut this: *mut Octree,
    mut box_0: Box3f,
    mut id: u32,
    mut depth: i32,
) {
    let L: *const Vec3 = &mut (*this).box_0.lower;
    let U: *const Vec3 = &mut (*this).box_0.upper;
    let C: Vec3 = Box3f_Center((*this).box_0);
    let childBound: [Box3f; 8] = [
        Box3f {
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
        Box3f {
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
        Box3f {
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
        Box3f {
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
        Box3f {
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
        Box3f {
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
        Box3f {
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
        Box3f {
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
    let mut intersections: i32 = 0_i32;
    let mut lastIntersection: i32 = 0_i32;
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if Box3f_IntersectsBox(box_0, childBound[i as usize]) {
            intersections += 1;
            lastIntersection = i;
        }
        i += 1;
    }
    if intersections == 0_i32 {
        return;
    }
    if intersections == 1_i32 {
        if ((*this).child[lastIntersection as usize]).is_null() {
            (*this).child[lastIntersection as usize] =
                Octree_Create(childBound[lastIntersection as usize]);
        }
        Octree_AddDepth(
            (*this).child[lastIntersection as usize],
            Box3f_Intersection(box_0, childBound[lastIntersection as usize]),
            id,
            depth + 1_i32,
        );
        return;
    }
    Octree_Insert(this, box_0, id);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Add(mut this: *mut Octree, mut box_0: Box3f, mut id: u32) {
    Octree_AddDepth(this, box_0, id, 0_i32);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Draw(mut this: *mut Octree) {
    Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    Draw_Box3(&mut (*this).box_0);
    Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        Draw_Box3(&mut (*elem).box_0);
        elem = (*elem).next;
    }
    let mut i: i32 = 0_i32;
    while i < 8_i32 {
        if !((*this).child[i as usize]).is_null() {
            Octree_Draw((*this).child[i as usize]);
        }
        i += 1;
    }
}
