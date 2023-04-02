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
pub struct BoxTree {
    pub root: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub box_0: Box3,
    pub data: *mut libc::c_void,
    pub sub: [*mut Node; 2],
}

#[inline]
unsafe extern "C" fn Node_Create(box_0: Box3, data: *mut libc::c_void) -> *mut Node {
    let this = MemNew!(Node);
    (*this).box_0 = box_0;
    (*this).sub[0] = std::ptr::null_mut();
    (*this).sub[1] = std::ptr::null_mut();
    (*this).data = data;
    this
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Create() -> *mut BoxTree {
    let this = MemNew!(BoxTree);
    (*this).root = std::ptr::null_mut();
    this
}

unsafe extern "C" fn Node_Free(this: *mut Node) {
    if !((*this).sub[0]).is_null() {
        Node_Free((*this).sub[0]);
    }
    if !((*this).sub[1]).is_null() {
        Node_Free((*this).sub[1]);
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Free(this: *mut BoxTree) {
    if !((*this).root).is_null() {
        Node_Free((*this).root);
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_FromMesh(mesh: *mut Mesh) -> *mut BoxTree {
    let this: *mut BoxTree = BoxTree_Create();
    let indexCount: i32 = Mesh_GetIndexCount(mesh);
    let indexData: *const i32 = Mesh_GetIndexData(mesh);
    let vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let mut i: i32 = 0;
    while i < indexCount {
        let v0: *const Vertex = vertexData.offset(*indexData.offset((i + 0) as isize) as isize);
        let v1: *const Vertex = vertexData.offset(*indexData.offset((i + 1) as isize) as isize);
        let v2: *const Vertex = vertexData.offset(*indexData.offset((i + 2) as isize) as isize);
        let box_0: Box3 = Box3::new(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        BoxTree_Add(this, box_0, std::ptr::null_mut());
        i += 3;
    }
    this
}

#[inline]
extern "C" fn Cost(box_0: Box3) -> f32 {
    box_0.volume()
}

#[inline]
extern "C" fn CostMerge(a: Box3, b: Box3) -> f32 {
    Cost(Box3::union(a, b))
}

unsafe extern "C" fn Node_Merge(this: *mut Node, src: *mut Node, prev: *mut *mut Node) {
    if this.is_null() {
        *prev = src;
        return;
    }
    if ((*this).sub[0]).is_null() {
        let parent: *mut Node = Node_Create(
            Box3::union((*this).box_0, (*src).box_0),
            std::ptr::null_mut(),
        );
        *prev = parent;
        (*parent).sub[0] = this;
        (*parent).sub[1] = src;
        // this = parent;
        return;
    }
    if Box3::contains((*this).box_0, (*src).box_0) {
        let cost0: f32 =
            CostMerge((*(*this).sub[0]).box_0, (*src).box_0) + Cost((*(*this).sub[1]).box_0);
        let cost1: f32 =
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
        let parent_0: *mut Node = Node_Create(
            Box3::union((*this).box_0, (*src).box_0),
            std::ptr::null_mut(),
        );
        *prev = parent_0;
        let costBase: f32 = Cost((*this).box_0) + Cost((*src).box_0);
        let cost0_0: f32 =
            CostMerge((*(*this).sub[0]).box_0, (*src).box_0) + Cost((*(*this).sub[1]).box_0);
        let cost1_0: f32 =
            CostMerge((*(*this).sub[1]).box_0, (*src).box_0) + Cost((*(*this).sub[0]).box_0);
        if costBase <= cost0_0 && costBase <= cost1_0 {
            (*parent_0).sub[0] = this;
            (*parent_0).sub[1] = src;
        } else if cost0_0 <= costBase && cost0_0 <= cost1_0 {
            (*parent_0).sub[0] = (*this).sub[0];
            (*parent_0).sub[1] = (*this).sub[1];
            MemFree(this as *const _);
            Node_Merge(
                (*parent_0).sub[0],
                src,
                &mut *((*parent_0).sub).as_mut_ptr().offset(0),
            );
        } else {
            (*parent_0).sub[0] = (*this).sub[0];
            (*parent_0).sub[1] = (*this).sub[1];
            MemFree(this as *const _);
            Node_Merge(
                (*parent_0).sub[1],
                src,
                &mut *((*parent_0).sub).as_mut_ptr().offset(1),
            );
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Add(this: *mut BoxTree, box_0: Box3, data: *mut libc::c_void) {
    Node_Merge((*this).root, Node_Create(box_0, data), &mut (*this).root);
}

unsafe extern "C" fn Node_GetMemory(this: *mut Node) -> i32 {
    let mut memory: i32 = std::mem::size_of::<Node>() as i32;
    if !((*this).sub[0]).is_null() {
        memory += Node_GetMemory((*this).sub[0]);
    }
    if !((*this).sub[1]).is_null() {
        memory += Node_GetMemory((*this).sub[1]);
    }
    memory
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_GetMemory(this: *mut BoxTree) -> i32 {
    let mut memory: i32 = std::mem::size_of::<BoxTree>() as i32;
    if !((*this).root).is_null() {
        memory += Node_GetMemory((*this).root);
    }
    memory
}

unsafe extern "C" fn Node_IntersectRay(this: *mut Node, o: Vec3, di: Vec3) -> bool {
    if !(*this).box_0.intersects_ray(o, di) {
        return false;
    }
    if !((*this).sub[0]).is_null() {
        if Node_IntersectRay((*this).sub[0], o, di) {
            return true;
        }
        if Node_IntersectRay((*this).sub[1], o, di) {
            return true;
        }
        false
    } else {
        true
    }
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_IntersectRay(
    this: *mut BoxTree,
    matrix: *mut Matrix,
    ro: *const Vec3,
    rd: *const Vec3,
) -> bool {
    if ((*this).root).is_null() {
        return false;
    }
    let inv: *mut Matrix = Matrix_Inverse(matrix);
    let mut invRo = Vec3::ZERO;
    Matrix_MulPoint(inv, &mut invRo, (*ro).x, (*ro).y, (*ro).z);
    let mut invRd = Vec3::ZERO;
    Matrix_MulDir(inv, &mut invRd, (*rd).x, (*rd).y, (*rd).z);
    Matrix_Free(inv);
    Node_IntersectRay((*this).root, invRo, invRd.recip())
}

unsafe extern "C" fn BoxTree_DrawNode(this: *mut Node, maxDepth: i32) {
    if maxDepth < 0 {
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
        BoxTree_DrawNode((*this).sub[0], maxDepth - 1);
    }
    if !((*this).sub[1]).is_null() {
        BoxTree_DrawNode((*this).sub[1], maxDepth - 1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Draw(this: *mut BoxTree, maxDepth: i32) {
    if !((*this).root).is_null() {
        BoxTree_DrawNode((*this).root, maxDepth);
    }
}
