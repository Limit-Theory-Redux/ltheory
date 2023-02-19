use ::libc;
use crate::internal::Memory::*;
use glam::Vec2;

extern "C" {
    pub type Mesh;
    pub type Matrix;
    fn Draw_Box3(box_0: *const Box3f);
    fn Draw_Color(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn Matrix_Free(_: *mut Matrix);
    fn Matrix_Inverse(_: *const Matrix) -> *mut Matrix;
    fn Matrix_MulDir(
        _: *const Matrix,
        out: *mut Vec3f,
        x: libc::c_float,
        y: libc::c_float,
        z: libc::c_float,
    );
    fn Matrix_MulPoint(
        _: *const Matrix,
        out: *mut Vec3f,
        x: libc::c_float,
        y: libc::c_float,
        z: libc::c_float,
    );
    fn Mesh_GetBound(_: *mut Mesh, out: *mut Box3f);
    fn Mesh_GetIndexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetIndexData(_: *mut Mesh) -> *mut libc::c_int;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
}
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
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
    pub id: uint64,
    pub box_0: Box3f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3f {
    pub lower: Vec3f,
    pub upper: Vec3f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3f,
    pub n: Vec3f,
    pub uv: Vec2,
}
#[inline]
unsafe extern "C" fn Box3f_IntersectsRay(
    mut self_0: Box3f,
    mut ro: Vec3f,
    mut rdi: Vec3f,
) -> bool {
    let mut t1: libc::c_double = (rdi.x * (self_0.lower.x - ro.x)) as libc::c_double;
    let mut t2: libc::c_double = (rdi.x * (self_0.upper.x - ro.x)) as libc::c_double;
    let mut tMin: libc::c_double = Min(t1, t2);
    let mut tMax: libc::c_double = Max(t1, t2);
    t1 = (rdi.y * (self_0.lower.y - ro.y)) as libc::c_double;
    t2 = (rdi.y * (self_0.upper.y - ro.y)) as libc::c_double;
    tMin = Max(tMin, Min(t1, t2));
    tMax = Min(tMax, Max(t1, t2));
    t1 = (rdi.z * (self_0.lower.z - ro.z)) as libc::c_double;
    t2 = (rdi.z * (self_0.upper.z - ro.z)) as libc::c_double;
    tMin = Max(tMin, Min(t1, t2));
    tMax = Min(tMax, Max(t1, t2));
    return tMax >= tMin && tMax > 0 as libc::c_int as libc::c_double;
}
#[inline]
unsafe extern "C" fn Box3f_IntersectsBox(mut a: Box3f, mut b: Box3f) -> bool {
    if a.lower.x > b.upper.x || a.upper.x < b.lower.x {
        return 0 as libc::c_int != 0;
    }
    if a.lower.y > b.upper.y || a.upper.y < b.lower.y {
        return 0 as libc::c_int != 0;
    }
    if a.lower.z > b.upper.z || a.upper.z < b.lower.z {
        return 0 as libc::c_int != 0;
    }
    return 1 as libc::c_int != 0;
}
#[inline]
unsafe extern "C" fn Box3f_Intersection(mut a: Box3f, mut b: Box3f) -> Box3f {
    let mut self_0: Box3f = {
        let mut init = Box3f {
            lower: {
                let mut init = Vec3f {
                    x: Maxf(a.lower.x, b.lower.x),
                    y: Maxf(a.lower.y, b.lower.y),
                    z: Maxf(a.lower.z, b.lower.z),
                };
                init
            },
            upper: {
                let mut init = Vec3f {
                    x: Minf(a.upper.x, b.upper.x),
                    y: Minf(a.upper.y, b.upper.y),
                    z: Minf(a.upper.z, b.upper.z),
                };
                init
            },
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Box3f_Center(mut self_0: Box3f) -> Vec3f {
    let mut center: Vec3f = {
        let mut init = Vec3f {
            x: (self_0.lower.x + self_0.upper.x) / 2 as libc::c_int as libc::c_float,
            y: (self_0.lower.y + self_0.upper.y) / 2 as libc::c_int as libc::c_float,
            z: (self_0.lower.z + self_0.upper.z) / 2 as libc::c_int as libc::c_float,
        };
        init
    };
    return center;
}
#[inline]
unsafe extern "C" fn Box3f_Create(mut lower: Vec3f, mut upper: Vec3f) -> Box3f {
    let mut result: Box3f = {
        let mut init = Box3f {
            lower: lower,
            upper: upper,
        };
        init
    };
    return result;
}
#[inline]
unsafe extern "C" fn Vec3f_Rcp(mut a: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: (1.0f64 / a.x as libc::c_double) as libc::c_float,
            y: (1.0f64 / a.y as libc::c_double) as libc::c_float,
            z: (1.0f64 / a.z as libc::c_double) as libc::c_float,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Maxf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Vec3f_Max(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: Maxf(a.x, b.x),
            y: Maxf(a.y, b.y),
            z: Maxf(a.z, b.z),
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Min(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: Minf(a.x, b.x),
            y: Minf(a.y, b.y),
            z: Minf(a.z, b.z),
        };
        init
    };
    return self_0;
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Create(mut box_0: Box3f) -> *mut Octree {
    let mut self_0: *mut Octree = MemAlloc(
        ::core::mem::size_of::<Octree>() as usize,
    ) as *mut Octree;
    MemZero(
        self_0 as *mut libc::c_void,
        ::core::mem::size_of::<Octree>() as usize,
    );
    (*self_0).box_0 = box_0;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Octree_Free(mut self_0: *mut Octree) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if !((*self_0).child[i as usize]).is_null() {
            Octree_Free((*self_0).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*self_0).elems;
    while !elem.is_null() {
        let mut next: *mut Node = (*elem).next;
        MemFree(elem as *const libc::c_void);
        elem = next;
    }
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Octree_FromMesh(mut mesh: *mut Mesh) -> *mut Octree {
    let mut meshBox: Box3f = Box3f {
        lower: Vec3f { x: 0., y: 0., z: 0. },
        upper: Vec3f { x: 0., y: 0., z: 0. },
    };
    Mesh_GetBound(mesh, &mut meshBox);
    let mut self_0: *mut Octree = Octree_Create(meshBox);
    let mut indexCount: libc::c_int = Mesh_GetIndexCount(mesh);
    let mut indexData: *const libc::c_int = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < indexCount {
        let mut v0: *const Vertex = vertexData
            .offset(*indexData.offset((i + 0 as libc::c_int) as isize) as isize);
        let mut v1: *const Vertex = vertexData
            .offset(*indexData.offset((i + 1 as libc::c_int) as isize) as isize);
        let mut v2: *const Vertex = vertexData
            .offset(*indexData.offset((i + 2 as libc::c_int) as isize) as isize);
        let mut box_0: Box3f = Box3f_Create(
            Vec3f_Min((*v0).p, Vec3f_Min((*v1).p, (*v2).p)),
            Vec3f_Max((*v0).p, Vec3f_Max((*v1).p, (*v2).p)),
        );
        Octree_Add(self_0, box_0, (i / 3 as libc::c_int) as uint32);
        i += 3 as libc::c_int;
    }
    return self_0;
}
unsafe extern "C" fn Octree_GetAvgLoadImpl(
    mut self_0: *mut Octree,
    mut load: *mut libc::c_double,
    mut nodes: *mut libc::c_double,
) {
    *nodes += 1 as libc::c_int as libc::c_double;
    let mut elem: *mut Node = (*self_0).elems;
    while !elem.is_null() {
        *load += 1 as libc::c_int as libc::c_double;
        elem = (*elem).next;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if !((*self_0).child[i as usize]).is_null() {
            Octree_GetAvgLoadImpl((*self_0).child[i as usize], load, nodes);
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Octree_GetAvgLoad(mut self_0: *mut Octree) -> libc::c_double {
    let mut load: libc::c_double = 0 as libc::c_int as libc::c_double;
    let mut nodes: libc::c_double = 0 as libc::c_int as libc::c_double;
    Octree_GetAvgLoadImpl(self_0, &mut load, &mut nodes);
    return load / nodes;
}
#[no_mangle]
pub unsafe extern "C" fn Octree_GetMaxLoad(mut self_0: *mut Octree) -> libc::c_int {
    let mut load: libc::c_int = 0 as libc::c_int;
    let mut elem: *mut Node = (*self_0).elems;
    while !elem.is_null() {
        load += 1 as libc::c_int;
        elem = (*elem).next;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if !((*self_0).child[i as usize]).is_null() {
            load = Max(
                load as libc::c_double,
                Octree_GetMaxLoad((*self_0).child[i as usize]) as libc::c_double,
            ) as libc::c_int;
        }
        i += 1;
    }
    return load;
}
#[no_mangle]
pub unsafe extern "C" fn Octree_GetMemory(mut self_0: *mut Octree) -> libc::c_int {
    let mut memory: libc::c_int = ::core::mem::size_of::<Octree>() as usize
        as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if !((*self_0).child[i as usize]).is_null() {
            memory += Octree_GetMemory((*self_0).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*self_0).elems;
    while !elem.is_null() {
        memory = (memory as usize)
            .wrapping_add(::core::mem::size_of::<Node>()) as libc::c_int
            as libc::c_int;
        elem = (*elem).next;
    }
    return memory;
}
unsafe extern "C" fn Octree_IntersectRayImpl(
    mut self_0: *mut Octree,
    mut o: Vec3f,
    mut di: Vec3f,
) -> bool {
    if !Box3f_IntersectsRay((*self_0).box_0, o, di) {
        return 0 as libc::c_int != 0;
    }
    let mut elem: *mut Node = (*self_0).elems;
    while !elem.is_null() {
        if Box3f_IntersectsRay((*elem).box_0, o, di) {
            return 1 as libc::c_int != 0;
        }
        elem = (*elem).next;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if !((*self_0).child[i as usize]).is_null() {
            if Octree_IntersectRayImpl((*self_0).child[i as usize], o, di) {
                return 1 as libc::c_int != 0;
            }
        }
        i += 1;
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Octree_IntersectRay(
    mut self_0: *mut Octree,
    mut matrix: *mut Matrix,
    mut ro: *const Vec3f,
    mut rd: *const Vec3f,
) -> bool {
    let mut inv: *mut Matrix = Matrix_Inverse(matrix);
    let mut invRo: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_MulPoint(inv, &mut invRo, (*ro).x, (*ro).y, (*ro).z);
    let mut invRd: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_MulDir(inv, &mut invRd, (*rd).x, (*rd).y, (*rd).z);
    Matrix_Free(inv);
    return Octree_IntersectRayImpl(self_0, invRo, Vec3f_Rcp(invRd));
}
unsafe extern "C" fn Octree_Insert(
    mut self_0: *mut Octree,
    mut box_0: Box3f,
    mut id: uint32,
) {
    let mut elem: *mut Node = MemAlloc(::core::mem::size_of::<Node>())
        as *mut Node;
    (*elem).box_0 = box_0;
    (*elem).id = id as uint64;
    (*elem).next = (*self_0).elems;
    (*self_0).elems = elem;
}
unsafe extern "C" fn Octree_AddDepth(
    mut self_0: *mut Octree,
    mut box_0: Box3f,
    mut id: uint32,
    mut depth: libc::c_int,
) {
    let L: *const Vec3f = &mut (*self_0).box_0.lower;
    let U: *const Vec3f = &mut (*self_0).box_0.upper;
    let C: Vec3f = Box3f_Center((*self_0).box_0);
    let childBound: [Box3f; 8] = [
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f {
                        x: (*L).x,
                        y: (*L).y,
                        z: (*L).z,
                    };
                    init
                },
                upper: {
                    let mut init = Vec3f { x: C.x, y: C.y, z: C.z };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f {
                        x: C.x,
                        y: (*L).y,
                        z: (*L).z,
                    };
                    init
                },
                upper: {
                    let mut init = Vec3f { x: (*U).x, y: C.y, z: C.z };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f {
                        x: (*L).x,
                        y: C.y,
                        z: (*L).z,
                    };
                    init
                },
                upper: {
                    let mut init = Vec3f { x: C.x, y: (*U).y, z: C.z };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f { x: C.x, y: C.y, z: (*L).z };
                    init
                },
                upper: {
                    let mut init = Vec3f {
                        x: (*U).x,
                        y: (*U).y,
                        z: C.z,
                    };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f {
                        x: (*L).x,
                        y: (*L).y,
                        z: C.z,
                    };
                    init
                },
                upper: {
                    let mut init = Vec3f { x: C.x, y: C.y, z: (*U).z };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f { x: C.x, y: (*L).y, z: C.z };
                    init
                },
                upper: {
                    let mut init = Vec3f {
                        x: (*U).x,
                        y: C.y,
                        z: (*U).z,
                    };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f { x: (*L).x, y: C.y, z: C.z };
                    init
                },
                upper: {
                    let mut init = Vec3f {
                        x: C.x,
                        y: (*U).y,
                        z: (*U).z,
                    };
                    init
                },
            };
            init
        },
        {
            let mut init = Box3f {
                lower: {
                    let mut init = Vec3f { x: C.x, y: C.y, z: C.z };
                    init
                },
                upper: {
                    let mut init = Vec3f {
                        x: (*U).x,
                        y: (*U).y,
                        z: (*U).z,
                    };
                    init
                },
            };
            init
        },
    ];
    let mut intersections: libc::c_int = 0 as libc::c_int;
    let mut lastIntersection: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if Box3f_IntersectsBox(box_0, childBound[i as usize]) {
            intersections += 1;
            lastIntersection = i;
        }
        i += 1;
    }
    if intersections == 0 as libc::c_int {
        return;
    }
    if intersections == 1 as libc::c_int {
        if ((*self_0).child[lastIntersection as usize]).is_null() {
            (*self_0)
                .child[lastIntersection
                as usize] = Octree_Create(childBound[lastIntersection as usize]);
        }
        Octree_AddDepth(
            (*self_0).child[lastIntersection as usize],
            Box3f_Intersection(box_0, childBound[lastIntersection as usize]),
            id,
            depth + 1 as libc::c_int,
        );
        return;
    }
    Octree_Insert(self_0, box_0, id);
}
#[no_mangle]
pub unsafe extern "C" fn Octree_Add(
    mut self_0: *mut Octree,
    mut box_0: Box3f,
    mut id: uint32,
) {
    Octree_AddDepth(self_0, box_0, id, 0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn Octree_Draw(mut self_0: *mut Octree) {
    Draw_Color(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    Draw_Box3(&mut (*self_0).box_0);
    Draw_Color(
        0 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    let mut elem: *mut Node = (*self_0).elems;
    while !elem.is_null() {
        Draw_Box3(&mut (*elem).box_0);
        elem = (*elem).next;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if !((*self_0).child[i as usize]).is_null() {
            Octree_Draw((*self_0).child[i as usize]);
        }
        i += 1;
    }
}
