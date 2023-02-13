use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Mesh;
    pub type Matrix;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn Draw_Color(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn Draw_Box3(box_0: *const Box3f);
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
    fn Mesh_GetIndexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetIndexData(_: *mut Mesh) -> *mut libc::c_int;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
}
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BoxTree {
    pub root: *mut Node,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub box_0: Box3f,
    pub data: *mut libc::c_void,
    pub sub: [*mut Node; 2],
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
pub struct Vec2f {
    pub x: libc::c_float,
    pub y: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3f,
    pub n: Vec3f,
    pub uv: Vec2f,
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
unsafe extern "C" fn Box3f_Volume(mut self_0: Box3f) -> libc::c_float {
    return (self_0.upper.x - self_0.lower.x) * (self_0.upper.y - self_0.lower.y)
        * (self_0.upper.z - self_0.lower.z);
}
#[inline]
unsafe extern "C" fn Box3f_Union(mut a: Box3f, mut b: Box3f) -> Box3f {
    let mut self_0: Box3f = {
        let mut init = Box3f {
            lower: {
                let mut init = Vec3f {
                    x: Minf(a.lower.x, b.lower.x),
                    y: Minf(a.lower.y, b.lower.y),
                    z: Minf(a.lower.z, b.lower.z),
                };
                init
            },
            upper: {
                let mut init = Vec3f {
                    x: Maxf(a.upper.x, b.upper.x),
                    y: Maxf(a.upper.y, b.upper.y),
                    z: Maxf(a.upper.z, b.upper.z),
                };
                init
            },
        };
        init
    };
    return self_0;
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
unsafe extern "C" fn Box3f_ContainsBox(mut a: Box3f, mut b: Box3f) -> bool {
    return a.lower.x <= b.lower.x && a.upper.x >= b.upper.x && a.lower.y <= b.lower.y
        && a.upper.y >= b.upper.y && a.lower.z <= b.lower.z && a.upper.z >= b.upper.z;
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
unsafe extern "C" fn Node_Create(
    mut box_0: Box3f,
    mut data: *mut libc::c_void,
) -> *mut Node {
    let mut self_0: *mut Node = MemAlloc(::core::mem::size_of::<Node>())
        as *mut Node;
    (*self_0).box_0 = box_0;
    (*self_0).sub[0 as libc::c_int as usize] = 0 as *mut Node;
    (*self_0).sub[1 as libc::c_int as usize] = 0 as *mut Node;
    (*self_0).data = data;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_Create() -> *mut BoxTree {
    let mut self_0: *mut BoxTree = MemAlloc(
        ::core::mem::size_of::<BoxTree>() as usize,
    ) as *mut BoxTree;
    (*self_0).root = 0 as *mut Node;
    return self_0;
}
unsafe extern "C" fn Node_Free(mut self_0: *mut Node) {
    if !((*self_0).sub[0 as libc::c_int as usize]).is_null() {
        Node_Free((*self_0).sub[0 as libc::c_int as usize]);
    }
    if !((*self_0).sub[1 as libc::c_int as usize]).is_null() {
        Node_Free((*self_0).sub[1 as libc::c_int as usize]);
    }
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_Free(mut self_0: *mut BoxTree) {
    if !((*self_0).root).is_null() {
        Node_Free((*self_0).root);
    }
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_FromMesh(mut mesh: *mut Mesh) -> *mut BoxTree {
    let mut self_0: *mut BoxTree = BoxTree_Create();
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
        BoxTree_Add(self_0, box_0, 0 as *mut libc::c_void);
        i += 3 as libc::c_int;
    }
    return self_0;
}
#[inline]
unsafe extern "C" fn Cost(mut box_0: Box3f) -> libc::c_float {
    return Box3f_Volume(box_0);
}
#[inline]
unsafe extern "C" fn CostMerge(mut a: Box3f, mut b: Box3f) -> libc::c_float {
    return Cost(Box3f_Union(a, b));
}
unsafe extern "C" fn Node_Merge(
    mut self_0: *mut Node,
    mut src: *mut Node,
    mut prev: *mut *mut Node,
) {
    if self_0.is_null() {
        *prev = src;
        return;
    }
    if ((*self_0).sub[0 as libc::c_int as usize]).is_null() {
        let mut parent: *mut Node = Node_Create(
            Box3f_Union((*self_0).box_0, (*src).box_0),
            0 as *mut libc::c_void,
        );
        *prev = parent;
        (*parent).sub[0 as libc::c_int as usize] = self_0;
        (*parent).sub[1 as libc::c_int as usize] = src;
        self_0 = parent;
        return;
    }
    if Box3f_ContainsBox((*self_0).box_0, (*src).box_0) {
        let mut cost0: libc::c_float = CostMerge(
            (*(*self_0).sub[0 as libc::c_int as usize]).box_0,
            (*src).box_0,
        ) + Cost((*(*self_0).sub[1 as libc::c_int as usize]).box_0);
        let mut cost1: libc::c_float = CostMerge(
            (*(*self_0).sub[1 as libc::c_int as usize]).box_0,
            (*src).box_0,
        ) + Cost((*(*self_0).sub[0 as libc::c_int as usize]).box_0);
        if cost0 < cost1 {
            Node_Merge(
                (*self_0).sub[0 as libc::c_int as usize],
                src,
                &mut *((*self_0).sub).as_mut_ptr().offset(0 as libc::c_int as isize),
            );
        } else {
            Node_Merge(
                (*self_0).sub[1 as libc::c_int as usize],
                src,
                &mut *((*self_0).sub).as_mut_ptr().offset(1 as libc::c_int as isize),
            );
        }
    } else {
        let mut parent_0: *mut Node = Node_Create(
            Box3f_Union((*self_0).box_0, (*src).box_0),
            0 as *mut libc::c_void,
        );
        *prev = parent_0;
        let mut costBase: libc::c_float = Cost((*self_0).box_0) + Cost((*src).box_0);
        let mut cost0_0: libc::c_float = CostMerge(
            (*(*self_0).sub[0 as libc::c_int as usize]).box_0,
            (*src).box_0,
        ) + Cost((*(*self_0).sub[1 as libc::c_int as usize]).box_0);
        let mut cost1_0: libc::c_float = CostMerge(
            (*(*self_0).sub[1 as libc::c_int as usize]).box_0,
            (*src).box_0,
        ) + Cost((*(*self_0).sub[0 as libc::c_int as usize]).box_0);
        if costBase <= cost0_0 && costBase <= cost1_0 {
            (*parent_0).sub[0 as libc::c_int as usize] = self_0;
            (*parent_0).sub[1 as libc::c_int as usize] = src;
        } else if cost0_0 <= costBase && cost0_0 <= cost1_0 {
            (*parent_0)
                .sub[0 as libc::c_int
                as usize] = (*self_0).sub[0 as libc::c_int as usize];
            (*parent_0)
                .sub[1 as libc::c_int
                as usize] = (*self_0).sub[1 as libc::c_int as usize];
            MemFree(self_0 as *const libc::c_void);
            Node_Merge(
                (*parent_0).sub[0 as libc::c_int as usize],
                src,
                &mut *((*parent_0).sub).as_mut_ptr().offset(0 as libc::c_int as isize),
            );
        } else {
            (*parent_0)
                .sub[0 as libc::c_int
                as usize] = (*self_0).sub[0 as libc::c_int as usize];
            (*parent_0)
                .sub[1 as libc::c_int
                as usize] = (*self_0).sub[1 as libc::c_int as usize];
            MemFree(self_0 as *const libc::c_void);
            Node_Merge(
                (*parent_0).sub[1 as libc::c_int as usize],
                src,
                &mut *((*parent_0).sub).as_mut_ptr().offset(1 as libc::c_int as isize),
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_Add(
    mut self_0: *mut BoxTree,
    mut box_0: Box3f,
    mut data: *mut libc::c_void,
) {
    Node_Merge((*self_0).root, Node_Create(box_0, data), &mut (*self_0).root);
}
unsafe extern "C" fn Node_GetMemory(mut self_0: *mut Node) -> libc::c_int {
    let mut memory: libc::c_int = ::core::mem::size_of::<Node>() as usize
        as libc::c_int;
    if !((*self_0).sub[0 as libc::c_int as usize]).is_null() {
        memory += Node_GetMemory((*self_0).sub[0 as libc::c_int as usize]);
    }
    if !((*self_0).sub[1 as libc::c_int as usize]).is_null() {
        memory += Node_GetMemory((*self_0).sub[1 as libc::c_int as usize]);
    }
    return memory;
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_GetMemory(mut self_0: *mut BoxTree) -> libc::c_int {
    let mut memory: libc::c_int = ::core::mem::size_of::<BoxTree>() as usize
        as libc::c_int;
    if !((*self_0).root).is_null() {
        memory += Node_GetMemory((*self_0).root);
    }
    return memory;
}
unsafe extern "C" fn Node_IntersectRay(
    mut self_0: *mut Node,
    mut o: Vec3f,
    mut di: Vec3f,
) -> bool {
    if !Box3f_IntersectsRay((*self_0).box_0, o, di) {
        return 0 as libc::c_int != 0;
    }
    if !((*self_0).sub[0 as libc::c_int as usize]).is_null() {
        if Node_IntersectRay((*self_0).sub[0 as libc::c_int as usize], o, di) {
            return 1 as libc::c_int != 0;
        }
        if Node_IntersectRay((*self_0).sub[1 as libc::c_int as usize], o, di) {
            return 1 as libc::c_int != 0;
        }
        return 0 as libc::c_int != 0;
    } else {
        return 1 as libc::c_int != 0
    };
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_IntersectRay(
    mut self_0: *mut BoxTree,
    mut matrix: *mut Matrix,
    mut ro: *const Vec3f,
    mut rd: *const Vec3f,
) -> bool {
    if ((*self_0).root).is_null() {
        return 0 as libc::c_int != 0;
    }
    let mut inv: *mut Matrix = Matrix_Inverse(matrix);
    let mut invRo: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_MulPoint(inv, &mut invRo, (*ro).x, (*ro).y, (*ro).z);
    let mut invRd: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_MulDir(inv, &mut invRd, (*rd).x, (*rd).y, (*rd).z);
    Matrix_Free(inv);
    return Node_IntersectRay((*self_0).root, invRo, Vec3f_Rcp(invRd));
}
unsafe extern "C" fn BoxTree_DrawNode(mut self_0: *mut Node, mut maxDepth: libc::c_int) {
    if maxDepth < 0 as libc::c_int {
        return;
    }
    if !((*self_0).sub[0 as libc::c_int as usize]).is_null()
        || !((*self_0).sub[1 as libc::c_int as usize]).is_null()
    {
        Draw_Color(
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
        );
        Draw_Box3(&mut (*self_0).box_0);
    } else {
        Draw_Color(
            0 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
        );
        Draw_Box3(&mut (*self_0).box_0);
    }
    if !((*self_0).sub[0 as libc::c_int as usize]).is_null() {
        BoxTree_DrawNode(
            (*self_0).sub[0 as libc::c_int as usize],
            maxDepth - 1 as libc::c_int,
        );
    }
    if !((*self_0).sub[1 as libc::c_int as usize]).is_null() {
        BoxTree_DrawNode(
            (*self_0).sub[1 as libc::c_int as usize],
            maxDepth - 1 as libc::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn BoxTree_Draw(
    mut self_0: *mut BoxTree,
    mut maxDepth: libc::c_int,
) {
    if !((*self_0).root).is_null() {
        BoxTree_DrawNode((*self_0).root, maxDepth);
    }
}
