use crate::internal::Memory::*;
use crate::Matrix::*;
use crate::Mesh::*;
use crate::Draw::*;
use glam::Vec2;
use glam::Vec3;
use libc;

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

#[inline]
unsafe extern "C" fn Box3f_Volume(mut this: Box3f) -> f32 {
    return (this.upper.x - this.lower.x)
        * (this.upper.y - this.lower.y)
        * (this.upper.z - this.lower.z);
}

#[inline]
unsafe extern "C" fn Box3f_Union(mut a: Box3f, mut b: Box3f) -> Box3f {
    let mut this: Box3f = Box3f {
        lower: Vec3 {
            x: Minf(a.lower.x, b.lower.x),
            y: Minf(a.lower.y, b.lower.y),
            z: Minf(a.lower.z, b.lower.z),
        },
        upper: Vec3 {
            x: Maxf(a.upper.x, b.upper.x),
            y: Maxf(a.upper.y, b.upper.y),
            z: Maxf(a.upper.z, b.upper.z),
        },
    };
    return this;
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
unsafe extern "C" fn Box3f_ContainsBox(mut a: Box3f, mut b: Box3f) -> bool {
    return a.lower.x <= b.lower.x
        && a.upper.x >= b.upper.x
        && a.lower.y <= b.lower.y
        && a.upper.y >= b.upper.y
        && a.lower.z <= b.lower.z
        && a.upper.z >= b.upper.z;
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
    return tMax >= tMin && tMax > 0 as i32 as f64;
}

#[inline]
unsafe extern "C" fn Node_Create(mut box_0: Box3f, mut data: *mut libc::c_void) -> *mut Node {
    let mut this: *mut Node = MemAlloc(::core::mem::size_of::<Node>()) as *mut Node;
    (*this).box_0 = box_0;
    (*this).sub[0] = std::ptr::null_mut();
    (*this).sub[1] = std::ptr::null_mut();
    (*this).data = data;
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Create() -> *mut BoxTree {
    let mut this: *mut BoxTree =
        MemAlloc(::core::mem::size_of::<BoxTree>() as usize) as *mut BoxTree;
    (*this).root = std::ptr::null_mut();
    return this;
}

unsafe extern "C" fn Node_Free(mut this: *mut Node) {
    if !((*this).sub[0]).is_null() {
        Node_Free((*this).sub[0]);
    }
    if !((*this).sub[1]).is_null() {
        Node_Free((*this).sub[1]);
    }
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Free(mut this: *mut BoxTree) {
    if !((*this).root).is_null() {
        Node_Free((*this).root);
    }
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_FromMesh(mut mesh: *mut Mesh) -> *mut BoxTree {
    let mut this: *mut BoxTree = BoxTree_Create();
    let mut indexCount: i32 = Mesh_GetIndexCount(mesh);
    let mut indexData: *const i32 = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let mut i: i32 = 0 as i32;
    while i < indexCount {
        let mut v0: *const Vertex =
            vertexData.offset(*indexData.offset((i + 0 as i32) as isize) as isize);
        let mut v1: *const Vertex =
            vertexData.offset(*indexData.offset((i + 1 as i32) as isize) as isize);
        let mut v2: *const Vertex =
            vertexData.offset(*indexData.offset((i + 2 as i32) as isize) as isize);
        let mut box_0: Box3f = Box3f_Create(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        BoxTree_Add(this, box_0, std::ptr::null_mut());
        i += 3 as i32;
    }
    return this;
}

#[inline]
unsafe extern "C" fn Cost(mut box_0: Box3f) -> f32 {
    return Box3f_Volume(box_0);
}

#[inline]
unsafe extern "C" fn CostMerge(mut a: Box3f, mut b: Box3f) -> f32 {
    return Cost(Box3f_Union(a, b));
}

unsafe extern "C" fn Node_Merge(mut this: *mut Node, mut src: *mut Node, mut prev: *mut *mut Node) {
    if this.is_null() {
        *prev = src;
        return;
    }
    if ((*this).sub[0]).is_null() {
        let mut parent: *mut Node = Node_Create(
            Box3f_Union((*this).box_0, (*src).box_0),
            std::ptr::null_mut(),
        );
        *prev = parent;
        (*parent).sub[0] = this;
        (*parent).sub[1] = src;
        this = parent;
        return;
    }
    if Box3f_ContainsBox((*this).box_0, (*src).box_0) {
        let mut cost0: f32 =
            CostMerge((*(*this).sub[0]).box_0, (*src).box_0) + Cost((*(*this).sub[1]).box_0);
        let mut cost1: f32 =
            CostMerge((*(*this).sub[1]).box_0, (*src).box_0) + Cost((*(*this).sub[0]).box_0);
        if cost0 < cost1 {
            Node_Merge(
                (*this).sub[0],
                src,
                &mut *((*this).sub).as_mut_ptr().offset(0),
            );
        } else {
            Node_Merge(
                (*this).sub[1],
                src,
                &mut *((*this).sub).as_mut_ptr().offset(1),
            );
        }
    } else {
        let mut parent_0: *mut Node = Node_Create(
            Box3f_Union((*this).box_0, (*src).box_0),
            std::ptr::null_mut(),
        );
        *prev = parent_0;
        let mut costBase: f32 = Cost((*this).box_0) + Cost((*src).box_0);
        let mut cost0_0: f32 =
            CostMerge((*(*this).sub[0]).box_0, (*src).box_0) + Cost((*(*this).sub[1]).box_0);
        let mut cost1_0: f32 =
            CostMerge((*(*this).sub[1]).box_0, (*src).box_0) + Cost((*(*this).sub[0]).box_0);
        if costBase <= cost0_0 && costBase <= cost1_0 {
            (*parent_0).sub[0] = this;
            (*parent_0).sub[1] = src;
        } else if cost0_0 <= costBase && cost0_0 <= cost1_0 {
            (*parent_0).sub[0 as i32 as usize] = (*this).sub[0];
            (*parent_0).sub[1 as i32 as usize] = (*this).sub[1];
            MemFree(this as *const libc::c_void);
            Node_Merge(
                (*parent_0).sub[0],
                src,
                &mut *((*parent_0).sub).as_mut_ptr().offset(0),
            );
        } else {
            (*parent_0).sub[0 as i32 as usize] = (*this).sub[0];
            (*parent_0).sub[1 as i32 as usize] = (*this).sub[1];
            MemFree(this as *const libc::c_void);
            Node_Merge(
                (*parent_0).sub[1],
                src,
                &mut *((*parent_0).sub).as_mut_ptr().offset(1),
            );
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Add(
    mut this: *mut BoxTree,
    mut box_0: Box3f,
    mut data: *mut libc::c_void,
) {
    Node_Merge((*this).root, Node_Create(box_0, data), &mut (*this).root);
}

unsafe extern "C" fn Node_GetMemory(mut this: *mut Node) -> i32 {
    let mut memory: i32 = ::core::mem::size_of::<Node>() as usize as i32;
    if !((*this).sub[0]).is_null() {
        memory += Node_GetMemory((*this).sub[0]);
    }
    if !((*this).sub[1]).is_null() {
        memory += Node_GetMemory((*this).sub[1]);
    }
    return memory;
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_GetMemory(mut this: *mut BoxTree) -> i32 {
    let mut memory: i32 = ::core::mem::size_of::<BoxTree>() as usize as i32;
    if !((*this).root).is_null() {
        memory += Node_GetMemory((*this).root);
    }
    return memory;
}

unsafe extern "C" fn Node_IntersectRay(mut this: *mut Node, mut o: Vec3, mut di: Vec3) -> bool {
    if !Box3f_IntersectsRay((*this).box_0, o, di) {
        return 0 as i32 != 0;
    }
    if !((*this).sub[0]).is_null() {
        if Node_IntersectRay((*this).sub[0], o, di) {
            return 1 as i32 != 0;
        }
        if Node_IntersectRay((*this).sub[1], o, di) {
            return 1 as i32 != 0;
        }
        return 0 as i32 != 0;
    } else {
        return 1 as i32 != 0;
    };
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_IntersectRay(
    mut this: *mut BoxTree,
    mut matrix: *mut Matrix,
    mut ro: *const Vec3,
    mut rd: *const Vec3,
) -> bool {
    if ((*this).root).is_null() {
        return 0 as i32 != 0;
    }
    let mut inv: *mut Matrix = Matrix_Inverse(matrix);
    let mut invRo = Vec3::ZERO;
    Matrix_MulPoint(inv, &mut invRo, (*ro).x, (*ro).y, (*ro).z);
    let mut invRd = Vec3::ZERO;
    Matrix_MulDir(inv, &mut invRd, (*rd).x, (*rd).y, (*rd).z);
    Matrix_Free(inv);
    return Node_IntersectRay((*this).root, invRo, invRd.recip());
}

unsafe extern "C" fn BoxTree_DrawNode(mut this: *mut Node, mut maxDepth: i32) {
    if maxDepth < 0 as i32 {
        return;
    }
    if !((*this).sub[0]).is_null() || !((*this).sub[1]).is_null() {
        Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
        Draw_Box3(&mut (*this).box_0);
    } else {
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
        Draw_Box3(&mut (*this).box_0);
    }
    if !((*this).sub[0]).is_null() {
        BoxTree_DrawNode((*this).sub[0], maxDepth - 1 as i32);
    }
    if !((*this).sub[1]).is_null() {
        BoxTree_DrawNode((*this).sub[1], maxDepth - 1 as i32);
    }
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Draw(mut this: *mut BoxTree, mut maxDepth: i32) {
    if !((*this).root).is_null() {
        BoxTree_DrawNode((*this).root, maxDepth);
    }
}
