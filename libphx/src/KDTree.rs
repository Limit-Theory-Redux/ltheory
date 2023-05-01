use crate::internal::Memory::*;
use crate::Common::*;
use crate::Draw::*;
use crate::Math::Box3;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Matrix::*;
use crate::Mesh::*;
use std::cmp::Ordering;
use std::ptr::slice_from_raw_parts_mut;
use std::slice;

const kMaxLeafSize: i32 = 64;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KDTree {
    pub box_0: Box3,
    pub back: *mut KDTree,
    pub front: *mut KDTree,
    pub elems: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub next: *mut Node,
    pub id: u64,
    pub box_0: Box3,
}

unsafe fn Partition(boxes: *mut Box3, boxCount: i32, dim: i32) -> *mut KDTree {
    let this = MemNew!(KDTree);
    if boxCount <= kMaxLeafSize {
        (*this).box_0 = *boxes.offset(0);
        (*this).back = std::ptr::null_mut();
        (*this).front = std::ptr::null_mut();
        (*this).elems = std::ptr::null_mut();
        for i in 1..boxCount {
            (*this).box_0 = Box3::union((*this).box_0, *boxes.offset(i as isize));
        }

        for i in 0..boxCount {
            let node = MemNew!(Node);
            (*node).box_0 = *boxes.offset(i as isize);
            (*node).next = (*this).elems;
            (*node).id = 0;
            (*this).elems = node;
        }
        return this;
    }

    let sortFn = match dim {
        0 => |a: &Box3, b: &Box3| a.lower.x.partial_cmp(&b.lower.x).unwrap_or(Ordering::Equal),
        1 => |a: &Box3, b: &Box3| a.lower.y.partial_cmp(&b.lower.y).unwrap_or(Ordering::Equal),
        _ => |a: &Box3, b: &Box3| a.lower.z.partial_cmp(&b.lower.z).unwrap_or(Ordering::Equal),
    };
    slice::from_raw_parts_mut(boxes, boxCount as usize).sort_by(sortFn);

    let boxCountBack: i32 = boxCount / 2;
    let boxCountFront: i32 = boxCount - boxCountBack;
    let boxesBack: *mut Box3 = MemNewArray!(Box3, boxCountBack);
    let boxesFront: *mut Box3 = MemNewArray!(Box3, boxCountFront);
    MemCpy(
        boxesBack as *mut _,
        boxes as *const _,
        (boxCountBack as usize).wrapping_mul(std::mem::size_of::<Box3>()),
    );
    MemCpy(
        boxesFront as *mut _,
        boxes.offset(boxCountBack as isize) as *const _,
        (boxCountFront as usize).wrapping_mul(std::mem::size_of::<Box3>()),
    );

    (*this).back = Partition(boxesBack, boxCountBack, (dim + 1) % 3);
    (*this).front = Partition(boxesFront, boxCountFront, (dim + 1) % 3);
    (*this).box_0 = Box3::union((*(*this).back).box_0, (*(*this).front).box_0);
    (*this).elems = std::ptr::null_mut();

    MemFree(boxesBack as *const _);
    MemFree(boxesFront as *const _);
    this
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_FromMesh(mesh: &mut Mesh) -> *mut KDTree {
    let indexCount: i32 = Mesh_GetIndexCount(mesh);
    let indexData: *const i32 = Mesh_GetIndexData(mesh);
    let vertexData: *const Vertex = Mesh_GetVertexData(mesh);

    let boxCount: i32 = indexCount / 3;
    let boxes: *mut Box3 = MemNewArray!(Box3, boxCount);

    for i in (0..indexCount).step_by(3) {
        let v0: *const Vertex = vertexData.offset(*indexData.offset((i + 0) as isize) as isize);
        let v1: *const Vertex = vertexData.offset(*indexData.offset((i + 1) as isize) as isize);
        let v2: *const Vertex = vertexData.offset(*indexData.offset((i + 2) as isize) as isize);
        *boxes.offset((i / 3) as isize) = Box3::new(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
    }

    let this: *mut KDTree = Partition(boxes, boxCount, 0);
    MemFree(boxes as *const _);
    this
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_Free(this: *mut KDTree) {
    if !((*this).back).is_null() {
        KDTree_Free((*this).back);
    }
    if !((*this).front).is_null() {
        KDTree_Free((*this).front);
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        let next: *mut Node = (*elem).next;
        MemFree(elem as *const _);
        elem = next;
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_GetMemory(this: &mut KDTree) -> i32 {
    let mut memory: i32 = std::mem::size_of::<KDTree>() as i32;
    if !(this.back).is_null() {
        memory += KDTree_GetMemory(&mut *this.back);
    }
    if !(this.front).is_null() {
        memory += KDTree_GetMemory(&mut *this.front);
    }
    let mut elem: *mut Node = this.elems;
    while !elem.is_null() {
        memory = (memory as usize).wrapping_add(std::mem::size_of::<Node>()) as i32;
        elem = (*elem).next;
    }
    memory
}

#[no_mangle]
pub extern "C" fn KDTree_IntersectRay(
    _this: &mut KDTree,
    _m: *mut Matrix,
    _a: *const Vec3,
    _b: *const Vec3,
) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_Draw(this: &mut KDTree, maxDepth: i32) {
    if maxDepth < 0 {
        return;
    }
    Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    Draw_Box3(&mut this.box_0);
    if !(this.back).is_null() {
        KDTree_Draw(&mut *this.back, maxDepth - 1);
    }
    if !(this.front).is_null() {
        KDTree_Draw(&mut *this.front, maxDepth - 1);
    }
    // #if 0
    //   Draw_Color(0, 1, 0, 1);
    //   for (Node* elem = self->elems; elem; elem = elem->next)
    //     Draw_Box3(&elem->box);
    // #endif
}
